use blake3::Hasher;

pub trait DeterministicState {
    fn state_hash(&self) -> [u8; 32];
}

pub fn hash_bytes(data: &[u8]) -> [u8; 32] {
    let mut hasher = Hasher::new();
    hasher.update(data);
    hasher.finalize().into()
}

pub fn hash_to_hex(hash: &[u8; 32]) -> String {
    hex::encode(hash)
}

pub fn verify_hash_equality(expected: &[u8; 32], actual: &[u8; 32]) -> bool {
    expected == actual
}
