//! Power management for Jetson devices.
//!
//! Provides nvpmodel and jetson_clocks integration.

use crate::Result;

/// Power mode settings for nvpmodel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PowerMode {
    /// Maximum performance mode (MAXN)
    Maxn,
    /// 15W power budget
    Power15W,
    /// 7W power budget
    Power7W,
    /// Custom power mode ID
    Custom(u8),
}

impl PowerMode {
    /// Get nvpmodel mode ID.
    #[must_use]
    pub const fn mode_id(&self) -> u8 {
        match self {
            Self::Maxn => 0,
            Self::Power15W => 1,
            Self::Power7W => 2,
            Self::Custom(id) => *id,
        }
    }

    /// Get human-readable name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Maxn => "MAXN",
            Self::Power15W => "15W",
            Self::Power7W => "7W",
            Self::Custom(_) => "Custom",
        }
    }

    /// Get power budget in watts.
    #[must_use]
    pub const fn power_budget_watts(&self) -> Option<u32> {
        match self {
            Self::Maxn => None, // Unlimited
            Self::Power15W => Some(15),
            Self::Power7W => Some(7),
            Self::Custom(_) => None,
        }
    }
}

impl Default for PowerMode {
    fn default() -> Self {
        Self::Maxn
    }
}

impl std::fmt::Display for PowerMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Jetson clocks controller.
#[derive(Debug, Default)]
pub struct JetsonClocks {
    enabled: bool,
}

impl JetsonClocks {
    /// Create new clocks controller.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable maximum clocks.
    ///
    /// # Errors
    ///
    /// Returns an error if clock control fails.
    pub fn enable(&mut self) -> Result<()> {
        // Would run: sudo jetson_clocks
        self.enabled = true;
        Ok(())
    }

    /// Disable maximum clocks (restore defaults).
    ///
    /// # Errors
    ///
    /// Returns an error if clock control fails.
    pub fn disable(&mut self) -> Result<()> {
        // Would run: sudo jetson_clocks --restore
        self.enabled = false;
        Ok(())
    }

    /// Check if maximum clocks are enabled.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// Power profile configuration.
#[derive(Debug, Clone)]
pub struct PowerProfile {
    /// Power mode
    pub mode: PowerMode,
    /// Enable jetson_clocks
    pub enable_clocks: bool,
    /// Fan speed (0-255)
    pub fan_speed: u8,
}

impl PowerProfile {
    /// Maximum performance profile.
    #[must_use]
    pub fn max_performance() -> Self {
        Self {
            mode: PowerMode::Maxn,
            enable_clocks: true,
            fan_speed: 255,
        }
    }

    /// Balanced profile.
    #[must_use]
    pub fn balanced() -> Self {
        Self {
            mode: PowerMode::Power15W,
            enable_clocks: false,
            fan_speed: 128,
        }
    }

    /// Power saving profile.
    #[must_use]
    pub fn power_saver() -> Self {
        Self {
            mode: PowerMode::Power7W,
            enable_clocks: false,
            fan_speed: 64,
        }
    }
}

impl Default for PowerProfile {
    fn default() -> Self {
        Self::balanced()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_mode_id() {
        assert_eq!(PowerMode::Maxn.mode_id(), 0);
        assert_eq!(PowerMode::Power15W.mode_id(), 1);
        assert_eq!(PowerMode::Power7W.mode_id(), 2);
        assert_eq!(PowerMode::Custom(5).mode_id(), 5);
    }

    #[test]
    fn test_power_mode_name() {
        assert_eq!(PowerMode::Maxn.name(), "MAXN");
        assert_eq!(PowerMode::Power15W.name(), "15W");
        assert_eq!(PowerMode::Power7W.name(), "7W");
        assert_eq!(PowerMode::Custom(10).name(), "Custom");
    }

    #[test]
    fn test_power_mode_budget() {
        assert_eq!(PowerMode::Maxn.power_budget_watts(), None);
        assert_eq!(PowerMode::Power15W.power_budget_watts(), Some(15));
        assert_eq!(PowerMode::Power7W.power_budget_watts(), Some(7));
        assert_eq!(PowerMode::Custom(3).power_budget_watts(), None);
    }

    #[test]
    fn test_power_mode_display() {
        assert_eq!(PowerMode::Maxn.to_string(), "MAXN");
        assert_eq!(PowerMode::Power15W.to_string(), "15W");
        assert_eq!(PowerMode::Power7W.to_string(), "7W");
        assert_eq!(PowerMode::Custom(99).to_string(), "Custom");
    }

    #[test]
    fn test_power_mode_default() {
        assert_eq!(PowerMode::default(), PowerMode::Maxn);
    }

    #[test]
    fn test_power_mode_equality() {
        assert_eq!(PowerMode::Maxn, PowerMode::Maxn);
        assert_ne!(PowerMode::Maxn, PowerMode::Power7W);
        assert_eq!(PowerMode::Custom(5), PowerMode::Custom(5));
        assert_ne!(PowerMode::Custom(5), PowerMode::Custom(6));
    }

    #[test]
    fn test_jetson_clocks() {
        let mut clocks = JetsonClocks::new();
        assert!(!clocks.is_enabled());
        clocks.enable().unwrap();
        assert!(clocks.is_enabled());
        clocks.disable().unwrap();
        assert!(!clocks.is_enabled());
    }

    #[test]
    fn test_jetson_clocks_default() {
        let clocks = JetsonClocks::default();
        assert!(!clocks.is_enabled());
    }

    #[test]
    fn test_power_profile_max_performance() {
        let profile = PowerProfile::max_performance();
        assert_eq!(profile.mode, PowerMode::Maxn);
        assert!(profile.enable_clocks);
        assert_eq!(profile.fan_speed, 255);
    }

    #[test]
    fn test_power_profile_balanced() {
        let profile = PowerProfile::balanced();
        assert_eq!(profile.mode, PowerMode::Power15W);
        assert!(!profile.enable_clocks);
        assert_eq!(profile.fan_speed, 128);
    }

    #[test]
    fn test_power_profile_power_saver() {
        let profile = PowerProfile::power_saver();
        assert_eq!(profile.mode, PowerMode::Power7W);
        assert!(!profile.enable_clocks);
        assert_eq!(profile.fan_speed, 64);
    }

    #[test]
    fn test_power_profile_default() {
        let profile = PowerProfile::default();
        assert_eq!(profile.mode, PowerMode::Power15W);
        assert!(!profile.enable_clocks);
        assert_eq!(profile.fan_speed, 128);
    }

    #[test]
    fn test_power_profile_clone() {
        let profile = PowerProfile::max_performance();
        let cloned = profile.clone();
        assert_eq!(cloned.mode, profile.mode);
        assert_eq!(cloned.enable_clocks, profile.enable_clocks);
        assert_eq!(cloned.fan_speed, profile.fan_speed);
    }

    #[test]
    fn test_power_mode_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(PowerMode::Maxn);
        set.insert(PowerMode::Maxn);
        assert_eq!(set.len(), 1);
        set.insert(PowerMode::Power7W);
        assert_eq!(set.len(), 2);
    }
}
