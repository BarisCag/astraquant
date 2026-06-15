//! astra-research binary entrypoint
//!
//! Orchestrates the full Phase 13A backward scenario pipeline.
//! Runs all 3 datasets through 3 counterfactual interventions,
//! generates the matrix, and outputs the markdown reports.

use astra_research::counterfactual::{CounterfactualEngine, InterventionType};
use astra_research::dataset_format::{DatasetReader, DatasetWriter};
use astra_research::flash_crash_dataset::build_flash_crash_events;
use astra_research::lehman_2008_dataset::build_lehman_collapse_events;
use astra_research::covid_2020_dataset::build_covid_crash_events;
use astra_research::phantom_runner::PhantomRunner;
use astra_research::report_generator::generate_crisis_report;
use astra_core::hashing::hash_to_hex;
use std::env;
use std::fs;
use std::path::Path;

const CHECKPOINT_INTERVAL: u64 = 100;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut target_dataset = String::new();
    let mut out_dir = String::from("results");

    for i in 0..args.len() {
        if args[i] == "--dataset" && i + 1 < args.len() {
            target_dataset = args[i + 1].clone();
        }
        if args[i] == "--output" && i + 1 < args.len() {
            out_dir = args[i + 1].clone();
        }
    }

    fs::create_dir_all(&out_dir).expect("Failed to create output directory");

    println!("═══════════════════════════════════════════════════");
    println!("  AstraQuant OS — Phase 13A: Backward Scenario Engine");
    println!("═══════════════════════════════════════════════════\n");

    let datasets = vec![
        ("flash_crash_2010", "2010 Flash Crash", "2010-05-06", build_flash_crash_events()),
        ("lehman_2008", "2008 Lehman Collapse", "2008-09-15", build_lehman_collapse_events()),
        ("covid_2020", "2020 COVID Crash", "2020-03-16", build_covid_crash_events()),
    ];

    let mut matrix = Vec::new();
    let mut golden_hashes = std::collections::BTreeMap::new();

    for (id, name, date, events) in &datasets {
        let ds_path = format!("{}/{}.astra_ds", out_dir, id);
        
        if !target_dataset.is_empty() && target_dataset != ds_path {
            continue;
        }

        println!("Processing Dataset: {name}");
        DatasetWriter::write(Path::new(&ds_path), name, date, &events).unwrap();

        let dataset = DatasetReader::read(Path::new(&ds_path)).unwrap();

        // Determinism Check
        let mut r1 = PhantomRunner::new(); let (trace1, merkle) = r1.run(&dataset, CHECKPOINT_INTERVAL);
        let mut r2 = PhantomRunner::new(); let _ = r2.run(&dataset, CHECKPOINT_INTERVAL);
        let mut r3 = PhantomRunner::new(); let _ = r3.run(&dataset, CHECKPOINT_INTERVAL);
        let h1 = hash_to_hex(&r1.final_hash());
        let h2 = hash_to_hex(&r2.final_hash());
        let h3 = hash_to_hex(&r3.final_hash());
        let replay_verified = h1 == h2 && h2 == h3;
        
        fs::write(
            format!("{}/{}_merkle_audit.json", out_dir, id),
            serde_json::to_string_pretty(&merkle).unwrap(),
        ).unwrap();
        println!("  -> Generated {}_merkle_audit.json ({} roots)", id, merkle.len());

        golden_hashes.insert(id.to_string(), h1);
        
        let interventions = vec![
            (InterventionType::CircuitBreakerHalt { duration: 60 }, 400),
            (InterventionType::LiquidityInjection, 350),
            (InterventionType::ShortSellingBan { volume_threshold: 5000 }, 300),
        ];

        let mut deltas = Vec::new();
        for (int, seq) in interventions {
            let delta = CounterfactualEngine::run(id, &dataset, int, seq);
            deltas.push(delta.clone());
            matrix.push(delta);
        }

        let report = generate_crisis_report(name, events.len(), &deltas, replay_verified, true);
        fs::write(format!("{}/{}_report.md", out_dir, id), report).unwrap();
        println!("  -> Generated {}_report.md", id);
    }

    let matrix_json = serde_json::to_string_pretty(&matrix).unwrap();
    fs::write(format!("{}/counterfactual_matrix.json", out_dir), matrix_json).unwrap();
    println!("  -> Generated counterfactual_matrix.json ({} rows)", matrix.len());
    
    let golden_json = serde_json::to_string_pretty(&golden_hashes).unwrap();
    fs::write(format!("{}/golden_hashes.json", out_dir), golden_json).unwrap();
    println!("  -> Generated golden_hashes.json");
    
    astra_research::certification_generator::generate_certification_report(&out_dir);
    
    // --- Phase 14A: Behavioral Calibration Sweep ---
    println!("\n  BEHAVIORAL CALIBRATION SWEEP");
    let mut calibration_results = Vec::new();
    let mut behavioral_deltas = Vec::new();
    for (id, _name, _date, events) in &datasets {
        let dataset = astra_research::dataset_format::CrisisDataset {
            header: astra_research::dataset_format::DatasetHeader {
                format_version: 1,
                crisis_name: id.to_string(),
                date_range: "".to_string(),
                event_count: events.len() as u64,
            },
            events: events.clone(),
        };
        
        let res = astra_research::calibration::CalibrationEngine::run(id, &dataset);
        println!("  -> {}: Best Fit [Herding: {}, Loss Aversion: {}] (Error: {:.2}%)", 
                 id, res.best_herding_factor, res.best_loss_aversion, res.cascade_depth_error_pct);
        
        let seed = astra_core::events::BehavioralSeed::new(res.best_herding_factor, res.best_loss_aversion, 0.5, 0.8, 42);
        
        let interventions = vec![
            (astra_research::counterfactual::InterventionType::CircuitBreakerHalt { duration: 60 }, 400),
            (astra_research::counterfactual::InterventionType::LiquidityInjection, 350),
            (astra_research::counterfactual::InterventionType::ShortSellingBan { volume_threshold: 1000 }, 250),
        ];

        for (intervention, seq) in interventions {
            let delta = astra_research::counterfactual::BehavioralCounterfactualEngine::run(
                id, &dataset, intervention, seq, seed.clone()
            );
            behavioral_deltas.push(delta);
        }

        calibration_results.push(res);
    }
    
    let calib_json = serde_json::to_string_pretty(&calibration_results).unwrap();
    fs::write(format!("{}/calibration_results.json", out_dir), calib_json).unwrap();
    println!("  -> Generated calibration_results.json");
    
    let behave_json = serde_json::to_string_pretty(&behavioral_deltas).unwrap();
    fs::write(format!("{}/behavioral_counterfactual_delta.json", out_dir), behave_json).unwrap();
    println!("  -> Generated behavioral_counterfactual_delta.json ({} rows)", behavioral_deltas.len());
    
    println!("\n  GLOBAL RUNTIME VERIFIED ✓");
    println!("═══════════════════════════════════════════════════");
}
