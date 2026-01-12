//! Device discovery and management for Jetson devices.
//!
//! This module provides functionality to discover, connect to, and manage
//! NVIDIA Jetson devices via USB, Ethernet, or mDNS.

use crate::{Error, JetsonModel, Result};
use std::net::IpAddr;

/// Connection method to Jetson device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionMethod {
    /// USB-C direct connection (192.168.55.1)
    Usb,
    /// Ethernet connection with IP address
    Ethernet(IpAddr),
    /// mDNS discovery
    Mdns(String),
}

impl Default for ConnectionMethod {
    fn default() -> Self {
        Self::Usb
    }
}

/// Device information for a discovered Jetson.
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    /// Device identifier
    pub id: String,
    /// Jetson model
    pub model: JetsonModel,
    /// Connection method
    pub connection: ConnectionMethod,
    /// JetPack/L4T version
    pub jetpack_version: Option<String>,
    /// Hostname
    pub hostname: Option<String>,
}

/// Handle to a connected Jetson device.
#[derive(Debug)]
pub struct JetsonDevice {
    /// Device information
    pub info: DeviceInfo,
    // SSH session would be stored here
}

impl JetsonDevice {
    /// Discover all Jetson devices on the network and USB.
    ///
    /// # Errors
    ///
    /// Returns an error if discovery fails.
    pub async fn discover_all() -> Result<Vec<Self>> {
        let mut devices = Vec::new();

        // Try USB connection first
        if let Ok(device) = Self::discover_usb().await {
            devices.push(device);
        }

        // Try mDNS discovery
        if let Ok(mdns_devices) = Self::discover_mdns().await {
            devices.extend(mdns_devices);
        }

        Ok(devices)
    }

    /// Discover Jetson via USB-C connection.
    ///
    /// # Errors
    ///
    /// Returns an error if USB discovery fails.
    pub async fn discover_usb() -> Result<Self> {
        // USB gadget network is typically 192.168.55.1
        let info = DeviceInfo {
            id: "jetson-usb".to_string(),
            model: JetsonModel::Unknown,
            connection: ConnectionMethod::Usb,
            jetpack_version: None,
            hostname: None,
        };

        Ok(Self { info })
    }

    /// Discover Jetson devices via mDNS.
    ///
    /// # Errors
    ///
    /// Returns an error if mDNS discovery fails.
    pub async fn discover_mdns() -> Result<Vec<Self>> {
        // Placeholder - would use mdns-sd crate
        Ok(Vec::new())
    }

    /// Connect to a specific IP address.
    ///
    /// # Errors
    ///
    /// Returns an error if connection fails.
    pub async fn connect(ip: IpAddr) -> Result<Self> {
        let info = DeviceInfo {
            id: format!("jetson-{ip}"),
            model: JetsonModel::Unknown,
            connection: ConnectionMethod::Ethernet(ip),
            jetpack_version: None,
            hostname: None,
        };

        Ok(Self { info })
    }

    /// Get device identifier.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.info.id
    }

    /// Get device model.
    #[must_use]
    pub fn model(&self) -> JetsonModel {
        self.info.model
    }

    /// Get device info.
    #[must_use]
    pub fn info(&self) -> &DeviceInfo {
        &self.info
    }

    /// Execute a command on the device.
    ///
    /// # Errors
    ///
    /// Returns an error if command execution fails.
    pub async fn exec(&self, command: &str) -> Result<String> {
        // Placeholder - would use SSH
        Err(Error::Internal(format!(
            "Command execution not implemented: {command}"
        )))
    }

    /// Get available memory in MB.
    ///
    /// # Errors
    ///
    /// Returns an error if memory query fails.
    pub async fn available_memory_mb(&self) -> Result<u64> {
        // Would parse /proc/meminfo via SSH
        Ok(self.info.model.memory_mb() / 2) // Conservative estimate
    }

    /// Get compute hint for trueno backend selection.
    #[must_use]
    pub fn compute_hint(&self) -> ComputeHint {
        ComputeHint {
            prefer_neon: true,
            memory_budget_mb: self.info.model.memory_mb() / 2,
            cuda_available: true,
        }
    }
}

/// Hint for trueno backend selection.
#[derive(Debug, Clone)]
pub struct ComputeHint {
    /// Prefer ARM NEON backend
    pub prefer_neon: bool,
    /// Memory budget in MB
    pub memory_budget_mb: u64,
    /// CUDA available (limited on Jetson)
    pub cuda_available: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_connection_method_default() {
        assert_eq!(ConnectionMethod::default(), ConnectionMethod::Usb);
    }

    #[test]
    fn test_connection_method_ethernet() {
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100));
        let method = ConnectionMethod::Ethernet(ip);
        assert_eq!(method, ConnectionMethod::Ethernet(ip));
    }

    #[test]
    fn test_connection_method_mdns() {
        let method = ConnectionMethod::Mdns("jetson.local".to_string());
        if let ConnectionMethod::Mdns(host) = method {
            assert_eq!(host, "jetson.local");
        } else {
            panic!("Expected Mdns variant");
        }
    }

    #[test]
    fn test_device_info_creation() {
        let info = DeviceInfo {
            id: "jetson-01".to_string(),
            model: JetsonModel::OrinNano8GB,
            connection: ConnectionMethod::Usb,
            jetpack_version: Some("5.1.1".to_string()),
            hostname: Some("jetson-nano".to_string()),
        };
        assert_eq!(info.id, "jetson-01");
        assert_eq!(info.model, JetsonModel::OrinNano8GB);
        assert_eq!(info.jetpack_version, Some("5.1.1".to_string()));
        assert_eq!(info.hostname, Some("jetson-nano".to_string()));
    }

    #[test]
    fn test_compute_hint() {
        let info = DeviceInfo {
            id: "test".to_string(),
            model: JetsonModel::OrinNano8GB,
            connection: ConnectionMethod::Usb,
            jetpack_version: None,
            hostname: None,
        };
        let device = JetsonDevice { info };
        let hint = device.compute_hint();
        assert!(hint.prefer_neon);
        assert_eq!(hint.memory_budget_mb, 4096);
        assert!(hint.cuda_available);
    }

    #[test]
    fn test_device_id() {
        let info = DeviceInfo {
            id: "my-jetson".to_string(),
            model: JetsonModel::OrinNX16GB,
            connection: ConnectionMethod::Usb,
            jetpack_version: None,
            hostname: None,
        };
        let device = JetsonDevice { info };
        assert_eq!(device.id(), "my-jetson");
    }

    #[test]
    fn test_device_model() {
        let info = DeviceInfo {
            id: "test".to_string(),
            model: JetsonModel::AgxOrin64GB,
            connection: ConnectionMethod::Usb,
            jetpack_version: None,
            hostname: None,
        };
        let device = JetsonDevice { info };
        assert_eq!(device.model(), JetsonModel::AgxOrin64GB);
    }

    #[test]
    fn test_device_info_accessor() {
        let info = DeviceInfo {
            id: "test-device".to_string(),
            model: JetsonModel::OrinNano4GB,
            connection: ConnectionMethod::Usb,
            jetpack_version: Some("6.0".to_string()),
            hostname: None,
        };
        let device = JetsonDevice { info };
        let retrieved = device.info();
        assert_eq!(retrieved.id, "test-device");
        assert_eq!(retrieved.model, JetsonModel::OrinNano4GB);
    }

    #[tokio::test]
    async fn test_discover_usb() {
        let device = JetsonDevice::discover_usb().await.unwrap();
        assert_eq!(device.id(), "jetson-usb");
        assert_eq!(device.info().connection, ConnectionMethod::Usb);
    }

    #[tokio::test]
    async fn test_discover_mdns() {
        let devices = JetsonDevice::discover_mdns().await.unwrap();
        // Placeholder returns empty vec
        assert!(devices.is_empty());
    }

    #[tokio::test]
    async fn test_discover_all() {
        let devices = JetsonDevice::discover_all().await.unwrap();
        // Should have at least the USB device
        assert!(!devices.is_empty());
    }

    #[tokio::test]
    async fn test_connect() {
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 55, 1));
        let device = JetsonDevice::connect(ip).await.unwrap();
        assert_eq!(device.id(), "jetson-192.168.55.1");
        if let ConnectionMethod::Ethernet(addr) = device.info().connection {
            assert_eq!(addr, ip);
        } else {
            panic!("Expected Ethernet connection");
        }
    }

    #[tokio::test]
    async fn test_available_memory() {
        let info = DeviceInfo {
            id: "test".to_string(),
            model: JetsonModel::OrinNano8GB,
            connection: ConnectionMethod::Usb,
            jetpack_version: None,
            hostname: None,
        };
        let device = JetsonDevice { info };
        let mem = device.available_memory_mb().await.unwrap();
        assert_eq!(mem, 4096); // Half of 8192
    }

    #[tokio::test]
    async fn test_exec_not_implemented() {
        let info = DeviceInfo {
            id: "test".to_string(),
            model: JetsonModel::Unknown,
            connection: ConnectionMethod::Usb,
            jetpack_version: None,
            hostname: None,
        };
        let device = JetsonDevice { info };
        let result = device.exec("ls -la").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_compute_hint_all_models() {
        for model in [
            JetsonModel::OrinNano4GB,
            JetsonModel::OrinNano8GB,
            JetsonModel::OrinNX8GB,
            JetsonModel::OrinNX16GB,
            JetsonModel::AgxOrin32GB,
            JetsonModel::AgxOrin64GB,
            JetsonModel::Unknown,
        ] {
            let info = DeviceInfo {
                id: "test".to_string(),
                model,
                connection: ConnectionMethod::Usb,
                jetpack_version: None,
                hostname: None,
            };
            let device = JetsonDevice { info };
            let hint = device.compute_hint();
            assert!(hint.prefer_neon);
            assert_eq!(hint.memory_budget_mb, model.memory_mb() / 2);
        }
    }
}
