use crate::journal::EventJournal;
use crate::kernel::AstraKernel;
use crate::replay::EventReducer;
use std::io;

pub struct ExecutionGateway {
    pub kernel: AstraKernel,
    pub journal: EventJournal,
}

impl ExecutionGateway {
    pub fn new(kernel: AstraKernel, journal: EventJournal) -> Self {
        Self { kernel, journal }
    }

    pub fn process_external_payload(&mut self, payload: Vec<u8>) -> io::Result<()> {
        let event = self.journal.commit(
            0,
            crate::events::EventType::MarketTick,
            payload,
            crate::events::PayloadMetadata::new(crate::events::PayloadEncoding::Bincode, 1),
        )?;
        self.kernel
            .apply(&event)
            .map_err(|e| io::Error::other(e.to_string()))?;
        Ok(())
    }

    pub fn ingest_raw_event(
        &mut self,
        timestamp_ns: u64,
        event_type: crate::events::EventType,
        payload: Vec<u8>,
    ) -> io::Result<()> {
        let event = self.journal.commit(
            timestamp_ns,
            event_type,
            payload,
            crate::events::PayloadMetadata::new(crate::events::PayloadEncoding::RawBytes, 1),
        )?;
        self.kernel
            .apply(&event)
            .map_err(|e| io::Error::other(e.to_string()))?;
        Ok(())
    }
}
