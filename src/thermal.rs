//! Thermal monitoring and management for Jetson devices.
//!
//! Provides tegrastats integration, thermal circuit breakers, and
//! proactive thermal management.

use crate::{device::JetsonDevice, Result};
use std::time::Duration;

/// Thermal statistics from tegrastats.
#[derive(Debug, Clone, Default)]
pub struct TegraStats {
    /// GPU temperature in Celsius
    pub gpu_temp: f32,
    /// CPU temperature in Celsius
    pub cpu_temp: f32,
    /// SOC temperature in Celsius
    pub soc_temp: f32,
    /// Total memory in MB
    pub total_memory_mb: u64,
    /// Used memory in MB
    pub used_memory_mb: u64,
    /// Available memory in MB
    pub available_memory_mb: u64,
    /// GPU utilization percentage
    pub gpu_utilization: f32,
    /// CPU utilization percentage
    pub cpu_utilization: f32,
    /// Power consumption in watts
    pub power_watts: f32,
}

/// Thermal zone types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThermalZone {
    /// GPU thermal zone
    Gpu,
    /// CPU thermal zone
    Cpu,
    /// SOC junction
    Soc,
    /// Board thermal zone
    Board,
}

/// Thermal policy configuration.
#[derive(Debug, Clone)]
pub struct ThermalPolicy {
    /// Temperature threshold to pause work (°C)
    pub threshold_c: f32,
    /// Temperature to resume work after cooldown (°C)
    pub cooldown_c: f32,
    /// Check interval in milliseconds
    pub check_interval_ms: u64,
}

impl ThermalPolicy {
    /// Conservative policy - pause at 65°C, resume at 55°C
    #[must_use]
    pub fn conservative() -> Self {
        Self {
            threshold_c: 65.0,
            cooldown_c: 55.0,
            check_interval_ms: 500,
        }
    }

    /// Aggressive policy - pause at 75°C, resume at 65°C
    #[must_use]
    pub fn aggressive() -> Self {
        Self {
            threshold_c: 75.0,
            cooldown_c: 65.0,
            check_interval_ms: 1000,
        }
    }

    /// Custom policy
    #[must_use]
    pub fn custom(threshold_c: f32, cooldown_c: f32, check_interval_ms: u64) -> Self {
        Self {
            threshold_c,
            cooldown_c,
            check_interval_ms,
        }
    }
}

impl Default for ThermalPolicy {
    fn default() -> Self {
        Self::conservative()
    }
}

/// Monitor for tegrastats data.
#[derive(Debug)]
pub struct TegraMonitor {
    policy: ThermalPolicy,
    last_stats: Option<TegraStats>,
}

impl TegraMonitor {
    /// Create a new monitor with default policy.
    #[must_use]
    pub fn new() -> Self {
        Self {
            policy: ThermalPolicy::default(),
            last_stats: None,
        }
    }

    /// Create a monitor connected to a device.
    ///
    /// # Errors
    ///
    /// Returns an error if connection fails.
    pub fn connect(_device: &JetsonDevice) -> Result<Self> {
        Ok(Self::new())
    }

    /// Set thermal policy.
    pub fn with_policy(mut self, policy: ThermalPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// Sample current stats.
    ///
    /// # Errors
    ///
    /// Returns an error if sampling fails.
    pub fn sample(&mut self) -> Result<TegraStats> {
        // Placeholder - would parse tegrastats output
        let stats = TegraStats {
            gpu_temp: 45.0,
            cpu_temp: 42.0,
            soc_temp: 44.0,
            total_memory_mb: 8192,
            used_memory_mb: 2048,
            available_memory_mb: 6144,
            gpu_utilization: 0.0,
            cpu_utilization: 10.0,
            power_watts: 5.0,
        };
        self.last_stats = Some(stats.clone());
        Ok(stats)
    }

    /// Get GPU temperature.
    ///
    /// # Errors
    ///
    /// Returns an error if temperature read fails.
    pub fn gpu_temp(&mut self) -> Result<f32> {
        let stats = self.sample()?;
        Ok(stats.gpu_temp)
    }

    /// Check if thermal threshold is exceeded.
    ///
    /// # Errors
    ///
    /// Returns an error if temperature read fails.
    pub fn is_throttled(&mut self) -> Result<bool> {
        let temp = self.gpu_temp()?;
        Ok(temp > self.policy.threshold_c)
    }

    /// Wait for cooldown if temperature exceeds threshold.
    ///
    /// # Errors
    ///
    /// Returns an error if monitoring fails.
    pub async fn wait_for_cooldown(&mut self) -> Result<()> {
        loop {
            let temp = self.gpu_temp()?;
            if temp <= self.policy.cooldown_c {
                return Ok(());
            }
            tracing::warn!(
                temp_c = temp,
                target_c = self.policy.cooldown_c,
                "Waiting for thermal cooldown"
            );
            tokio::time::sleep(Duration::from_millis(self.policy.check_interval_ms)).await;
        }
    }
}

impl Default for TegraMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Thermal circuit breaker - Jidoka pattern.
///
/// Automatically stops work when temperature exceeds threshold.
#[derive(Debug)]
pub struct ThermalCircuitBreaker {
    monitor: TegraMonitor,
}

impl ThermalCircuitBreaker {
    /// Create a new circuit breaker.
    #[must_use]
    pub fn new(monitor: TegraMonitor) -> Self {
        Self { monitor }
    }

    /// Check if circuit is open (too hot).
    ///
    /// # Errors
    ///
    /// Returns an error if temperature check fails.
    pub fn is_open(&mut self) -> Result<bool> {
        self.monitor.is_throttled()
    }

    /// Guard work with thermal protection.
    ///
    /// # Errors
    ///
    /// Returns an error if thermal check or work fails.
    pub async fn guard<F, T>(&mut self, work: F) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>>,
    {
        // Check before starting
        if self.is_open()? {
            tracing::warn!("Thermal circuit breaker OPEN - waiting for cooldown");
            self.monitor.wait_for_cooldown().await?;
        }

        work.await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thermal_policy_conservative() {
        let policy = ThermalPolicy::conservative();
        assert_eq!(policy.threshold_c, 65.0);
        assert_eq!(policy.cooldown_c, 55.0);
        assert_eq!(policy.check_interval_ms, 500);
    }

    #[test]
    fn test_thermal_policy_aggressive() {
        let policy = ThermalPolicy::aggressive();
        assert_eq!(policy.threshold_c, 75.0);
        assert_eq!(policy.cooldown_c, 65.0);
        assert_eq!(policy.check_interval_ms, 1000);
    }

    #[test]
    fn test_thermal_policy_custom() {
        let policy = ThermalPolicy::custom(80.0, 70.0, 250);
        assert_eq!(policy.threshold_c, 80.0);
        assert_eq!(policy.cooldown_c, 70.0);
        assert_eq!(policy.check_interval_ms, 250);
    }

    #[test]
    fn test_thermal_policy_default() {
        let policy = ThermalPolicy::default();
        assert_eq!(policy.threshold_c, 65.0);
        assert_eq!(policy.cooldown_c, 55.0);
    }

    #[test]
    fn test_thermal_policy_clone() {
        let policy = ThermalPolicy::aggressive();
        let cloned = policy.clone();
        assert_eq!(cloned.threshold_c, policy.threshold_c);
        assert_eq!(cloned.cooldown_c, policy.cooldown_c);
    }

    #[test]
    fn test_tegra_stats_default() {
        let stats = TegraStats::default();
        assert_eq!(stats.gpu_temp, 0.0);
        assert_eq!(stats.cpu_temp, 0.0);
        assert_eq!(stats.total_memory_mb, 0);
    }

    #[test]
    fn test_tegra_stats_clone() {
        let stats = TegraStats {
            gpu_temp: 55.0,
            cpu_temp: 52.0,
            soc_temp: 53.0,
            total_memory_mb: 8192,
            used_memory_mb: 4096,
            available_memory_mb: 4096,
            gpu_utilization: 75.0,
            cpu_utilization: 50.0,
            power_watts: 10.5,
        };
        let cloned = stats.clone();
        assert_eq!(cloned.gpu_temp, 55.0);
        assert_eq!(cloned.power_watts, 10.5);
    }

    #[test]
    fn test_thermal_zone_variants() {
        assert_eq!(ThermalZone::Gpu, ThermalZone::Gpu);
        assert_ne!(ThermalZone::Gpu, ThermalZone::Cpu);
        assert_ne!(ThermalZone::Soc, ThermalZone::Board);
    }

    #[test]
    fn test_tegra_monitor_new() {
        let monitor = TegraMonitor::new();
        assert!(monitor.last_stats.is_none());
    }

    #[test]
    fn test_tegra_monitor_default() {
        let monitor = TegraMonitor::default();
        assert!(monitor.last_stats.is_none());
    }

    #[test]
    fn test_tegra_monitor_with_policy() {
        let monitor = TegraMonitor::new().with_policy(ThermalPolicy::aggressive());
        assert_eq!(monitor.policy.threshold_c, 75.0);
    }

    #[test]
    fn test_tegra_monitor_sample() {
        let mut monitor = TegraMonitor::new();
        let stats = monitor.sample().unwrap();
        assert!(stats.gpu_temp > 0.0);
        assert!(stats.total_memory_mb > 0);
        assert!(stats.available_memory_mb > 0);
        assert!(monitor.last_stats.is_some());
    }

    #[test]
    fn test_tegra_monitor_gpu_temp() {
        let mut monitor = TegraMonitor::new();
        let temp = monitor.gpu_temp().unwrap();
        assert_eq!(temp, 45.0); // Placeholder value
    }

    #[test]
    fn test_tegra_monitor_is_throttled() {
        let mut monitor = TegraMonitor::new();
        // Placeholder returns 45°C, threshold is 65°C
        assert!(!monitor.is_throttled().unwrap());

        // With aggressive policy threshold 75°C
        let mut monitor2 = TegraMonitor::new().with_policy(ThermalPolicy::aggressive());
        assert!(!monitor2.is_throttled().unwrap());
    }

    #[test]
    fn test_circuit_breaker_new() {
        let monitor = TegraMonitor::new();
        let breaker = ThermalCircuitBreaker::new(monitor);
        assert!(breaker.monitor.last_stats.is_none());
    }

    #[test]
    fn test_circuit_breaker_closed() {
        let monitor = TegraMonitor::new();
        let mut breaker = ThermalCircuitBreaker::new(monitor);
        // With placeholder returning 45°C and threshold 65°C, should be closed
        assert!(!breaker.is_open().unwrap());
    }

    #[tokio::test]
    async fn test_circuit_breaker_guard() {
        let monitor = TegraMonitor::new();
        let mut breaker = ThermalCircuitBreaker::new(monitor);

        let result = breaker.guard(async { Ok(42) }).await;
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_tegra_monitor_connect() {
        use crate::device::{ConnectionMethod, DeviceInfo, JetsonDevice};

        let device = JetsonDevice {
            info: DeviceInfo {
                id: "test".to_string(),
                model: crate::JetsonModel::OrinNano8GB,
                connection: ConnectionMethod::Usb,
                jetpack_version: None,
                hostname: None,
            },
        };

        let monitor = TegraMonitor::connect(&device).unwrap();
        assert!(monitor.last_stats.is_none());
    }
}
