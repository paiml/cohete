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
    info: DeviceInfo,
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

    #[test]
    fn test_connection_method_default() {
        assert_eq!(ConnectionMethod::default(), ConnectionMethod::Usb);
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
    }
}
