use crate::event::ALMEvent;
use crate::types::ALMMismatchReport;

pub struct ImmunizationTracker {
    duration_gap_threshold: f64,
}

impl ImmunizationTracker {
    pub fn new() -> Self {
        Self {
            duration_gap_threshold: 0.25,
        }
    }

    pub fn with_threshold(threshold: f64) -> Self {
        Self {
            duration_gap_threshold: threshold,
        }
    }

    pub fn check(&self, report: &ALMMismatchReport) -> Option<ALMEvent> {
        if report.duration_gap.abs() > self.duration_gap_threshold {
            Some(ALMEvent::RebalancingRecommended {
                currency: report.currency,
                tenor: report.tenor,
                duration_gap: report.duration_gap,
                recommended_action: format!(
                    "Rebalance required: Duration gap {:.2} exceeds threshold {:.2}",
                    report.duration_gap, self.duration_gap_threshold
                ),
            })
        } else {
            None
        }
    }

    pub fn check_all(&self, reports: &[ALMMismatchReport]) -> Vec<ALMEvent> {
        reports.iter().filter_map(|r| self.check(r)).collect()
    }
}

impl Default for ImmunizationTracker {
    fn default() -> Self {
        Self::new()
    }
}
