use crate::events::AstraEvent;
use crate::kernel::AstraKernel;
use crate::replay::EventReducer;

pub struct Synchronizer;

impl Synchronizer {
    pub fn apply_replicated_events(
        kernel: &mut AstraKernel,
        events: &[AstraEvent],
    ) -> Result<(), String> {
        for event in events {
            kernel.apply(event).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}
