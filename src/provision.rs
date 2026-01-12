//! Provisioning and setup for Jetson devices.
//!
//! Provides automated device setup, SSH configuration, and NVMe provisioning.

use crate::{storage::StorageLayout, Result};
use std::path::PathBuf;

/// Provision configuration.
#[derive(Debug, Clone)]
pub struct ProvisionConfig {
    /// NVMe configuration
    pub nvme: NvmeProvisionConfig,
    /// SSH configuration
    pub ssh: SshProvisionConfig,
    /// Packages to install
    pub packages: Vec<String>,
}

impl Default for ProvisionConfig {
    fn default() -> Self {
        Self {
            nvme: NvmeProvisionConfig::default(),
            ssh: SshProvisionConfig::default(),
            packages: vec![
                "nvtop".to_string(),
                "htop".to_string(),
                "tmux".to_string(),
            ],
        }
    }
}

/// NVMe provisioning configuration.
#[derive(Debug, Clone)]
pub struct NvmeProvisionConfig {
    /// Enable NVMe setup
    pub enabled: bool,
    /// Mount point
    pub mount_point: PathBuf,
    /// Swap size in GB
    pub swap_size_gb: u64,
}

impl Default for NvmeProvisionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mount_point: PathBuf::from("/mnt/nvme"),
            swap_size_gb: 16,
        }
    }
}

/// SSH provisioning configuration.
#[derive(Debug, Clone)]
pub struct SshProvisionConfig {
    /// Copy SSH key to device
    pub copy_id: bool,
    /// Host entry name for ~/.ssh/config
    pub config_host: String,
    /// Username
    pub username: String,
}

impl Default for SshProvisionConfig {
    fn default() -> Self {
        Self {
            copy_id: true,
            config_host: "jetson".to_string(),
            username: "nvidia".to_string(),
        }
    }
}

/// Interactive setup wizard.
#[derive(Debug)]
pub struct SetupWizard {
    config: ProvisionConfig,
}

impl SetupWizard {
    /// Create new setup wizard.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: ProvisionConfig::default(),
        }
    }

    /// Run interactive setup.
    ///
    /// # Errors
    ///
    /// Returns an error if setup fails.
    pub async fn run(&self) -> Result<ProvisionResult> {
        // Placeholder - would run interactive prompts
        Ok(ProvisionResult {
            ssh_configured: true,
            nvme_configured: self.config.nvme.enabled,
            packages_installed: self.config.packages.clone(),
            storage_layout: StorageLayout::default(),
        })
    }

    /// Set provision config.
    pub fn with_config(mut self, config: ProvisionConfig) -> Self {
        self.config = config;
        self
    }
}

impl Default for SetupWizard {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of provisioning operation.
#[derive(Debug)]
pub struct ProvisionResult {
    /// SSH configured successfully
    pub ssh_configured: bool,
    /// NVMe configured successfully
    pub nvme_configured: bool,
    /// Packages installed
    pub packages_installed: Vec<String>,
    /// Storage layout
    pub storage_layout: StorageLayout,
}

/// SSH config entry.
#[derive(Debug, Clone)]
pub struct SshConfigEntry {
    /// Host alias
    pub host: String,
    /// Hostname or IP
    pub hostname: String,
    /// Username
    pub user: String,
    /// Identity file (optional)
    pub identity_file: Option<PathBuf>,
}

impl SshConfigEntry {
    /// Create entry for USB connection.
    #[must_use]
    pub fn usb(user: impl Into<String>) -> Self {
        Self {
            host: "jetson-usb".to_string(),
            hostname: "192.168.55.1".to_string(),
            user: user.into(),
            identity_file: None,
        }
    }

    /// Create entry for Ethernet connection.
    #[must_use]
    pub fn ethernet(ip: impl Into<String>, user: impl Into<String>) -> Self {
        Self {
            host: "jetson-eth".to_string(),
            hostname: ip.into(),
            user: user.into(),
            identity_file: None,
        }
    }

    /// Generate SSH config block.
    #[must_use]
    pub fn to_config_block(&self) -> String {
        let mut block = format!(
            "Host {}\n    HostName {}\n    User {}\n",
            self.host, self.hostname, self.user
        );
        if let Some(ref identity) = self.identity_file {
            block.push_str(&format!("    IdentityFile {}\n", identity.display()));
        }
        block
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provision_config_default() {
        let config = ProvisionConfig::default();
        assert!(config.nvme.enabled);
        assert_eq!(config.nvme.swap_size_gb, 16);
    }

    #[test]
    fn test_ssh_config_entry() {
        let entry = SshConfigEntry::usb("nvidia");
        assert_eq!(entry.hostname, "192.168.55.1");
        let block = entry.to_config_block();
        assert!(block.contains("Host jetson-usb"));
        assert!(block.contains("192.168.55.1"));
    }

    #[test]
    fn test_setup_wizard() {
        let wizard = SetupWizard::new();
        assert!(wizard.config.nvme.enabled);
    }
}
