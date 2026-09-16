use serde::{Deserialize, Serialize};

use crate::error::AppError;

/// Per-profile midweek and weekend meeting times in the machine's local timezone.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeetingScheduleSetting {
    /// ISO-8601 weekday, 1 = Monday … 7 = Sunday.
    pub midweek_weekday: u8,
    /// 24h `HH:mm`.
    pub midweek_time: String,
    pub weekend_weekday: u8,
    pub weekend_time: String,
}

impl Default for MeetingScheduleSetting {
    fn default() -> Self {
        Self {
            midweek_weekday: 2,
            midweek_time: "19:00".into(),
            weekend_weekday: 7,
            weekend_time: "10:00".into(),
        }
    }
}

/// Rejects weekdays outside 1–7 and times that are not zero-padded `HH:mm`.
pub fn validate_meeting_schedule(value: &MeetingScheduleSetting) -> Result<(), AppError> {
    if !is_weekday(value.midweek_weekday) || !is_weekday(value.weekend_weekday) {
        return Err(AppError::Invariant("weekday must be 1-7".into()));
    }
    if !is_hhmm(&value.midweek_time) || !is_hhmm(&value.weekend_time) {
        return Err(AppError::Invariant("time must be HH:mm".into()));
    }
    Ok(())
}

fn is_weekday(value: u8) -> bool {
    (1..=7).contains(&value)
}

fn is_hhmm(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 5 || bytes[2] != b':' {
        return false;
    }
    let Ok(hour) = value[..2].parse::<u8>() else {
        return false;
    };
    let Ok(minute) = value[3..].parse::<u8>() else {
        return false;
    };
    hour <= 23 && minute <= 59
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_tuesday_evening_sunday_morning() {
        let value = MeetingScheduleSetting::default();
        assert_eq!(value.midweek_weekday, 2);
        assert_eq!(value.midweek_time, "19:00");
        assert_eq!(value.weekend_weekday, 7);
        assert_eq!(value.weekend_time, "10:00");
        validate_meeting_schedule(&value).expect("default");
    }

    #[test]
    fn rejects_weekday_out_of_range() {
        let mut value = MeetingScheduleSetting::default();
        value.midweek_weekday = 0;
        assert!(validate_meeting_schedule(&value).is_err());
        value.midweek_weekday = 8;
        assert!(validate_meeting_schedule(&value).is_err());
    }

    #[test]
    fn rejects_unpadded_and_invalid_times() {
        let mut value = MeetingScheduleSetting::default();
        value.midweek_time = "9:00".into();
        assert!(validate_meeting_schedule(&value).is_err());
        value.midweek_time = "24:00".into();
        assert!(validate_meeting_schedule(&value).is_err());
        value.midweek_time = "19:00".into();
        assert!(validate_meeting_schedule(&value).is_ok());
    }
}
