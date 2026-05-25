use astra_core::hashing::hash_to_hex;
use astra_core::journal::EventJournal;
use astra_core::kernel::AstraKernel;
use astra_core::replay::ReplayEngine;
use std::time::Instant;

pub struct AuditExplorer;

impl AuditExplorer {
    pub fn verify_timeline_isolation(journal_path: &str) -> Result<bool, String> {
        let limits = astra_core::risk::RiskLimits::new(
            astra_core::types::Money::new(100_000_000),
            astra_core::types::Quantity::new(1_000),
        );
        // Boot a completely isolated phantom kernel
        let mut phantom_kernel = AstraKernel::new(astra_core::runtime::StrategyRuntime::new(
            astra_core::exchange::ExchangeRuntime::new(limits),
        ));
        let journal = EventJournal::open(journal_path).map_err(|e| e.to_string())?;

        let start = Instant::now();
        let result = ReplayEngine::replay_journal(&journal, &mut phantom_kernel);
        let duration = start.elapsed();

        match result {
            Ok(res) => {
                println!(
                    "Audit successful: replayed to seq {} in {:?}. Final Hash: {}",
                    res.final_sequence_id,
                    duration,
                    hash_to_hex(&res.final_state_hash)
                );
                Ok(true)
            }
            Err(e) => {
                println!("Audit FAILED: {}", e);
                Ok(false)
            }
        }
    }
}
