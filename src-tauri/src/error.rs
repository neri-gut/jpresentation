use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Domain/repository failure. Commands map this to [`AppErrorDto`].
#[derive(Debug, Error)]
pub enum AppError {
    #[error("not found")]
    NotFound,
    #[error("busy")]
    Busy,
    #[error("cannot delete the last profile")]
    LastProfile,
    #[error("unknown content language")]
    UnknownContentLanguage,
    #[error("monitor missing")]
    MonitorMissing,
    #[error("invariant: {0}")]
    Invariant(String),
    #[error("io: {0}")]
    Io(String),
    #[error("db: {0}")]
    Db(String),
}

impl AppError {
    /// Stable machine code for logs and the console toast. Never includes user paths.
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound => "NotFound",
            Self::Busy => "Busy",
            Self::LastProfile => "LastProfile",
            Self::UnknownContentLanguage => "UnknownContentLanguage",
            Self::MonitorMissing => "MonitorMissing",
            Self::Invariant(_) => "Invariant",
            Self::Io(_) => "Io",
            Self::Db(_) => "Db",
        }
    }
}

/// IPC error payload. Vue shows this on the console; the audience webview is not closed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppErrorDto {
    pub code: String,
    pub message: String,
    pub rev: u64,
}

impl From<AppError> for AppErrorDto {
    fn from(err: AppError) -> Self {
        Self {
            code: err.code().to_string(),
            message: err.to_string(),
            rev: 0,
        }
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        Self::Db(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dto_preserves_code() {
        let dto = AppErrorDto::from(AppError::NotFound);
        assert_eq!(dto.code, "NotFound");
        assert_eq!(dto.rev, 0);
        assert_eq!(AppErrorDto::from(AppError::LastProfile).code, "LastProfile");
        assert_eq!(
            AppErrorDto::from(AppError::UnknownContentLanguage).code,
            "UnknownContentLanguage"
        );
        assert_eq!(
            AppErrorDto::from(AppError::MonitorMissing).code,
            "MonitorMissing"
        );
    }
}
