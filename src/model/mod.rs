//! Transaction shape model: extraction and aggregation.

use serde::{Deserialize, Serialize};

/// Single transaction shape (metadata only; no addresses or values).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TxShape {
    /// Number of transparent inputs.
    pub n_vin: u32,
    /// Number of transparent outputs.
    pub n_vout: u32,
    /// Number of Sprout joinSplits (0 for v4+ only).
    pub n_joinsplit: u32,
    /// Number of Sapling shielded spends.
    pub n_sapling_spend: u32,
    /// Number of Sapling shielded outputs.
    pub n_sapling_output: u32,
    /// Number of Orchard actions (v5+).
    pub n_orchard_action: u32,
    /// Size bucket (0..=5). See util::size_bucket.
    pub size_bucket: u8,
    /// Transaction version (1–6).
    pub version: u32,
}

impl TxShape {
    /// Whether the transaction has any transparent component.
    pub fn has_transparent(&self) -> bool {
        self.n_vin > 0 || self.n_vout > 0
    }

    /// Whether the transaction has any shielded component.
    pub fn has_shielded(&self) -> bool {
        self.n_joinsplit > 0
            || self.n_sapling_spend > 0
            || self.n_sapling_output > 0
            || self.n_orchard_action > 0
    }
}

/// Aggregate shape statistics for a block or range.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ShapeStats {
    /// Total transactions counted.
    pub n_txs: u64,
    /// Histogram: count per (n_vin, n_vout) bucket; key = "vin_vout" e.g. "1_2".
    pub vin_vout_hist: std::collections::HashMap<String, u64>,
    /// Histogram: count per size_bucket (0..=5).
    pub size_bucket_hist: [u64; 6],
    /// Histogram: count per version.
    pub version_hist: std::collections::HashMap<u32, u64>,
    /// Count of txs with transparent component.
    pub with_transparent: u64,
    /// Count of txs with shielded component.
    pub with_shielded: u64,
    /// Shannon entropy of size_bucket distribution.
    pub size_entropy: f64,
}

impl ShapeStats {
    pub fn from_shapes(shapes: &[TxShape]) -> Self {
        let n_txs = shapes.len() as u64;
        let mut vin_vout_hist = std::collections::HashMap::new();
        let mut size_bucket_hist = [0u64; 6];
        let mut version_hist = std::collections::HashMap::new();
        let mut with_transparent = 0u64;
        let mut with_shielded = 0u64;

        for s in shapes {
            let key = format!("{}_{}", s.n_vin, s.n_vout);
            *vin_vout_hist.entry(key).or_insert(0) += 1;
            if s.size_bucket < 6 {
                size_bucket_hist[s.size_bucket as usize] += 1;
            }
            *version_hist.entry(s.version).or_insert(0) += 1;
            if s.has_transparent() {
                with_transparent += 1;
            }
            if s.has_shielded() {
                with_shielded += 1;
            }
        }

        let size_entropy = crate::util::entropy(&size_bucket_hist);

        ShapeStats {
            n_txs,
            vin_vout_hist,
            size_bucket_hist,
            version_hist,
            with_transparent,
            with_shielded,
            size_entropy,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::size_bucket;

    #[test]
    fn test_shape_has_transparent_shielded() {
        let s = TxShape {
            n_vin: 1,
            n_vout: 0,
            n_joinsplit: 0,
            n_sapling_spend: 0,
            n_sapling_output: 0,
            n_orchard_action: 0,
            size_bucket: 0,
            version: 4,
        };
        assert!(s.has_transparent());
        assert!(!s.has_shielded());
    }

    #[test]
    fn test_shape_stats_from_shapes() {
        let shapes = vec![
            TxShape {
                n_vin: 1,
                n_vout: 2,
                n_joinsplit: 0,
                n_sapling_spend: 0,
                n_sapling_output: 0,
                n_orchard_action: 0,
                size_bucket: size_bucket(300),
                version: 4,
            },
            TxShape {
                n_vin: 1,
                n_vout: 2,
                n_joinsplit: 0,
                n_sapling_spend: 0,
                n_sapling_output: 0,
                n_orchard_action: 0,
                size_bucket: size_bucket(300),
                version: 4,
            },
        ];
        let stats = ShapeStats::from_shapes(&shapes);
        assert_eq!(stats.n_txs, 2);
        assert_eq!(stats.vin_vout_hist.get("1_2"), Some(&2));
        assert_eq!(stats.with_transparent, 2);
    }
}
