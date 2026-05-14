use glob::glob;
use serde::Serialize;

use crate::fs;
use crate::manifest::{Manifest, SuiteConfig};

#[derive(Debug, Serialize)]
pub(crate) struct Finding {
    pub(crate) code: &'static str,
    pub(crate) message: String,
}

pub(crate) fn inspect(manifest: &Manifest) -> Vec<Finding> {
    let mut findings = Vec::new();
    inspect_features(manifest, &mut findings);
    inspect_suites(manifest, &mut findings);
    findings
}

fn inspect_features(manifest: &Manifest, findings: &mut Vec<Finding>) {
    if manifest.suites.is_empty() {
        findings.push(finding("no_suites", "manifest defines no suites"));
    }

    for (feature_name, feature) in &manifest.features {
        if feature.suites.is_empty() {
            findings
                .push(finding("feature_empty", &format!("feature has no suites: {feature_name}")));
        }
        for suite_name in &feature.suites {
            if !manifest.suites.contains_key(suite_name) {
                findings.push(finding(
                    "feature_missing_suite",
                    &format!("feature {feature_name} references missing suite: {suite_name}"),
                ));
            }
        }
    }
}

fn inspect_suites(manifest: &Manifest, findings: &mut Vec<Finding>) {
    for (suite_name, suite) in &manifest.suites {
        inspect_fixture_globs(suite_name, suite, findings);
        inspect_command(suite_name, suite, findings);
        inspect_storage(suite_name, suite, findings);
    }
}

fn inspect_fixture_globs(suite_name: &str, suite: &SuiteConfig, findings: &mut Vec<Finding>) {
    if suite.fixtures.is_empty() {
        findings
            .push(finding("fixtures_empty", &format!("suite has no fixture globs: {suite_name}")));
    }
    for pattern in &suite.fixtures {
        match glob(pattern) {
            Ok(mut entries) => {
                if entries.find_map(Result::ok).is_none() {
                    findings.push(finding(
                        "fixtures_no_matches",
                        &format!("suite {suite_name} fixture glob matched no files: {pattern}"),
                    ));
                }
            }
            Err(error) => findings.push(finding(
                "fixtures_invalid_glob",
                &format!("suite {suite_name} has invalid fixture glob {pattern}: {error}"),
            )),
        }
    }
}

fn inspect_command(suite_name: &str, suite: &SuiteConfig, findings: &mut Vec<Finding>) {
    if suite.command.argv.is_empty() {
        findings
            .push(finding("command_empty", &format!("suite has empty command argv: {suite_name}")));
    }
    if suite.command.ok_exit_codes.is_empty() {
        findings.push(finding(
            "exit_codes_empty",
            &format!("suite has no ok_exit_codes: {suite_name}"),
        ));
    }
    if let Some(normalizer) = &suite.output.normalizer
        && normalizer.argv.is_empty()
    {
        findings.push(finding(
            "normalizer_empty",
            &format!("suite has empty normalizer argv: {suite_name}"),
        ));
    }
}

fn inspect_storage(suite_name: &str, suite: &SuiteConfig, findings: &mut Vec<Finding>) {
    let approved = suite.storage.approved.join("approved.normalized.json");
    if !fs::exists(&approved) {
        findings.push(finding(
            "approved_missing",
            &format!("suite {suite_name} missing approved output: {}", approved.display()),
        ));
    }
    if suite.storage.approved == suite.storage.received {
        findings.push(finding(
            "storage_collision",
            &format!("suite {suite_name} approved_dir and received_dir are the same"),
        ));
    }
    if suite.storage.approved == suite.storage.diff {
        findings.push(finding(
            "storage_collision",
            &format!("suite {suite_name} approved_dir and diff_dir are the same"),
        ));
    }
}

fn finding(code: &'static str, message: &str) -> Finding {
    Finding { code, message: message.to_owned() }
}
