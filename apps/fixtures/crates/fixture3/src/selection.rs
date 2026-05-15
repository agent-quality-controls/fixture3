use crate::error::AppError;
use crate::manifest::Manifest;

pub(crate) type SuiteNames = Vec<String>;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Selector<'a> {
    pub(crate) suite: Option<&'a str>,
    pub(crate) all: bool,
    pub(crate) tag: Option<&'a str>,
    pub(crate) feature: Option<&'a str>,
    pub(crate) default_all: bool,
}

pub(crate) fn suite_names(
    manifest: &Manifest,
    selector: Selector<'_>,
) -> Result<SuiteNames, AppError> {
    if selector.all
        || selector.default_all
            && selector.suite.is_none()
            && selector.tag.is_none()
            && selector.feature.is_none()
    {
        return Ok(manifest.suites.keys().cloned().collect());
    }

    if let Some(name) = selector.suite {
        if !manifest.suites.contains_key(name) {
            return Err(AppError::Manifest(format!("suite not found in manifest: {name}")));
        }
        return Ok(vec![name.to_owned()]);
    }

    if let Some(tag) = selector.tag {
        let names = manifest
            .suites
            .iter()
            .filter(|(_, suite)| suite.tags.iter().any(|value| value == tag))
            .map(|(name, _)| name.clone())
            .collect::<Vec<_>>();
        if names.is_empty() {
            return Err(AppError::Manifest(format!("tag not found in manifest suites: {tag}")));
        }
        return Ok(names);
    }

    if let Some(feature) = selector.feature {
        let config = manifest.features.get(feature).ok_or_else(|| {
            AppError::Manifest(format!("feature not found in manifest: {feature}"))
        })?;
        let mut names = Vec::new();
        for suite_name in &config.suites {
            if !manifest.suites.contains_key(suite_name) {
                return Err(AppError::Manifest(format!(
                    "feature {feature} references missing suite: {suite_name}"
                )));
            }
            names.push(suite_name.clone());
        }
        if names.is_empty() {
            return Err(AppError::Manifest(format!("feature has no suites: {feature}")));
        }
        return Ok(names);
    }

    Err(AppError::Manifest("suite target is required".to_owned()))
}
