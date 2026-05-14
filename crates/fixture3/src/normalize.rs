use crate::command;
use crate::error::AppError;
use crate::manifest::{OutputConfig, OutputFormat};

pub(crate) fn normalize(output: &[u8], config: &OutputConfig) -> Result<String, AppError> {
    let normalized_input = if let Some(normalizer) = &config.normalizer {
        command::run_stdin_command(&normalizer.argv, output)?.stdout
    } else {
        output.to_vec()
    };

    match config.format {
        OutputFormat::Json => normalize_json(&normalized_input),
    }
}

fn normalize_json(output: &[u8]) -> Result<String, AppError> {
    let value: serde_json::Value = serde_json::from_slice(output)
        .map_err(|source| AppError::Json { context: "command stdout".to_owned(), source })?;
    let mut normalized = serde_json::to_string_pretty(&value)
        .map_err(|source| AppError::Json { context: "normalized output".to_owned(), source })?;
    normalized.push('\n');
    Ok(normalized)
}
