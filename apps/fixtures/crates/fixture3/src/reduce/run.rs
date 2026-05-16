use std::collections::BTreeSet;

use fixture3_ddmin::{DdminInput, DdminOptions, ddmin};

use crate::error::AppError;

const DEFAULT_DIR_ORACLE_CALLS: usize = 200;
const DEFAULT_FILE_ORACLE_CALLS: usize = 100;

pub(crate) fn run(args: &crate::args::ReduceArgs) -> Result<super::report::ReduceReport, AppError> {
    let plan = super::reducer::ReducerPlan::parse(&args.reducers)?;
    let candidates = super::candidate::collect(&args.fixture_root)?;
    let directory_candidates = super::directory::collect(&candidates)?;
    crate::fs::create_dir_all(&args.work_dir)?;
    crate::fs::remove_dir_all(&args.work_dir.join("best"))?;
    crate::fs::remove_dir_all(&args.work_dir.join("trials"))?;
    crate::fs::remove_dir_all(&args.work_dir.join("trial-current"))?;

    let mut oracle = super::oracle::ReduceOracle::new(
        args.suite.clone(),
        args.manifest.clone(),
        args.fixture_root.clone(),
        args.work_dir.clone(),
        candidates.clone(),
    );
    let mut state = super::state::ReductionState::new(candidates);
    let mut phases = Vec::new();
    let mut guarantee = "complete".to_owned();
    let base_context = super::report::ReportContext::new(plan.names(), directory_candidates.len());

    for reducer in plan.reducers() {
        if remaining_budget(args, oracle.calls()) == Some(0) {
            "incomplete:max-oracle-calls-reached".clone_into(&mut guarantee);
            break;
        }

        let context = base_context.with_phases(phases.clone());
        let phase = match reducer {
            super::reducer::ReducerKind::Dirs => run_directory_reducer(
                args,
                &directory_candidates,
                &mut state,
                &mut oracle,
                &context,
            ),
            super::reducer::ReducerKind::Files => {
                run_file_reducer(args, &mut state, &mut oracle, &context)
            }
        };
        guarantee = phase_guarantee(&phase);
        phases.push(phase);
        if guarantee != "complete" {
            break;
        }
    }

    let context = base_context.with_phases(phases);
    let reduce_report = super::report::ReduceReport::from_state(
        args,
        &context,
        &state,
        oracle.progress(),
        guarantee,
    );
    super::report::write(&args.work_dir, &reduce_report)?;
    Ok(reduce_report)
}

fn run_directory_reducer(
    args: &crate::args::ReduceArgs,
    directories: &[super::directory::DirectoryCandidate],
    state: &mut super::state::ReductionState,
    oracle: &mut super::oracle::ReduceOracle,
    context: &super::report::ReportContext,
) -> super::report::PhaseReport {
    let _covered_file_count =
        directories.iter().map(super::directory::DirectoryCandidate::file_count).sum::<usize>();
    let mut guarantee = "complete".to_owned();
    let phase_start_calls = oracle.calls();

    for (_depth, layer) in super::directory::by_depth(directories) {
        let active = layer
            .into_iter()
            .filter(|candidate| !state.directory_is_removed(candidate))
            .collect::<Vec<_>>();
        if active.is_empty() {
            continue;
        }
        let Some(options) =
            ddmin_options(args, oracle.calls(), phase_start_calls, DEFAULT_DIR_ORACLE_CALLS)
        else {
            "incomplete:max-oracle-calls-reached".clone_into(&mut guarantee);
            break;
        };

        let input = DdminInput::new(active.clone(), options);
        let output = ddmin(input, &mut |remaining: &[super::directory::DirectoryCandidate]| {
            let remaining_ids = remaining
                .iter()
                .map(super::directory::DirectoryCandidate::id)
                .collect::<BTreeSet<_>>();
            let removed = active
                .iter()
                .filter(|candidate| !remaining_ids.contains(&candidate.id()))
                .map(|candidate| candidate.relative_path().to_path_buf());
            let trial_state = state.with_removed_directories(removed);
            oracle.evaluate_state(&trial_state, context)
        });

        state.apply_removed_directories(
            output.removed().iter().map(|candidate| candidate.relative_path().to_path_buf()),
        );
        guarantee = super::report::guarantee_text(output.guarantee());
        if guarantee != "complete" {
            break;
        }
    }

    let remaining_count =
        directories.iter().filter(|candidate| !state.directory_is_removed(candidate)).count();
    super::report::PhaseReport::new("dirs", directories.len(), remaining_count, guarantee)
}

fn run_file_reducer(
    args: &crate::args::ReduceArgs,
    state: &mut super::state::ReductionState,
    oracle: &mut super::oracle::ReduceOracle,
    context: &super::report::ReportContext,
) -> super::report::PhaseReport {
    let active = state.remaining_files();
    let phase_start_calls = oracle.calls();
    let Some(options) =
        ddmin_options(args, oracle.calls(), phase_start_calls, DEFAULT_FILE_ORACLE_CALLS)
    else {
        return super::report::PhaseReport::new(
            "files",
            active.len(),
            active.len(),
            "incomplete:max-oracle-calls-reached".to_owned(),
        );
    };

    let input = DdminInput::new(active.clone(), options);
    let output = ddmin(input, &mut |remaining: &[super::candidate::FileCandidate]| {
        let trial_state = state.with_file_candidates(remaining);
        oracle.evaluate_state(&trial_state, context)
    });
    state.apply_file_candidates(output.remaining());

    super::report::PhaseReport::new(
        "files",
        active.len(),
        state.remaining_files().len(),
        super::report::guarantee_text(output.guarantee()),
    )
}

fn ddmin_options(
    args: &crate::args::ReduceArgs,
    calls: usize,
    phase_start_calls: usize,
    default_phase_budget: usize,
) -> Option<DdminOptions> {
    let max_calls = if let Some(remaining) = remaining_budget(args, calls) {
        if remaining == 0 {
            return None;
        }
        Some(remaining)
    } else {
        let used_in_phase = calls.saturating_sub(phase_start_calls);
        let remaining = default_phase_budget.saturating_sub(used_in_phase);
        if remaining == 0 {
            return None;
        }
        Some(remaining)
    };
    Some(DdminOptions::new(std::num::NonZeroUsize::MIN, max_calls))
}

fn remaining_budget(args: &crate::args::ReduceArgs, calls: usize) -> Option<usize> {
    args.max_oracle_calls.map(|limit| limit.get().saturating_sub(calls))
}

fn phase_guarantee(phase: &super::report::PhaseReport) -> String {
    phase.guarantee().to_owned()
}
