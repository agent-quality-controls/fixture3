use std::collections::BTreeSet;

use crate::error::AppError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ReducerKind {
    Dirs,
    Files,
}

#[derive(Clone, Debug)]
pub(crate) struct ReducerPlan {
    reducers: Vec<ReducerKind>,
}

impl ReducerKind {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Dirs => "dirs",
            Self::Files => "files",
        }
    }
}

impl ReducerPlan {
    pub(crate) fn parse(raw: &str) -> Result<Self, AppError> {
        let mut reducers = Vec::new();
        let mut seen = BTreeSet::new();
        for item in raw.split(',') {
            let trimmed = item.trim();
            let reducer = match trimmed {
                "dirs" => ReducerKind::Dirs,
                "files" => ReducerKind::Files,
                "" => {
                    return Err(AppError::Manifest(
                        "reducer list contains an empty reducer name".to_owned(),
                    ));
                }
                other => {
                    return Err(AppError::Manifest(format!(
                        "unknown reducer: {other}; expected dirs or files"
                    )));
                }
            };
            if !seen.insert(reducer.name()) {
                return Err(AppError::Manifest(format!("duplicate reducer: {}", reducer.name())));
            }
            reducers.push(reducer);
        }
        if reducers.is_empty() {
            return Err(AppError::Manifest("reducer list is empty".to_owned()));
        }
        Ok(Self { reducers })
    }

    pub(crate) fn reducers(&self) -> &[ReducerKind] {
        &self.reducers
    }

    pub(crate) fn names(&self) -> Vec<String> {
        self.reducers.iter().map(|reducer| reducer.name().to_owned()).collect()
    }
}
