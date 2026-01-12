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
    fn test_swap_config_default_ml() {
        let config = SwapConfig::default_ml();
        assert_eq!(config.size_gb, 16);
        assert_eq!(config.swappiness, 10);
        assert_eq!(config.path, PathBuf::from("/mnt/nvme/swapfile"));
    }

    #[test]
    fn test_swap_config_default() {
        let config = SwapConfig::default();
        assert_eq!(config.size_gb, 16);
        assert_eq!(config.swappiness, 10);
    }

    #[test]
    fn test_swap_config_clone() {
        let config = SwapConfig {
            path: PathBuf::from("/custom/swap"),
            size_gb: 32,
            swappiness: 5,
        };
        let cloned = config.clone();
        assert_eq!(cloned.size_gb, 32);
        assert_eq!(cloned.swappiness, 5);
        assert_eq!(cloned.path, PathBuf::from("/custom/swap"));
    }

    #[test]
    fn test_storage_layout_default() {
        let layout = StorageLayout::default();
        assert_eq!(layout.nvme_mount, PathBuf::from("/mnt/nvme"));
        assert_eq!(layout.models_dir, PathBuf::from("/mnt/nvme/models"));
        assert_eq!(layout.data_dir, PathBuf::from("/mnt/nvme/data"));
        assert_eq!(layout.cache_dir, PathBuf::from("/mnt/nvme/cache"));
        assert_eq!(layout.docker_dir, Some(PathBuf::from("/mnt/nvme/docker")));
    }

    #[test]
    fn test_storage_layout_default_layout() {
        let layout = StorageLayout::default_layout();
        assert_eq!(layout.nvme_mount, PathBuf::from("/mnt/nvme"));
        assert_eq!(layout.models_dir, PathBuf::from("/mnt/nvme/models"));
    }

    #[test]
    fn test_storage_layout_clone() {
        let layout = StorageLayout::default();
        let cloned = layout.clone();
        assert_eq!(cloned.nvme_mount, layout.nvme_mount);
        assert_eq!(cloned.models_dir, layout.models_dir);
    }

    #[test]
    fn test_nvme_device_detect() {
        let result = NvmeDevice::detect();
        assert!(result.is_ok());
        // Placeholder returns None
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_nvme_device_capacity_gb() {
        let nvme = NvmeDevice {
            device_path: PathBuf::from("/dev/nvme0n1"),
            mount_point: PathBuf::from("/mnt/nvme"),
            capacity_bytes: 512 * 1024 * 1024 * 1024, // 512GB
            available_bytes: 256 * 1024 * 1024 * 1024,
        };
        assert_eq!(nvme.capacity_gb(), 512);
    }

    #[test]
    fn test_nvme_device_available_gb() {
        let nvme = NvmeDevice {
            device_path: PathBuf::from("/dev/nvme0n1"),
            mount_point: PathBuf::from("/mnt/nvme"),
            capacity_bytes: 512 * 1024 * 1024 * 1024,
            available_bytes: 256 * 1024 * 1024 * 1024, // 256GB free
        };
        assert_eq!(nvme.available_gb(), 256);
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

    #[test]
    fn test_nvme_utilization_empty() {
        let nvme = NvmeDevice {
            device_path: PathBuf::from("/dev/nvme0n1"),
            mount_point: PathBuf::from("/mnt/nvme"),
            capacity_bytes: 0,
            available_bytes: 0,
        };
        assert_eq!(nvme.utilization_percent(), 0.0);
    }

    #[test]
    fn test_nvme_utilization_full() {
        let nvme = NvmeDevice {
            device_path: PathBuf::from("/dev/nvme0n1"),
            mount_point: PathBuf::from("/mnt/nvme"),
            capacity_bytes: 100 * 1024 * 1024 * 1024,
            available_bytes: 0,
        };
        assert!((nvme.utilization_percent() - 100.0).abs() < 0.1);
    }
}
