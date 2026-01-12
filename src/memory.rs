//! Memory management for constrained Jetson devices.
//!
//! Provides budget-aware allocation, memory tracking, and OOM prevention.

use crate::{Error, Result};
use std::sync::atomic::{AtomicU64, Ordering};

/// Memory budget enforcer - Poka-Yoke pattern.
///
/// Prevents allocation that would exceed the configured budget.
#[derive(Debug)]
pub struct MemoryBudget {
    /// Total device memory in MB
    total_mb: u64,
    /// Reserved for system in MB
    reserved_mb: u64,
    /// Currently allocated in MB
    allocated: AtomicU64,
}

impl MemoryBudget {
    /// Create a new memory budget.
    ///
    /// # Arguments
    ///
    /// * `total_mb` - Total device memory in MB
    /// * `reserved_mb` - Memory reserved for system (default: 2048 MB)
    #[must_use]
    pub fn new(total_mb: u64, reserved_mb: u64) -> Self {
        Self {
            total_mb,
            reserved_mb,
            allocated: AtomicU64::new(0),
        }
    }

    /// Create budget for Jetson Orin Nano 8GB.
    #[must_use]
    pub fn orin_nano_8gb() -> Self {
        Self::new(8192, 2048)
    }

    /// Create budget for Jetson Orin Nano 4GB.
    #[must_use]
    pub fn orin_nano_4gb() -> Self {
        Self::new(4096, 1024)
    }

    /// Get available memory in MB.
    #[must_use]
    pub fn available_mb(&self) -> u64 {
        let allocated = self.allocated.load(Ordering::Acquire);
        self.total_mb.saturating_sub(self.reserved_mb + allocated)
    }

    /// Get allocated memory in MB.
    #[must_use]
    pub fn allocated_mb(&self) -> u64 {
        self.allocated.load(Ordering::Acquire)
    }

    /// Get usable memory (total - reserved) in MB.
    #[must_use]
    pub fn usable_mb(&self) -> u64 {
        self.total_mb.saturating_sub(self.reserved_mb)
    }

    /// Try to allocate memory.
    ///
    /// # Errors
    ///
    /// Returns `Error::InsufficientMemory` if allocation would exceed budget.
    pub fn try_allocate(&self, size_mb: u64) -> Result<MemoryGuard<'_>> {
        let available = self.available_mb();
        if size_mb > available {
            return Err(Error::InsufficientMemory {
                requested_mb: size_mb,
                available_mb: available,
            });
        }
        self.allocated.fetch_add(size_mb, Ordering::Release);
        Ok(MemoryGuard {
            budget: self,
            size_mb,
        })
    }

    /// Check if allocation would fit.
    #[must_use]
    pub fn can_allocate(&self, size_mb: u64) -> bool {
        size_mb <= self.available_mb()
    }

    /// Get memory utilization as percentage.
    #[must_use]
    pub fn utilization_percent(&self) -> f32 {
        let usable = self.usable_mb() as f32;
        if usable == 0.0 {
            return 0.0;
        }
        (self.allocated_mb() as f32 / usable) * 100.0
    }
}

/// RAII guard for allocated memory.
///
/// Automatically releases memory when dropped.
#[derive(Debug)]
pub struct MemoryGuard<'a> {
    budget: &'a MemoryBudget,
    size_mb: u64,
}

impl Drop for MemoryGuard<'_> {
    fn drop(&mut self) {
        self.budget
            .allocated
            .fetch_sub(self.size_mb, Ordering::Release);
    }
}

impl MemoryGuard<'_> {
    /// Get the size of this allocation in MB.
    #[must_use]
    pub fn size_mb(&self) -> u64 {
        self.size_mb
    }
}

/// Estimate model memory requirements.
#[derive(Debug, Clone)]
pub struct ModelMemoryEstimate {
    /// Model weights in MB
    pub weights_mb: u64,
    /// Activation memory in MB
    pub activations_mb: u64,
    /// KV cache per token in KB
    pub kv_cache_per_token_kb: u64,
    /// Maximum context length
    pub max_context: u64,
}

impl ModelMemoryEstimate {
    /// Create estimate for a given parameter count and quantization.
    #[must_use]
    pub fn for_params(params_billions: f64, bits_per_param: u8, max_context: u64) -> Self {
        let weights_mb = ((params_billions * 1e9 * f64::from(bits_per_param)) / 8.0 / 1e6) as u64;
        let activations_mb = (params_billions * 100.0) as u64; // Rough estimate
        let kv_cache_per_token_kb = (params_billions * 2.0) as u64; // Rough estimate

        Self {
            weights_mb,
            activations_mb,
            kv_cache_per_token_kb,
            max_context,
        }
    }

    /// Total memory for given context length.
    #[must_use]
    pub fn total_mb(&self, context_length: u64) -> u64 {
        let kv_cache_mb = (self.kv_cache_per_token_kb * context_length) / 1024;
        self.weights_mb + self.activations_mb + kv_cache_mb
    }

    /// Check if model fits in budget.
    #[must_use]
    pub fn fits_in(&self, budget: &MemoryBudget, context_length: u64) -> bool {
        budget.can_allocate(self.total_mb(context_length))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_budget_available() {
        let budget = MemoryBudget::new(8192, 2048);
        assert_eq!(budget.available_mb(), 6144);
        assert_eq!(budget.usable_mb(), 6144);
    }

    #[test]
    fn test_memory_budget_allocation() {
        let budget = MemoryBudget::new(8192, 2048);
        let guard = budget.try_allocate(4000).unwrap();
        assert_eq!(budget.allocated_mb(), 4000);
        assert_eq!(budget.available_mb(), 2144);
        drop(guard);
        assert_eq!(budget.allocated_mb(), 0);
    }

    #[test]
    fn test_memory_budget_overflow() {
        let budget = MemoryBudget::new(8192, 2048);
        let result = budget.try_allocate(7000);
        assert!(result.is_err());
    }

    #[test]
    fn test_model_estimate() {
        // 7B model at 4-bit
        let estimate = ModelMemoryEstimate::for_params(7.0, 4, 2048);
        assert!(estimate.weights_mb > 3000); // ~3.5GB for 7B at 4-bit
    }

    #[test]
    fn test_utilization() {
        let budget = MemoryBudget::new(8192, 2048);
        assert_eq!(budget.utilization_percent(), 0.0);
        let _guard = budget.try_allocate(3072).unwrap();
        assert!((budget.utilization_percent() - 50.0).abs() < 0.1);
    }
}
