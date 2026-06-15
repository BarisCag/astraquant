# Market Microstructure Research in AstraQuant

AstraQuant's Phase 7A implementation transforms the platform into an institutional-grade, deterministic execution research environment.

## The Problem with Stochastic Simulation

Most market simulators attempt to model queue priority and queue jumping using probability (e.g., Gaussian slippage, Monte Carlo fill rates). While visually appealing, these stochastic engines are strictly forbidden in AstraQuant because:
1. **They break deterministic replay.** If identical orders result in random fills, you cannot reliably benchmark code optimization or strategy tuning.
2. **They obscure true edge cases.** Random latency or arbitrary fill percentages make it impossible to deterministically prove a risk limit was hit precisely on the correct engine sequence.

## The Structural Approach to Microstructure

AstraQuant handles market impact and queue evolution structurally rather than probabilistically.

### Queue Position & Advancement

Instead of calculating a "fill probability", AstraQuant deterministically tracks exact position in the order queue.
* `QueuePosition`: Tracks the precise quantity sitting ahead of and behind an order at the moment of insertion.
* `QueueState`: Efficient cumulative depletion accounting (`cumulative_depleted_quantity`) ensures O(1) amortized tracking without O(N) mutation loops across the `VecDeque`. 

As fills and cancellations occur ahead of a resting order, its position strictly decrements until it reaches the front of the queue and becomes executed.

### Execution Quality Analytics

Instead of approximating execution quality with floating-point math, AstraQuant utilizes basis points and Parts-Per-Million (PPM) strictly.

* **Realized Spread (x2)**: Uses `scaled_midpoint = best_bid + best_ask` to avoid fractional representations, eliminating `f64` totally. 
* **Passive Fill Ratio**: Calculated purely on integer quantities via `total_passive / (total_passive + total_aggressive)`.
* **Queue Survival**: Analyzes what percentage of submitted passive intent actually reached execution rather than being cancelled ahead of time.

## Market Impact Approximations

Impact modeling is strictly deterministic:
* **Sweep Cost**: Calculates the volume-weighted average price difference from the arrival BBO based on the *actual swept depth* of the book.
* **Adverse Price Distance**: The maximum price movement explicitly caused by the aggressive order penetrating the depth.

By remaining strictly deterministic, institutional researchers can use `astra-inspect` to produce byte-for-byte identical reports on queue structures across any target architecture without fear of rounding discrepancies.
