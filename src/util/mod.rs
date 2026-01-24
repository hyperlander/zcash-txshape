//! Shared utilities.

/// Size bucket index for transaction size (bytes). Buckets: 0-256, 257-512, 513-1024, 1025-2048, 2049-4096, 4097+.
pub fn size_bucket(size_bytes: u32) -> u8 {
    match size_bytes {
        0..=256 => 0,
        257..=512 => 1,
        513..=1024 => 2,
        1025..=2048 => 3,
        2049..=4096 => 4,
        _ => 5,
    }
}

/// Shannon entropy of a distribution (counts per bucket). Returns 0 if total is 0.
pub fn entropy(counts: &[u64]) -> f64 {
    let total: u64 = counts.iter().sum();
    if total == 0 {
        return 0.0;
    }
    let n = total as f64;
    -counts
        .iter()
        .filter(|&&c| c > 0)
        .map(|&c| {
            let p = c as f64 / n;
            p * p.log2()
        })
        .sum::<f64>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_bucket() {
        assert_eq!(size_bucket(0), 0);
        assert_eq!(size_bucket(256), 0);
        assert_eq!(size_bucket(257), 1);
        assert_eq!(size_bucket(512), 1);
        assert_eq!(size_bucket(1024), 2);
        assert_eq!(size_bucket(4096), 4);
        assert_eq!(size_bucket(5000), 5);
    }

    #[test]
    fn test_entropy() {
        assert_eq!(entropy(&[]), 0.0);
        assert_eq!(entropy(&[0, 0]), 0.0);
        assert!((entropy(&[1, 1]) - 1.0).abs() < 1e-10);
        assert!((entropy(&[1, 1, 1, 1]) - 2.0).abs() < 1e-10);
    }
}
