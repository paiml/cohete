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
        assert!(!config.packages.is_empty());
        assert!(config.packages.contains(&"nvtop".to_string()));
    }

    #[test]
    fn test_provision_config_clone() {
        let config = ProvisionConfig::default();
        let cloned = config.clone();
        assert_eq!(cloned.packages.len(), config.packages.len());
        assert_eq!(cloned.nvme.swap_size_gb, config.nvme.swap_size_gb);
    }

    #[test]
    fn test_nvme_provision_config_default() {
        let config = NvmeProvisionConfig::default();
        assert!(config.enabled);
        assert_eq!(config.mount_point, PathBuf::from("/mnt/nvme"));
        assert_eq!(config.swap_size_gb, 16);
    }

    #[test]
    fn test_nvme_provision_config_clone() {
        let config = NvmeProvisionConfig {
            enabled: false,
            mount_point: PathBuf::from("/custom"),
            swap_size_gb: 32,
        };
        let cloned = config.clone();
        assert!(!cloned.enabled);
        assert_eq!(cloned.swap_size_gb, 32);
    }

    #[test]
    fn test_ssh_provision_config_default() {
        let config = SshProvisionConfig::default();
        assert!(config.copy_id);
        assert_eq!(config.config_host, "jetson");
        assert_eq!(config.username, "nvidia");
    }

    #[test]
    fn test_ssh_provision_config_clone() {
        let config = SshProvisionConfig {
            copy_id: false,
            config_host: "custom-host".to_string(),
            username: "admin".to_string(),
        };
        let cloned = config.clone();
        assert!(!cloned.copy_id);
        assert_eq!(cloned.config_host, "custom-host");
    }

    #[test]
    fn test_ssh_config_entry_usb() {
        let entry = SshConfigEntry::usb("nvidia");
        assert_eq!(entry.host, "jetson-usb");
        assert_eq!(entry.hostname, "192.168.55.1");
        assert_eq!(entry.user, "nvidia");
        assert!(entry.identity_file.is_none());
    }

    #[test]
    fn test_ssh_config_entry_ethernet() {
        let entry = SshConfigEntry::ethernet("192.168.1.100", "admin");
        assert_eq!(entry.host, "jetson-eth");
        assert_eq!(entry.hostname, "192.168.1.100");
        assert_eq!(entry.user, "admin");
    }

    #[test]
    fn test_ssh_config_entry_to_config_block() {
        let entry = SshConfigEntry::usb("nvidia");
        let block = entry.to_config_block();
        assert!(block.contains("Host jetson-usb"));
        assert!(block.contains("HostName 192.168.55.1"));
        assert!(block.contains("User nvidia"));
    }

    #[test]
    fn test_ssh_config_entry_with_identity_file() {
        let entry = SshConfigEntry {
            host: "jetson".to_string(),
            hostname: "192.168.55.1".to_string(),
            user: "nvidia".to_string(),
            identity_file: Some(PathBuf::from("/home/user/.ssh/jetson_key")),
        };
        let block = entry.to_config_block();
        assert!(block.contains("IdentityFile /home/user/.ssh/jetson_key"));
    }

    #[test]
    fn test_ssh_config_entry_clone() {
        let entry = SshConfigEntry::usb("test");
        let cloned = entry.clone();
        assert_eq!(cloned.host, entry.host);
        assert_eq!(cloned.hostname, entry.hostname);
    }

    #[test]
    fn test_setup_wizard_new() {
        let wizard = SetupWizard::new();
        assert!(wizard.config.nvme.enabled);
    }

    #[test]
    fn test_setup_wizard_default() {
        let wizard = SetupWizard::default();
        assert!(wizard.config.nvme.enabled);
    }

    #[test]
    fn test_setup_wizard_with_config() {
        let config = ProvisionConfig {
            nvme: NvmeProvisionConfig {
                enabled: false,
                mount_point: PathBuf::from("/custom"),
                swap_size_gb: 8,
            },
            ssh: SshProvisionConfig::default(),
            packages: vec![],
        };
        let wizard = SetupWizard::new().with_config(config);
        assert!(!wizard.config.nvme.enabled);
        assert_eq!(wizard.config.nvme.swap_size_gb, 8);
    }

    #[tokio::test]
    async fn test_setup_wizard_run() {
        let wizard = SetupWizard::new();
        let result = wizard.run().await.unwrap();
        assert!(result.ssh_configured);
        assert!(result.nvme_configured);
        assert!(!result.packages_installed.is_empty());
    }

    #[tokio::test]
    async fn test_setup_wizard_run_no_nvme() {
        let config = ProvisionConfig {
            nvme: NvmeProvisionConfig {
                enabled: false,
                ..Default::default()
            },
            ..Default::default()
        };
        let wizard = SetupWizard::new().with_config(config);
        let result = wizard.run().await.unwrap();
        assert!(!result.nvme_configured);
    }

    #[test]
    fn test_provision_result_fields() {
        let result = ProvisionResult {
            ssh_configured: true,
            nvme_configured: true,
            packages_installed: vec!["htop".to_string()],
            storage_layout: StorageLayout::default(),
        };
        assert!(result.ssh_configured);
        assert!(result.nvme_configured);
        assert_eq!(result.packages_installed.len(), 1);
    }
}
