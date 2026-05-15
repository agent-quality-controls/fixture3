use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct DiffReport {
    pub(crate) status: DiffStatus,
    pub(crate) changed: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum DiffStatus {
    Matched,
    Different,
}

impl DiffReport {
    pub(crate) const fn exit_code(&self) -> u8 {
        match self.status {
            DiffStatus::Matched => 0,
            DiffStatus::Different => 1,
        }
    }
}

impl DiffStatus {
    pub(crate) const fn as_str(&self) -> &'static str {
        match self {
            Self::Matched => "matched",
            Self::Different => "different",
        }
    }
}

pub(crate) fn compare(approved: &str, received: &str) -> (DiffReport, String) {
    if approved == received {
        return (
            DiffReport { status: DiffStatus::Matched, changed: false },
            "status: matched\n".to_owned(),
        );
    }

    let text = format!("status: different\n--- approved\n{approved}+++ received\n{received}");
    (DiffReport { status: DiffStatus::Different, changed: true }, text)
}
