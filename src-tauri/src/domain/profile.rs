use serde::{Deserialize, Serialize};

use crate::error::AppError;

/// Opaque profile identifier. Serialized as a UUID string over IPC.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProfileId(pub String);

impl ProfileId {
    /// Borrow the UUID string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Profile row as seen by Vue. `content_locale` stores a JW `langwritten` code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileDto {
    pub id: ProfileId,
    pub name: String,
    pub ui_locale: String,
    pub content_locale: String,
    pub media_provider: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Payload for `profile_create`. New profiles always start with UI locale `en`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProfileDto {
    pub name: String,
}

/// Payload for `profile_select`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectProfileDto {
    pub id: String,
}

/// Payload for `profile_update`. Only provided fields are written.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProfileDto {
    pub id: String,
    pub name: Option<String>,
    pub ui_locale: Option<String>,
    pub content_locale: Option<String>,
}

/// Payload for `profile_duplicate`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateProfileDto {
    pub id: String,
}

/// Payload for `profile_delete`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteProfileDto {
    pub id: String,
}

/// Lookup key for `settings_get`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingKeyDto {
    pub profile_id: String,
    pub key: String,
}

/// One settings cell. `value_json` is a JSON object string for a named key, never a bag of primitives.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingDto {
    pub profile_id: String,
    pub key: String,
    pub value_json: String,
}

/// Console appearance stored per profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceSetting {
    pub theme: String,
    pub accent: String,
    pub density: String,
}

impl Default for AppearanceSetting {
    fn default() -> Self {
        Self {
            theme: "system".into(),
            accent: "blue".into(),
            density: "compact".into(),
        }
    }
}

/// Operator right-panel chrome stored per profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelSetting {
    pub collapsed: bool,
    pub width: u32,
}

impl Default for PanelSetting {
    fn default() -> Self {
        Self {
            collapsed: false,
            width: 280,
        }
    }
}

/// Persistence port for congregation profiles. Implementations live in `db`.
pub trait ProfileStore {
    /// Every profile, name order.
    fn list(&self) -> Result<Vec<ProfileDto>, AppError>;

    /// Inserts a profile. `ui_locale` is always `en` for a new row.
    fn create(&self, input: CreateProfileDto) -> Result<ProfileDto, AppError>;

    /// Marks the profile as last used by bumping `updated_at`.
    fn select(&self, id: &ProfileId) -> Result<ProfileDto, AppError>;

    /// Last used profile, or `NotFound` if the database is empty.
    fn selected(&self) -> Result<ProfileDto, AppError>;

    /// Partial update of name, UI locale, and/or content locale.
    fn update(&self, input: UpdateProfileDto) -> Result<ProfileDto, AppError>;

    /// Copies the profile and all of its settings. Does not select the copy.
    fn duplicate(&self, id: &ProfileId) -> Result<ProfileDto, AppError>;

    /// Deletes a profile. Returns the profile that remains selected.
    fn delete(&self, id: &ProfileId) -> Result<ProfileDto, AppError>;

    /// Fetch one profile by id.
    fn get(&self, id: &ProfileId) -> Result<ProfileDto, AppError>;
}

/// Trims and rejects empty or overly long congregation names.
pub fn validate_profile_name(name: &str) -> Result<String, AppError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(AppError::Invariant("profile name is required".into()));
    }
    if trimmed.chars().count() > 80 {
        return Err(AppError::Invariant("profile name is too long".into()));
    }
    Ok(trimmed.to_string())
}

/// Closed theme / accent / density tokens. No free hex.
pub fn validate_appearance(value: &AppearanceSetting) -> Result<(), AppError> {
    match value.theme.as_str() {
        "system" | "light" | "dark" => {}
        _ => return Err(AppError::Invariant("unknown theme".into())),
    }
    match value.accent.as_str() {
        "blue" | "teal" | "violet" | "amber" => {}
        _ => return Err(AppError::Invariant("unknown accent".into())),
    }
    match value.density.as_str() {
        "compact" | "comfortable" => {}
        _ => return Err(AppError::Invariant("unknown density".into())),
    }
    Ok(())
}

/// Name for a duplicated profile, capped at 80 characters.
pub fn duplicate_name(name: &str) -> String {
    const SUFFIX: &str = " (copy)";
    const MAX: usize = 80;
    let suffix_len = SUFFIX.chars().count();
    if name.chars().count() + suffix_len <= MAX {
        return format!("{name}{SUFFIX}");
    }
    let keep = MAX.saturating_sub(suffix_len);
    let truncated: String = name.chars().take(keep).collect();
    format!("{truncated}{SUFFIX}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_blank_name() {
        assert!(validate_profile_name("   ").is_err());
    }

    #[test]
    fn trims_name() {
        let name = validate_profile_name("  Cong A  ").expect("name");
        assert_eq!(name, "Cong A");
    }

    #[test]
    fn duplicate_name_appends_copy() {
        assert_eq!(duplicate_name("Cong A"), "Cong A (copy)");
    }

    #[test]
    fn duplicate_name_stays_within_80_chars() {
        let long = "A".repeat(80);
        let copied = duplicate_name(&long);
        assert_eq!(copied.chars().count(), 80);
        assert!(copied.ends_with(" (copy)"));
    }

    #[test]
    fn appearance_rejects_free_hex() {
        let mut value = AppearanceSetting::default();
        value.accent = "#ff00aa".into();
        assert!(validate_appearance(&value).is_err());
        value.accent = "teal".into();
        validate_appearance(&value).expect("teal");
    }
}
