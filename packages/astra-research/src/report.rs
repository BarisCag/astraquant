use crate::experiment::{ExperimentRun, ExperimentSuite};
use std::io::Write;

pub fn export_experiment_json<W: Write>(suite: &ExperimentSuite, out: &mut W) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(suite).unwrap_or_else(|_| "{}".to_string());
    write!(out, "{}", json)
}

pub fn export_experiment_csv<W: Write>(suite: &ExperimentSuite, out: &mut W) -> std::io::Result<()> {
    writeln!(out, "run_id,seed,final_hash,liquidity_drop,margin_multiplier,systemic_variance,liquidity_resilience")?;
    for run in &suite.runs {
        writeln!(
            out,
            "{},{},{},{},{},{},{}",
            run.run_id,
            run.initial_seed,
            run.final_state_hash,
            run.parameter_set.liquidity_drop_ppm,
            run.parameter_set.margin_multiplier_ppm,
            run.systemic_metrics.systemic_variance_score_ppm,
            run.systemic_metrics.liquidity_resilience_score_ppm
        )?;
    }
    Ok(())
}

pub fn export_experiment_mermaid_topology<W: Write>(suite: &ExperimentSuite, out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph TD")?;
    writeln!(out, "    Suite[{}]", suite.suite_id)?;
    for run in &suite.runs {
        writeln!(out, "    Suite --> Run_{}[{}]", run.run_id, run.run_id)?;
        writeln!(out, "    Run_{} --> Hash_{}[{}]", run.run_id, run.run_id, run.final_state_hash)?;
    }
    Ok(())
}

pub fn export_replay_certification_manifest<W: Write>(run: &ExperimentRun, out: &mut W) -> std::io::Result<()> {
    writeln!(out, "--- REPLAY CERTIFICATION MANIFEST ---")?;
    writeln!(out, "Run ID: {}", run.run_id)?;
    writeln!(out, "Seed: {}", run.initial_seed)?;
    writeln!(out, "Final State Hash: {}", run.final_state_hash)?;
    writeln!(out, "Certification Hash: {}", run.replay_certification_hash)?;
    writeln!(out, "-------------------------------------")?;
    Ok(())
}
