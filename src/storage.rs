//! Storage management for Jetson devices.
//!
//! Provides NVMe SSD management, swap configuration, and model storage.

use crate::Result;
use std::path::PathBuf;

/// NVMe device handle.
#[derive(Debug)]
pub struct NvmeDevice {
    /// Device path (e.g., /dev/nvme0n1)
    pub device_path: PathBuf,
    /// Mount point
    pub mount_point: PathBuf,
    /// Total capacity in bytes
    pub capacity_bytes: u64,
    /// Available space in bytes
    pub available_bytes: u64,
}

impl NvmeDevice {
    /// Detect NVMe device.
    ///
    /// # Errors
    ///
    /// Returns an error if no NVMe device is found.
    pub fn detect() -> Result<Option<Self>> {
        // Would check /dev/nvme0n1
        Ok(None)
    }

    /// Get capacity in GB.
    #[must_use]
    pub fn capacity_gb(&self) -> u64 {
        self.capacity_bytes / (1024 * 1024 * 1024)
    }

    /// Get available space in GB.
    #[must_use]
    pub fn available_gb(&self) -> u64 {
        self.available_bytes / (1024 * 1024 * 1024)
    }

    /// Get utilization percentage.
    #[must_use]
    pub fn utilization_percent(&self) -> f32 {
        if self.capacity_bytes == 0 {
            return 0.0;
        }
        let used = self.capacity_bytes - self.available_bytes;
        (used as f32 / self.capacity_bytes as f32) * 100.0
    }
}

/// Swap file configuration.
#[derive(Debug, Clone)]
pub struct SwapConfig {
    /// Swap file path
    pub path: PathBuf,
    /// Swap size in GB
    pub size_gb: u64,
    /// Swappiness value (0-100)
    pub swappiness: u8,
}

impl SwapConfig {
    /// Default swap configuration for ML workloads.
    #[must_use]
    pub fn default_ml() -> Self {
        Self {
            path: PathBuf::from("/mnt/nvme/swapfile"),
            size_gb: 16,
            swappiness: 10, // Low swappiness for ML
        }
    }
}

impl Default for SwapConfig {
    fn default() -> Self {
        Self::default_ml()
    }
}

/// Storage layout configuration.
#[derive(Debug, Clone)]
pub struct StorageLayout {
    /// NVMe mount point
    pub nvme_mount: PathBuf,
    /// Models directory
    pub models_dir: PathBuf,
    /// Data directory
    pub data_dir: PathBuf,
    /// Cache directory
    pub cache_dir: PathBuf,
    /// Docker directory (optional)
    pub docker_dir: Option<PathBuf>,
}

impl StorageLayout {
    /// Default storage layout.
    #[must_use]
    pub fn default_layout() -> Self {
        let nvme_mount = PathBuf::from("/mnt/nvme");
        Self {
            models_dir: nvme_mount.join("models"),
            data_dir: nvme_mount.join("data"),
            cache_dir: nvme_mount.join("cache"),
            docker_dir: Some(nvme_mount.join("docker")),
            nvme_mount,
        }
    }
}

impl Default for StorageLayout {
    fn default() -> Self {
        Self::default_layout()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swap_config_default() {
        let config = SwapConfig::default_ml();
        assert_eq!(config.size_gb, 16);
        assert_eq!(config.swappiness, 10);
    }

    #[test]
    fn test_storage_layout() {
        let layout = StorageLayout::default_layout();
        assert_eq!(layout.nvme_mount, PathBuf::from("/mnt/nvme"));
        assert_eq!(layout.models_dir, PathBuf::from("/mnt/nvme/models"));
    }

    #[test]
    fn test_nvme_utilization() {
        let nvme = NvmeDevice {
            device_path: PathBuf::from("/dev/nvme0n1"),
            mount_point: PathBuf::from("/mnt/nvme"),
            capacity_bytes: 500 * 1024 * 1024 * 1024, // 500GB
            available_bytes: 250 * 1024 * 1024 * 1024, // 250GB free
        };
        assert!((nvme.utilization_percent() - 50.0).abs() < 0.1);
    }
}
