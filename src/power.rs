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
    }

    #[test]
    fn test_power_mode_display() {
        assert_eq!(PowerMode::Maxn.to_string(), "MAXN");
        assert_eq!(PowerMode::Power15W.to_string(), "15W");
    }

    #[test]
    fn test_jetson_clocks() {
        let mut clocks = JetsonClocks::new();
        assert!(!clocks.is_enabled());
        clocks.enable().unwrap();
        assert!(clocks.is_enabled());
    }

    #[test]
    fn test_power_profiles() {
        let max = PowerProfile::max_performance();
        assert_eq!(max.mode, PowerMode::Maxn);
        assert!(max.enable_clocks);
        assert_eq!(max.fan_speed, 255);
    }
}
