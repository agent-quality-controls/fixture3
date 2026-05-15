use std::num::NonZeroUsize;

use fixture3_ddmin::{DdminInput, DdminOptions, ddmin};

use crate::error::AppError;

pub(crate) fn run(args: &crate::args::ReduceArgs) -> Result<super::report::ReduceReport, AppError> {
    let candidates = super::candidate::collect(&args.fixture_root)?;
    crate::fs::create_dir_all(&args.work_dir)?;

    let options = DdminOptions::new(NonZeroUsize::MIN, None);
    let input = DdminInput::new(candidates.clone(), options);
    let mut oracle = super::oracle::ReduceOracle::new(
        args.suite.clone(),
        args.manifest.clone(),
        args.fixture_root.clone(),
        args.work_dir.clone(),
    );
    let output = ddmin(input, &mut oracle);
    let reduce_report =
        super::report::ReduceReport::from_ddmin(args, candidates.len(), &output, oracle.calls());
    super::report::write(&args.work_dir, &reduce_report)?;
    Ok(reduce_report)
}
