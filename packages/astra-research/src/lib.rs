pub mod experiment;
pub mod sweep;
pub mod comparison;
pub mod report;
pub mod dataset_format;
pub mod flash_crash_dataset;
pub mod lehman_2008_dataset;
pub mod covid_2020_dataset;
pub mod phantom_runner;
pub mod counterfactual;
pub mod report_generator;
pub mod certification_generator;
pub mod calibration;

#[cfg(test)]
mod tests {
    use super::*;
    use astra_core::hashing::hash_to_hex;
    use std::fs;
    use std::collections::BTreeMap;

    #[test]
    fn test_ci_golden_hashes_match() {
        // Load the golden hashes from disk
        let golden_json = match fs::read_to_string("results/golden_hashes.json") {
            Ok(s) => s,
            Err(_) => {
                println!("No golden hashes found, skipping CI validation (run astra-research first)");
                return;
            }
        };
        let golden_hashes: BTreeMap<String, String> = serde_json::from_str(&golden_json).unwrap();

        let datasets = vec![
            ("flash_crash_2010", flash_crash_dataset::build_flash_crash_events()),
            ("lehman_2008", lehman_2008_dataset::build_lehman_collapse_events()),
            ("covid_2020", covid_2020_dataset::build_covid_crash_events()),
        ];

        for (id, events) in datasets {
            let expected_hash = golden_hashes.get(id).expect("Golden hash missing for dataset");
            
            let dataset = dataset_format::CrisisDataset {
                header: dataset_format::DatasetHeader {
                    format_version: 1,
                    crisis_name: id.to_string(),
                    date_range: "".to_string(),
                    event_count: events.len() as u64,
                },
                events,
            };

            let mut runner = phantom_runner::PhantomRunner::new();
            runner.run(&dataset, 100);
            let final_hash = hash_to_hex(&runner.final_hash());

            assert_eq!(
                &final_hash, expected_hash,
                "CRITICAL: Replay divergence detected for dataset '{}'. \
                 Expected golden hash {}, but got {}. \
                 This violates the determinism guarantee and fails the CI gate.",
                id, expected_hash, final_hash
            );
        }
    }
}


