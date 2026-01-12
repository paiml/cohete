//! Model quantization for Jetson memory constraints.
//!
//! Provides memory-aware quantization for edge deployment.

use crate::{memory::MemoryBudget, Result};

/// Quantization levels (compatible with llama.cpp).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuantLevel {
    /// 4-bit quantization (type 0)
    Q4_0,
    /// 4-bit quantization (type 1)
    Q4_1,
    /// 5-bit quantization (type 0)
    Q5_0,
    /// 5-bit quantization (type 1)
    Q5_1,
    /// 8-bit quantization
    Q8_0,
    /// 16-bit floating point
    F16,
    /// 32-bit floating point (not recommended for Jetson)
    F32,
}

impl QuantLevel {
    /// Bits per parameter.
    #[must_use]
    pub const fn bits_per_param(&self) -> u8 {
        match self {
            Self::Q4_0 | Self::Q4_1 => 4,
            Self::Q5_0 | Self::Q5_1 => 5,
            Self::Q8_0 => 8,
            Self::F16 => 16,
            Self::F32 => 32,
        }
    }

    /// Approximate perplexity increase percentage vs F16.
    #[must_use]
    pub const fn perplexity_delta_percent(&self) -> f32 {
        match self {
            Self::Q4_0 => 5.0,
            Self::Q4_1 => 4.0,
            Self::Q5_0 => 3.0,
            Self::Q5_1 => 2.5,
            Self::Q8_0 => 1.0,
            Self::F16 => 0.0,
            Self::F32 => 0.0,
        }
    }

    /// Memory reduction factor vs F16.
    #[must_use]
    pub const fn memory_factor(&self) -> f32 {
        match self {
            Self::Q4_0 | Self::Q4_1 => 0.25,
            Self::Q5_0 | Self::Q5_1 => 0.3125,
            Self::Q8_0 => 0.5,
            Self::F16 => 1.0,
            Self::F32 => 2.0,
        }
    }

    /// String representation.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Q4_0 => "q4_0",
            Self::Q4_1 => "q4_1",
            Self::Q5_0 => "q5_0",
            Self::Q5_1 => "q5_1",
            Self::Q8_0 => "q8_0",
            Self::F16 => "f16",
            Self::F32 => "f32",
        }
    }
}

impl std::fmt::Display for QuantLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Jetson-optimized quantizer.
#[derive(Debug)]
pub struct JetsonQuantizer {
    level: QuantLevel,
    target_memory_mb: Option<u64>,
}

impl JetsonQuantizer {
    /// Create quantizer with specified level.
    #[must_use]
    pub fn new(level: QuantLevel) -> Self {
        Self {
            level,
            target_memory_mb: None,
        }
    }

    /// Set target memory budget.
    #[must_use]
    pub fn with_target_memory_mb(mut self, budget: u64) -> Self {
        self.target_memory_mb = Some(budget);
        self
    }

    /// Get current quantization level.
    #[must_use]
    pub fn level(&self) -> QuantLevel {
        self.level
    }

    /// Select optimal quantization level for memory budget.
    #[must_use]
    pub fn select_for_budget(model_f16_size_mb: u64, budget: &MemoryBudget) -> QuantLevel {
        let available = budget.available_mb();

        // Try each level from highest quality to lowest
        for level in [
            QuantLevel::F16,
            QuantLevel::Q8_0,
            QuantLevel::Q5_1,
            QuantLevel::Q5_0,
            QuantLevel::Q4_1,
            QuantLevel::Q4_0,
        ] {
            let estimated_size = (model_f16_size_mb as f32 * level.memory_factor()) as u64;
            if estimated_size <= available {
                return level;
            }
        }

        // Default to most aggressive
        QuantLevel::Q4_0
    }

    /// Quantize model bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if quantization fails.
    pub fn quantize(&self, _model: &[u8]) -> Result<QuantResult> {
        // Placeholder - would use llama.cpp quantization
        Ok(QuantResult {
            level: self.level,
            original_size_mb: 14000, // 7B F16
            quantized_size_mb: (14000.0 * self.level.memory_factor()) as u64,
            estimated_perplexity_delta: self.level.perplexity_delta_percent(),
        })
    }
}

/// Result of quantization operation.
#[derive(Debug, Clone)]
pub struct QuantResult {
    /// Quantization level used
    pub level: QuantLevel,
    /// Original model size in MB
    pub original_size_mb: u64,
    /// Quantized model size in MB
    pub quantized_size_mb: u64,
    /// Estimated perplexity increase percentage
    pub estimated_perplexity_delta: f32,
}

impl QuantResult {
    /// Get compression ratio.
    #[must_use]
    pub fn compression_ratio(&self) -> f32 {
        if self.quantized_size_mb == 0 {
            return 0.0;
        }
        self.original_size_mb as f32 / self.quantized_size_mb as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quant_level_bits() {
        assert_eq!(QuantLevel::Q4_0.bits_per_param(), 4);
        assert_eq!(QuantLevel::Q8_0.bits_per_param(), 8);
        assert_eq!(QuantLevel::F16.bits_per_param(), 16);
    }

    #[test]
    fn test_quant_level_memory_factor() {
        assert_eq!(QuantLevel::Q4_0.memory_factor(), 0.25);
        assert_eq!(QuantLevel::Q8_0.memory_factor(), 0.5);
        assert_eq!(QuantLevel::F16.memory_factor(), 1.0);
    }

    #[test]
    fn test_quant_level_display() {
        assert_eq!(QuantLevel::Q4_0.to_string(), "q4_0");
        assert_eq!(QuantLevel::F16.to_string(), "f16");
    }

    #[test]
    fn test_select_for_budget() {
        let budget = MemoryBudget::orin_nano_8gb(); // 6144 MB available
        // 14GB F16 model: at Q5_1 = 14000 * 0.3125 = 4375 MB (fits)
        let level = JetsonQuantizer::select_for_budget(14000, &budget);
        assert_eq!(level, QuantLevel::Q5_1);

        // 20GB F16 model: Q4_1 = 20000 * 0.25 = 5000 MB (fits, higher quality than Q4_0)
        let level = JetsonQuantizer::select_for_budget(20000, &budget);
        assert_eq!(level, QuantLevel::Q4_1);
    }

    #[test]
    fn test_quantizer() {
        let quantizer = JetsonQuantizer::new(QuantLevel::Q4_0);
        let result = quantizer.quantize(&[]).unwrap();
        assert_eq!(result.level, QuantLevel::Q4_0);
        assert!(result.compression_ratio() > 1.0);
    }
}
