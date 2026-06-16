use crate::journal::EventJournal;

use std::io;
use std::time::{SystemTime, UNIX_EPOCH};

use std::collections::VecDeque;

pub struct ExecutionGateway {
    pub journal: EventJournal,
    queue: VecDeque<crate::events::AstraEvent>,
}

impl ExecutionGateway {
    pub fn new(journal: EventJournal) -> Self {
        Self {
            journal,
            queue: VecDeque::new(),
        }
    }

    /// Ingests an external payload, stamping it with the current wall-clock time.
    /// Wall-clock stamping is ONLY permitted here at the quarantine boundary.
    /// The kernel itself never reads wall-clock time.
    pub fn process_external_payload(&mut self, payload: Vec<u8>) -> io::Result<()> {
        let timestamp_ns = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let event = self.journal.commit(
            timestamp_ns,
            crate::events::EventType::MarketTick,
            payload,
            crate::events::PayloadMetadata::new(crate::events::PayloadEncoding::Bincode, 1),
        )?;
        self.queue.push_back(event);
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
        self.queue.push_back(event);
        Ok(())
    }

    pub fn next_event(&mut self) -> Option<crate::events::AstraEvent> {
        self.queue.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::journal::EventJournal;

    use crate::risk::create_default_risk_engine;

    use crate::types::{Money, Quantity};

    fn make_gateway(path: &std::path::Path) -> ExecutionGateway {
        let _limits = create_default_risk_engine(Money::new(1_000_000), Quantity::new(1_000));
        let journal = EventJournal::create(path, 1_700_000_000_000_000_000).unwrap();
        ExecutionGateway::new(journal)
    }

    /// Verifies that two consecutive calls to process_external_payload produce
    /// journal events with strictly increasing timestamp_ns values.
    #[test]
    fn test_consecutive_events_have_increasing_timestamps() {
        let _tempdir = tempfile::tempdir().unwrap();
        let dir = _tempdir.path().to_path_buf();

        let path = dir.join("ts_test.astra_jl");
        let _ = std::fs::remove_file(&path);

        let mut gw = make_gateway(&path);

        gw.process_external_payload(vec![1, 2, 3]).unwrap();
        // Small sleep to guarantee SystemTime advances between two calls.
        std::thread::sleep(std::time::Duration::from_millis(1));
        gw.process_external_payload(vec![4, 5, 6]).unwrap();

        // Read back the two events and verify ts ordering.
        let journal = EventJournal::open(&path).unwrap();
        let events: Vec<_> = journal.iter().unwrap().map(|r| r.unwrap()).collect();
        assert_eq!(events.len(), 2, "expected exactly 2 events");
        assert!(
            events[1].timestamp_ns > events[0].timestamp_ns,
            "timestamps must be strictly increasing: first={} second={}",
            events[0].timestamp_ns,
            events[1].timestamp_ns
        );
        assert_ne!(events[0].timestamp_ns, 0, "timestamp must not be zero");
    }
}
