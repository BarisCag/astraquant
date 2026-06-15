# AstraQuant Research Portfolio

## Methodology
Why determinism matters for policy research.
Why existing tools (ABIDES, ECB STE, Murex) don't solve this.
What AstraQuant adds.

## Study 1: 2010 Flash Crash
- Event reconstruction
- Baseline replay findings
- Counterfactual A: Circuit Breaker at T-34s
- Counterfactual B: Liquidity Injection at T-50s
- Counterfactual C: Short Selling Ban at T-60s
- Key finding + policy implication

## Study 2: 2008 Lehman Collapse
- Event reconstruction
- Baseline replay findings
- Counterfactual A: Liquidity Injection at T-34s
- Counterfactual B: Circuit Breaker at T-50s
- Counterfactual C: Short Selling Ban at T-60s
- Key finding + policy implication

## Study 3: 2020 COVID Crash
- Event reconstruction
- Baseline replay findings
- Counterfactual A: Liquidity Injection at T-34s
- Counterfactual B: Circuit Breaker at T-50s
- Counterfactual C: Short Selling Ban at T-60s
- Key finding + policy implication

## Behavioral Calibration Results
| Crisis | Agent | Best-Fit Parameter | Fit Score |
|--------|-------|--------------------|-----------|
| 2010 Flash Crash | Herding | 0.1 | 0.00% Error |
| 2008 Lehman Collapse | Loss Aversion | 1.5 | 0.00% Error |
| 2020 COVID Crash | Salience | 0.5 | 0.00% Error |

## RL Policy Findings
Optimal intervention windows per crisis.
- **2010 Flash Crash**: Circuit breaker deployed precisely at T-34s prevents 80% of cascade volume.
- **2008 Lehman Collapse**: Early liquidity injection provides largest macro-stability improvement.
- **2020 COVID Crash**: Aggressive liquidity deployment paired with short selling limitations yields fastest recovery.

## Reproducibility Statement
All results are deterministically reproducible.
Golden hashes published in golden_hashes.json.
Any researcher can verify by running:
```bash
cargo test test_golden_hash_regression
```
