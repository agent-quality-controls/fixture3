use crate::error::AppError;

pub(crate) fn normalize(output: &[u8]) -> Result<String, AppError> {
    normalize_json(output)
}

#[allow(
    clippy::disallowed_methods,
    reason = "normalize_json is the JSON boundary for command stdout"
)]
fn normalize_json(output: &[u8]) -> Result<String, AppError> {
    let value: serde_json::Value = serde_json::from_slice(output).map_err(|source| {
        AppError::Json { context: "suite command stdout must be JSON".to_owned(), source }
    })?;
    let mut normalized = serde_json::to_string_pretty(&value)
        .map_err(|source| AppError::Json { context: "normalized output".to_owned(), source })?;
    normalized.push('\n');
    Ok(normalized)
}
