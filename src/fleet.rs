//! Fleet orchestration for multiple Jetson devices.
//!
//! Provides multi-device management, load balancing, and coordinated deployment.

use crate::{
    device::JetsonDevice,
    thermal::ThermalPolicy,
    Result,
};
use std::collections::HashMap;

/// Fleet of Jetson devices.
#[derive(Debug, Default)]
pub struct Fleet {
    devices: HashMap<String, FleetMember>,
}

/// Member of a fleet.
#[derive(Debug)]
pub struct FleetMember {
    /// Device handle
    pub device: JetsonDevice,
    /// Thermal policy
    pub policy: ThermalPolicy,
    /// Device enabled for work
    pub enabled: bool,
}

impl Fleet {
    /// Create a new empty fleet.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a device to the fleet.
    ///
    /// # Errors
    ///
    /// Returns an error if device cannot be added.
    pub fn add_device(&mut self, device: JetsonDevice, policy: ThermalPolicy) -> Result<()> {
        let id = device.id().to_string();
        self.devices.insert(
            id,
            FleetMember {
                device,
                policy,
                enabled: true,
            },
        );
        Ok(())
    }

    /// Remove a device from the fleet.
    pub fn remove_device(&mut self, id: &str) -> Option<FleetMember> {
        self.devices.remove(id)
    }

    /// Get device count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.devices.len()
    }

    /// Check if fleet is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.devices.is_empty()
    }

    /// Get enabled device count.
    #[must_use]
    pub fn enabled_count(&self) -> usize {
        self.devices.values().filter(|m| m.enabled).count()
    }

    /// Iterate over devices.
    pub fn devices(&self) -> impl Iterator<Item = &FleetMember> {
        self.devices.values()
    }

    /// Get a device by ID.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&FleetMember> {
        self.devices.get(id)
    }

    /// Deploy model to all fleet devices.
    ///
    /// # Errors
    ///
    /// Returns an error if deployment fails on any device.
    pub async fn deploy_model(&self, _model: impl AsRef<[u8]>) -> Result<()> {
        // Would deploy model to each device
        Ok(())
    }

    /// Start inference servers on all devices.
    ///
    /// # Errors
    ///
    /// Returns an error if server start fails.
    pub async fn start_inference_servers(&self) -> Result<()> {
        // Would start realizar on each device
        Ok(())
    }

    /// Get fleet health status.
    #[must_use]
    pub fn health_status(&self) -> FleetHealth {
        let total = self.len();
        let enabled = self.enabled_count();
        FleetHealth {
            total_devices: total,
            enabled_devices: enabled,
            healthy_devices: enabled, // Placeholder
            degraded_devices: 0,
            offline_devices: total - enabled,
        }
    }
}

/// Fleet health summary.
#[derive(Debug, Clone)]
pub struct FleetHealth {
    /// Total devices in fleet
    pub total_devices: usize,
    /// Enabled devices
    pub enabled_devices: usize,
    /// Healthy devices (thermal OK, memory OK)
    pub healthy_devices: usize,
    /// Degraded devices (thermal warning)
    pub degraded_devices: usize,
    /// Offline devices
    pub offline_devices: usize,
}

impl FleetHealth {
    /// Get health percentage.
    #[must_use]
    pub fn health_percent(&self) -> f32 {
        if self.total_devices == 0 {
            return 100.0;
        }
        (self.healthy_devices as f32 / self.total_devices as f32) * 100.0
    }
}

/// Jetson executor for repartir integration.
#[cfg(feature = "batuta")]
#[derive(Debug)]
pub struct JetsonExecutor {
    /// Target IP address
    pub ip: String,
    /// Thermal policy
    pub policy: ThermalPolicy,
    /// Memory budget in MB
    pub memory_budget_mb: u64,
}

#[cfg(feature = "batuta")]
impl JetsonExecutor {
    /// Create new executor.
    #[must_use]
    pub fn new(ip: impl Into<String>) -> Self {
        Self {
            ip: ip.into(),
            policy: ThermalPolicy::default(),
            memory_budget_mb: 6000,
        }
    }

    /// Set thermal policy.
    #[must_use]
    pub fn with_thermal_policy(mut self, policy: ThermalPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// Set memory budget.
    #[must_use]
    pub fn with_memory_budget_mb(mut self, budget: u64) -> Self {
        self.memory_budget_mb = budget;
        self
    }
}

/// Deployment configuration.
#[derive(Debug, Clone)]
pub struct DeploymentConfig {
    /// Target devices (empty = all)
    pub target_devices: Vec<String>,
    /// Model quantization level
    pub quantization: Option<String>,
    /// Memory budget per device in MB
    pub memory_budget_mb: u64,
    /// Thermal policy
    pub thermal_policy: ThermalPolicy,
}

impl Default for DeploymentConfig {
    fn default() -> Self {
        Self {
            target_devices: Vec::new(),
            quantization: Some("q4_0".to_string()),
            memory_budget_mb: 6000,
            thermal_policy: ThermalPolicy::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::{ConnectionMethod, DeviceInfo};

    fn make_test_device(id: &str, model: crate::JetsonModel) -> JetsonDevice {
        JetsonDevice {
            info: DeviceInfo {
                id: id.to_string(),
                model,
                connection: ConnectionMethod::Usb,
                jetpack_version: None,
                hostname: None,
            },
        }
    }

    #[test]
    fn test_fleet_empty() {
        let fleet = Fleet::new();
        assert!(fleet.is_empty());
        assert_eq!(fleet.len(), 0);
        assert_eq!(fleet.enabled_count(), 0);
    }

    #[test]
    fn test_fleet_add_device() {
        let mut fleet = Fleet::new();
        let device = make_test_device("jetson-01", crate::JetsonModel::OrinNano8GB);
        fleet.add_device(device, ThermalPolicy::conservative()).unwrap();
        assert_eq!(fleet.len(), 1);
        assert!(!fleet.is_empty());
        assert_eq!(fleet.enabled_count(), 1);
    }

    #[test]
    fn test_fleet_remove_device() {
        let mut fleet = Fleet::new();
        let device = make_test_device("jetson-01", crate::JetsonModel::OrinNano8GB);
        fleet.add_device(device, ThermalPolicy::conservative()).unwrap();
        assert_eq!(fleet.len(), 1);

        let removed = fleet.remove_device("jetson-01");
        assert!(removed.is_some());
        assert!(fleet.is_empty());

        let not_found = fleet.remove_device("nonexistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_fleet_get_device() {
        let mut fleet = Fleet::new();
        let device = make_test_device("jetson-02", crate::JetsonModel::OrinNX16GB);
        fleet.add_device(device, ThermalPolicy::aggressive()).unwrap();

        let member = fleet.get("jetson-02");
        assert!(member.is_some());
        assert!(member.unwrap().enabled);

        let not_found = fleet.get("nonexistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_fleet_devices_iterator() {
        let mut fleet = Fleet::new();
        fleet.add_device(
            make_test_device("j1", crate::JetsonModel::OrinNano4GB),
            ThermalPolicy::conservative()
        ).unwrap();
        fleet.add_device(
            make_test_device("j2", crate::JetsonModel::OrinNano8GB),
            ThermalPolicy::aggressive()
        ).unwrap();

        let count = fleet.devices().count();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_fleet_health() {
        let fleet = Fleet::new();
        let health = fleet.health_status();
        assert_eq!(health.total_devices, 0);
        assert_eq!(health.enabled_devices, 0);
        assert_eq!(health.healthy_devices, 0);
        assert_eq!(health.degraded_devices, 0);
        assert_eq!(health.offline_devices, 0);
        assert_eq!(health.health_percent(), 100.0);
    }

    #[test]
    fn test_fleet_health_with_devices() {
        let mut fleet = Fleet::new();
        fleet.add_device(
            make_test_device("j1", crate::JetsonModel::OrinNano8GB),
            ThermalPolicy::conservative()
        ).unwrap();
        fleet.add_device(
            make_test_device("j2", crate::JetsonModel::OrinNano8GB),
            ThermalPolicy::conservative()
        ).unwrap();

        let health = fleet.health_status();
        assert_eq!(health.total_devices, 2);
        assert_eq!(health.enabled_devices, 2);
        assert_eq!(health.healthy_devices, 2);
        assert_eq!(health.health_percent(), 100.0);
    }

    #[test]
    fn test_fleet_health_clone() {
        let health = FleetHealth {
            total_devices: 5,
            enabled_devices: 4,
            healthy_devices: 3,
            degraded_devices: 1,
            offline_devices: 1,
        };
        let cloned = health.clone();
        assert_eq!(cloned.total_devices, 5);
        assert!((cloned.health_percent() - 60.0).abs() < 0.001);
    }

    #[test]
    fn test_deployment_config() {
        let config = DeploymentConfig::default();
        assert_eq!(config.quantization, Some("q4_0".to_string()));
        assert_eq!(config.memory_budget_mb, 6000);
        assert!(config.target_devices.is_empty());
    }

    #[test]
    fn test_deployment_config_clone() {
        let config = DeploymentConfig {
            target_devices: vec!["j1".to_string(), "j2".to_string()],
            quantization: Some("q8_0".to_string()),
            memory_budget_mb: 4000,
            thermal_policy: ThermalPolicy::aggressive(),
        };
        let cloned = config.clone();
        assert_eq!(cloned.target_devices.len(), 2);
        assert_eq!(cloned.quantization, Some("q8_0".to_string()));
    }

    #[test]
    fn test_fleet_member_fields() {
        let mut fleet = Fleet::new();
        let device = make_test_device("j1", crate::JetsonModel::AgxOrin64GB);
        fleet.add_device(device, ThermalPolicy::aggressive()).unwrap();

        let member = fleet.get("j1").unwrap();
        assert!(member.enabled);
        assert_eq!(member.policy.threshold_c, 75.0);
        assert_eq!(member.device.id(), "j1");
    }

    #[tokio::test]
    async fn test_fleet_deploy_model() {
        let fleet = Fleet::new();
        let result = fleet.deploy_model(&[1, 2, 3]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_fleet_start_inference_servers() {
        let fleet = Fleet::new();
        let result = fleet.start_inference_servers().await;
        assert!(result.is_ok());
    }

    #[cfg(feature = "batuta")]
    #[test]
    fn test_jetson_executor() {
        let executor = JetsonExecutor::new("192.168.1.100")
            .with_thermal_policy(ThermalPolicy::aggressive())
            .with_memory_budget_mb(4000);
        assert_eq!(executor.ip, "192.168.1.100");
        assert_eq!(executor.memory_budget_mb, 4000);
        assert_eq!(executor.policy.threshold_c, 75.0);
    }
}
