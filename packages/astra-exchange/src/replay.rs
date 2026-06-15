use crate::runtime::ExchangeRuntime;
use crate::state::ExchangeStateHash;
use astra_core::journal::EventJournal;
use astra_stream::replay::collect_journal_files;
use std::path::Path;

pub struct FullReplayEngine {
    pub runtime: ExchangeRuntime,
}

impl FullReplayEngine {
    pub fn new(runtime: ExchangeRuntime) -> Self {
        Self { runtime }
    }

    pub fn replay_directory(&mut self, journal_dir: &Path) -> Result<ExchangeStateHash, String> {
        let mut files = collect_journal_files(journal_dir).map_err(|e| e.to_string())?;
        files.sort();

        for file in files {
            let iter = EventJournal::iter_path(&file).map_err(|e| e.to_string())?;
            for event_res in iter {
                let event = event_res.map_err(|e| e.to_string())?;
                let mut children = self.runtime.apply_event(&event)?;
                while !children.is_empty() {
                    let mut next_gen = Vec::new();
                    for child in children {
                        let mut gc = self.runtime.apply_event(&child)?;
                        next_gen.append(&mut gc);
                    }
                    children = next_gen;
                }
            }
        }
        Ok(self.runtime.generate_global_hash())
    }
}
