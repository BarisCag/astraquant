use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

pub struct DeterministicPrng {
    rng: ChaCha8Rng,
}

impl DeterministicPrng {
    pub fn new(
        prev_journal_hash: &[u8; 32],
        event_timestamp_ns: u64,
        event_type_discriminant: u8,
        event_nonce: u64,
    ) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(prev_journal_hash);
        hasher.update(&event_timestamp_ns.to_le_bytes());
        hasher.update(&[event_type_discriminant]);
        hasher.update(&event_nonce.to_le_bytes());
        let hash = hasher.finalize();
        
        let seed: [u8; 32] = *hash.as_bytes();
        let rng = ChaCha8Rng::from_seed(seed);
        
        Self { rng }
    }

    /// Derives a deterministic pseudo-random float between 0.0 and 1.0.
    pub fn next_f64(&mut self) -> f64 {
        self.rng.gen()
    }
}
