use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::{Mutex, MutexGuard};

use rusqlite::Connection;
use tauri::{AppHandle, Manager};

use crate::db::{configure_connection, migrate};
use crate::domain::media::ProviderRegistry;
use crate::domain::output::OutputState;
use crate::error::AppError;
use crate::jobs::JobQueue;

/// Process-wide state. Rust owns the database, stage, providers, and job pool.
pub struct AppState {
    pub db: Mutex<Connection>,
    pub output: Mutex<OutputState>,
    /// Empty until a later change registers a `MediaProvider`.
    #[allow(dead_code)]
    pub providers: Mutex<ProviderRegistry>,
    /// Idle pool; later changes enqueue index/thumbnail/download work.
    #[allow(dead_code)]
    pub jobs: JobQueue,
    /// `{app_data}/media` — `week/{lang}/{monday}/…`
    pub media_root: PathBuf,
    /// Set by `week_cancel` to stop an in-flight media download.
    pub week_cancel: AtomicBool,
    /// True while fetch/download holds the operator.
    pub week_busy: AtomicBool,
}

impl AppState {
    /// Opens `app_data_dir/jpresentation.db`, migrates, and seeds a Default profile if needed.
    pub fn open(app: &AppHandle) -> Result<Self, AppError> {
        let dir = app
            .path()
            .app_data_dir()
            .map_err(|e| AppError::Io(e.to_string()))?;
        std::fs::create_dir_all(&dir).map_err(|e| AppError::Io(e.to_string()))?;
        Self::open_with_media(&dir.join("jpresentation.db"), &dir.join("media"))
    }

    /// Opens a specific database file. Used by tests with a temp path.
    #[allow(dead_code)]
    pub fn open_path(db_path: &Path) -> Result<Self, AppError> {
        let media = db_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("media");
        Self::open_with_media(db_path, &media)
    }

    fn open_with_media(db_path: &Path, media_root: &Path) -> Result<Self, AppError> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| AppError::Io(e.to_string()))?;
        }
        std::fs::create_dir_all(media_root).map_err(|e| AppError::Io(e.to_string()))?;
        let conn = Connection::open(db_path)?;
        configure_connection(&conn)?;
        migrate(&conn)?;
        Ok(Self {
            db: Mutex::new(conn),
            output: Mutex::new(OutputState::idle()),
            providers: Mutex::new(ProviderRegistry::new()),
            jobs: JobQueue::new(),
            media_root: media_root.to_path_buf(),
            week_cancel: AtomicBool::new(false),
            week_busy: AtomicBool::new(false),
        })
    }

    /// Locks SQLite. Poison is reported as `Busy` so a panicked command cannot take down the stage.
    pub fn lock_db(&self) -> Result<MutexGuard<'_, Connection>, AppError> {
        self.db.lock().map_err(|_| AppError::Busy)
    }

    /// Locks the in-memory output snapshots.
    pub fn lock_output(&self) -> Result<MutexGuard<'_, OutputState>, AppError> {
        self.output.lock().map_err(|_| AppError::Busy)
    }
}
