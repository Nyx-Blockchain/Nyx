// src/tip_selection.rs

//! Tip selection algorithm for choosing parent transactions.
//!
//! Implements the weighted random walk algorithm from the Nyx whitepaper:
//! - Starts from genesis or latest snapshot
//! - Uses exponential weighting based on confirmation scores
//! - Prevents double-spending attacks by favoring high-weight paths

use crate::errors::{NyxError, Result};
use crate::types::Hash;
use crate::dag::DagProcessor;
use crate::TIP_SELECTION_ALPHA;
use rand::Rng;

/// Tip selector implementing the weighted random walk algorithm
pub struct TipSelector {
    /// Reference to the DAG processor
    dag: DagProcessor,

    /// Alpha parameter controlling randomness (default: 0.5)
    alpha: f64,
}

impl TipSelector {
    /// Creates a new tip selector
    ///
    /// # Arguments
    /// * `dag` - The DAG processor to select tips from
    pub fn new(dag: DagProcessor) -> Self {
        Self {
            dag,
            alpha: TIP_SELECTION_ALPHA,
        }
    }

    /// Creates a new tip selector with custom alpha parameter
    ///
    /// # Arguments
    /// * `dag` - The DAG processor
    /// * `alpha` - Controls randomness (0.0 = fully random, 1.0 = always pick highest score)
    pub fn with_alpha(dag: DagProcessor, alpha: f64) -> Self {
        Self { dag, alpha }
    }

    /// Selects two tips for a new transaction
    ///
    /// Uses weighted random walk as specified in whitepaper:
    /// P(C) = exp(Score(C) × α) / Σ exp(Score(Ci) × α)
    ///
    /// # Returns
    /// Two distinct transaction hashes to use as parents
    pub fn select_tips(&self) -> Result<[Hash; 2]> {
        let tips = self.dag.get_tips()?;

        if tips.is_empty() {
            return Err(NyxError::TipSelectionError(
                "No tips available for selection".to_string()
            ));
        }

        if tips.len() == 1 {
            // Special case: only one tip available, use it twice
            return Ok([tips[0], tips[0]]);
        }

        // Select first tip
        let tip1 = self.select_single_tip(&tips)?;

        // Select second tip (must be different from first)
        let mut tip2 = self.select_single_tip(&tips)?;
        let mut attempts = 0;
        while tip2 == tip1 && attempts < 10 {
            tip2 = self.select_single_tip(&tips)?;
            attempts += 1;
        }

        // If we couldn't find a different tip after 10 attempts, just pick another one
        if tip2 == tip1 && tips.len() > 1 {
            tip2 = tips.iter()
                .find(|&t| *t != tip1)
                .copied()
                .ok_or_else(|| NyxError::TipSelectionError(
                    "Could not select distinct tips".to_string()
                ))?;
        }

        Ok([tip1, tip2])
    }

    /// Selects a single tip using weighted random selection
    fn select_single_tip(&self, tips: &[Hash]) -> Result<Hash> {
        if tips.is_empty() {
            return Err(NyxError::TipSelectionError(
                "No tips available".to_string()
            ));
        }

        if tips.len() == 1 {
            return Ok(tips[0]);
        }

        // Calculate weights for each tip
        let mut weights = Vec::new();
        let mut total_weight = 0.0;

        for tip_hash in tips {
            let score = self.dag.get_score(tip_hash)?;
            let weight = (score * self.alpha).exp();
            weights.push(weight);
            total_weight += weight;
        }

        // Normalize weights to probabilities
        let probabilities: Vec<f64> = weights.iter()
            .map(|w| w / total_weight)
            .collect();

        // Select a tip based on weighted probability
        let mut rng = rand::thread_rng();
        let random_value: f64 = rng.gen();

        let mut cumulative = 0.0;
        for (i, prob) in probabilities.iter().enumerate() {
            cumulative += prob;
            if random_value <= cumulative {
                return Ok(tips[i]);
            }
        }

        // Fallback: return last tip (should rarely happen due to floating point)
        Ok(tips[tips.len() - 1])
    }

    /// Selects tips with a preference for specific characteristics
    ///
    /// This can be used to implement different selection strategies:
    /// - Prefer recent tips
    /// - Prefer tips with lower scores (help weak transactions)
    /// - Geographic preferences, etc.
    pub fn select_tips_with_preference<F>(&self, prefer: F) -> Result<[Hash; 2]>
    where
        F: Fn(&Hash) -> f64,
    {
        let tips = self.dag.get_tips()?;

        if tips.is_empty() {
            return Err(NyxError::TipSelectionError(
                "No tips available".to_string()
            ));
        }

        if tips.len() == 1 {
            return Ok([tips[0], tips[0]]);
        }

        // Calculate weights with preference function
        let mut weights = Vec::new();
        let mut total_weight = 0.0;

        for tip_hash in &tips {
            let score = self.dag.get_score(tip_hash)?;
            let preference = prefer(tip_hash);
            let weight = (score * self.alpha).exp() * preference;
            weights.push(weight);
            total_weight += weight;
        }

        // Select two tips
        let tip1 = self.select_from_weights(&tips, &weights, total_weight)?;
        let mut tip2 = self.select_from_weights(&tips, &weights, total_weight)?;

        // Ensure distinct tips
        let mut attempts = 0;
        while tip2 == tip1 && attempts < 10 {
            tip2 = self.select_from_weights(&tips, &weights, total_weight)?;
            attempts += 1;
        }

        Ok([tip1, tip2])
    }

    /// Helper function to select a tip from weighted distribution
    fn select_from_weights(
        &self,
        tips: &[Hash],
        weights: &[f64],
        total_weight: f64,
    ) -> Result<Hash> {
        let mut rng = rand::thread_rng();
        let random_value: f64 = rng.gen_range(0.0..total_weight);

        let mut cumulative = 0.0;
        for (i, weight) in weights.iter().enumerate() {
            cumulative += weight;
            if random_value <= cumulative {
                return Ok(tips[i]);
            }
        }

        Ok(tips[tips.len() - 1])
    }

    /// Returns the current alpha parameter
    pub fn alpha(&self) -> f64 {
        self.alpha
    }

    /// Sets a new alpha parameter
    ///
    /// # Arguments
    /// * `alpha` - New alpha value (should be between 0.0 and 1.0)
    pub fn set_alpha(&mut self, alpha: f64) {
        self.alpha = alpha.clamp(0.0, 1.0);
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::MemoryStorage;

    #[test]
    fn test_tip_selector_creation() {
        let storage = MemoryStorage::new();
        let dag = DagProcessor::new(storage);
        let selector = TipSelector::new(dag);

        assert_eq!(selector.alpha(), TIP_SELECTION_ALPHA);
    }

    #[test]
    fn test_custom_alpha() {
        let storage = MemoryStorage::new();
        let dag = DagProcessor::new(storage);
        let mut selector = TipSelector::with_alpha(dag, 0.8);

        assert_eq!(selector.alpha(), 0.8);

        selector.set_alpha(0.3);
        assert_eq!(selector.alpha(), 0.3);
    }

    #[test]
    fn test_alpha_clamping() {
        let storage = MemoryStorage::new();
        let dag = DagProcessor::new(storage);
        let mut selector = TipSelector::new(dag);

        selector.set_alpha(1.5); // Above 1.0
        assert_eq!(selector.alpha(), 1.0);

        selector.set_alpha(-0.5); // Below 0.0
        assert_eq!(selector.alpha(), 0.0);
    }
}
