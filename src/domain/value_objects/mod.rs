use crate::shared::errors::DomainError;
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid;

/// Song ID value object - immutable identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SongId(String);

impl SongId {
    /// Create SongId from file path using MD5 hash
    pub fn from_path(path: &FilePath) -> Self {
        let hash = format!("{:x}", md5::compute(path.as_str()));
        Self(hash)
    }

    /// Create SongId from string (for deserialization)
    #[allow(dead_code)]
    pub fn from_string(id: String) -> Self {
        Self(id)
    }

    /// Get the underlying string value
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SongId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// File path value object with validation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilePath(String);

impl FilePath {
    /// Create a new file path with validation
    pub fn new(path: &str) -> Result<Self, DomainError> {
        if path.trim().is_empty() {
            return Err(DomainError::InvalidFilePath("Path cannot be empty".to_string()));
        }

        // Basic validation - could be extended with more sophisticated checks
        if !path.contains('.') {
            return Err(DomainError::InvalidFilePath("Path must have an extension".to_string()));
        }

        Ok(Self(path.to_string()))
    }

    /// Get the underlying string value
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get file extension
    #[allow(dead_code)]
    pub fn extension(&self) -> Option<&str> {
        std::path::Path::new(&self.0)
            .extension()
            .and_then(|ext| ext.to_str())
    }

    /// Get filename without extension
    #[allow(dead_code)]
    pub fn filename_stem(&self) -> Option<&str> {
        std::path::Path::new(&self.0)
            .file_stem()
            .and_then(|stem| stem.to_str())
    }
}

/// Duration value object with business rules
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Duration(u64); // seconds

impl Duration {
    /// Create duration from seconds
    pub fn from_seconds(seconds: u64) -> Self {
        Self(seconds)
    }

    /// Create duration from minutes and seconds
    #[allow(dead_code)]
    pub fn from_minutes_seconds(minutes: u64, seconds: u64) -> Result<Self, DomainError> {
        if seconds >= 60 {
            return Err(DomainError::InvalidDuration("Seconds must be less than 60".to_string()));
        }
        Ok(Self(minutes * 60 + seconds))
    }

    /// Get total seconds
    pub fn total_seconds(&self) -> u64 {
        self.0
    }

    /// Get minutes component
    #[allow(dead_code)]
    pub fn minutes(&self) -> u64 {
        self.0 / 60
    }

    /// Get seconds component (0-59)
    #[allow(dead_code)]
    pub fn seconds(&self) -> u64 {
        self.0 % 60
    }

    /// Format as MM:SS
    #[allow(dead_code)]
    pub fn format_mm_ss(&self) -> String {
        format!("{:02}:{:02}", self.minutes(), self.seconds())
    }

    /// Format as H:MM:SS for long durations
    #[allow(dead_code)]
    pub fn format_h_mm_ss(&self) -> String {
        let hours = self.0 / 3600;
        let minutes = (self.0 % 3600) / 60;
        let seconds = self.0 % 60;
        
        if hours > 0 {
            format!("{}:{:02}:{:02}", hours, minutes, seconds)
        } else {
            format!("{:02}:{:02}", minutes, seconds)
        }
    }
}

/// Volume value object with validation (0.0 to 1.0)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Volume(f32);

impl Volume {
    /// Create volume with validation
    #[allow(dead_code)]
    pub fn new(value: f32) -> Result<Self, DomainError> {
        if !(0.0..=1.0).contains(&value) {
            return Err(DomainError::InvalidVolume(value));
        }
        Ok(Self(value))
    }

    /// Get the volume value
    #[allow(dead_code)]
    pub fn value(&self) -> f32 {
        self.0
    }

    /// Get volume as percentage (0-100)
    #[allow(dead_code)]
    pub fn as_percentage(&self) -> u8 {
        (self.0 * 100.0) as u8
    }

    /// Increase volume by step, clamped to 1.0
    #[allow(dead_code)]
    pub fn increase(&self, step: f32) -> Self {
        Self((self.0 + step).clamp(0.0, 1.0))
    }

    /// Decrease volume by step, clamped to 0.0
    #[allow(dead_code)]
    pub fn decrease(&self, step: f32) -> Self {
        Self((self.0 - step).clamp(0.0, 1.0))
    }

    /// Check if volume is muted
    #[allow(dead_code)]
    pub fn is_muted(&self) -> bool {
        self.0 == 0.0
    }
}

/// Playlist ID value object
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlaylistId(String);

impl PlaylistId {
    /// Create new playlist ID using UUID
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    /// Create from existing string
    pub fn from_string(id: String) -> Self {
        Self(id)
    }

    /// Get the underlying string value
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for PlaylistId {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_path_validation() {
        assert!(FilePath::new("").is_err());
        assert!(FilePath::new("invalid").is_err());
        assert!(FilePath::new("/path/to/song.mp3").is_ok());
    }

    #[test]
    fn test_duration_formatting() {
        let duration = Duration::from_seconds(125);
        assert_eq!(duration.format_mm_ss(), "02:05");
        assert_eq!(duration.minutes(), 2);
        assert_eq!(duration.seconds(), 5);
    }

    #[test]
    fn test_volume_validation() {
        assert!(Volume::new(-0.1).is_err());
        assert!(Volume::new(1.1).is_err());
        assert!(Volume::new(0.5).is_ok());
        
        let vol = Volume::new(0.5).unwrap();
        assert_eq!(vol.as_percentage(), 50);
    }

    #[test]
    fn test_volume_operations() {
        let vol = Volume::new(0.5).unwrap();
        let increased = vol.increase(0.3);
        let decreased = vol.decrease(0.3);
        
        assert!((increased.value() - 0.8).abs() < f32::EPSILON);
        assert!((decreased.value() - 0.2).abs() < 0.001); // Allow small floating point error
    }
}
