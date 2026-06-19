
use blake3::Hasher;
use tokio::sync::RwLock;

#[derive(Clone, Debug)]
pub struct AuditEntry {
    pub timestamp_ns: u64,
    pub api_key_hash: [u8; 32],
    pub endpoint: String,
    pub method: String,
    pub request_body_hash: [u8; 32],
    pub response_status: u16,
    pub processing_time_ms: u64,
    pub chain_hash: [u8; 32],
}

pub struct AuditTrail {
    entries: RwLock<Vec<AuditEntry>>,
    prev_hash: RwLock<[u8; 32]>,
}

impl AuditTrail {
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(Vec::new()),
            prev_hash: RwLock::new([0u8; 32]), // Genesis hash
        }
    }

    pub async fn append(&self, mut entry: AuditEntry) {
        let mut prev = self.prev_hash.write().await;
        
        let mut hasher = Hasher::new();
        hasher.update(&*prev);
        hasher.update(&entry.timestamp_ns.to_le_bytes());
        hasher.update(&entry.api_key_hash);
        hasher.update(entry.endpoint.as_bytes());
        hasher.update(entry.method.as_bytes());
        hasher.update(&entry.request_body_hash);
        hasher.update(&entry.response_status.to_le_bytes());
        hasher.update(&entry.processing_time_ms.to_le_bytes());
        
        let new_hash = *hasher.finalize().as_bytes();
        entry.chain_hash = new_hash;
        *prev = new_hash;
        
        let mut entries = self.entries.write().await;
        entries.push(entry);
    }

    pub async fn verify_chain(&self) -> bool {
        let entries = self.entries.read().await;
        let mut current_hash = [0u8; 32];
        
        for entry in entries.iter() {
            let mut hasher = Hasher::new();
            hasher.update(&current_hash);
            hasher.update(&entry.timestamp_ns.to_le_bytes());
            hasher.update(&entry.api_key_hash);
            hasher.update(entry.endpoint.as_bytes());
            hasher.update(entry.method.as_bytes());
            hasher.update(&entry.request_body_hash);
            hasher.update(&entry.response_status.to_le_bytes());
            hasher.update(&entry.processing_time_ms.to_le_bytes());
            
            let expected_hash = *hasher.finalize().as_bytes();
            if expected_hash != entry.chain_hash {
                return false;
            }
            current_hash = expected_hash;
        }
        
        let prev = self.prev_hash.read().await;
        *prev == current_hash
    }

    pub async fn current_hash(&self) -> [u8; 32] {
        *self.prev_hash.read().await
    }
    
    pub async fn tamper_with_entry(&self, index: usize) {
        let mut entries = self.entries.write().await;
        if let Some(entry) = entries.get_mut(index) {
            entry.response_status = 500;
        }
    }
    
    pub async fn count(&self) -> usize {
        self.entries.read().await.len()
    }
}

impl Default for AuditTrail {
    fn default() -> Self {
        Self::new()
    }
}
