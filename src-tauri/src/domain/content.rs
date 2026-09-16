use serde::{Deserialize, Serialize};

use crate::error::AppError;

const SEED_JSON: &str = include_str!("content_languages.json");

/// One JW `langwritten` entry from the embedded seed. Not a BCP-47 UI locale.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentLanguageDto {
    pub langwritten: String,
    pub name: String,
    pub locale: String,
    pub direction: String,
    pub script: String,
}

/// Embedded frequent-language seed (`research/idiomas.md`). No network.
pub fn content_languages() -> Result<Vec<ContentLanguageDto>, AppError> {
    serde_json::from_str(SEED_JSON).map_err(|e| AppError::Invariant(e.to_string()))
}

/// Accepts a JW `langwritten` that exists in the seed. Trims; does not rewrite case.
pub fn validate_content_locale(code: &str) -> Result<String, AppError> {
    let trimmed = code.trim();
    if trimmed.is_empty() {
        return Err(AppError::UnknownContentLanguage);
    }
    let seed = content_languages()?;
    if seed.iter().any(|item| item.langwritten == trimmed) {
        Ok(trimmed.to_string())
    } else {
        Err(AppError::UnknownContentLanguage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_contains_twenty_one_codes() {
        let seed = content_languages().expect("seed json");
        assert_eq!(seed.len(), 21);
        assert!(seed.iter().any(|item| item.langwritten == "E"));
        assert!(seed.iter().any(|item| item.langwritten == "TG"));
        assert!(seed.iter().any(|item| item.langwritten == "A" && item.direction == "rtl"));
    }

    #[test]
    fn rejects_unknown_langwritten() {
        assert!(matches!(
            validate_content_locale("ZZZ"),
            Err(AppError::UnknownContentLanguage)
        ));
        assert!(matches!(
            validate_content_locale("   "),
            Err(AppError::UnknownContentLanguage)
        ));
    }

    #[test]
    fn accepts_seed_code() {
        assert_eq!(validate_content_locale("S").expect("S"), "S");
        assert_eq!(validate_content_locale("  CHS ").expect("CHS"), "CHS");
    }
}
