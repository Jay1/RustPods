# Battery Intelligence System: Technical Architecture and Predictive Modeling

## Executive Summary

The RustPods Battery Intelligence System implements a rigorous, time-based methodology for battery state estimation and predictive analytics. This system is engineered for reliability, computational efficiency, and high-fidelity 1% increment interpolation, leveraging only data available from the Windows BLE/CLI interface. The design eschews speculative modeling and extraneous complexity, focusing on robust, empirically grounded prediction.

## Limitations of Prior Approaches

- Excessive data logging resulting in redundant storage and file proliferation
- Absence of mathematical prediction and fine-grained (1%) state estimation
- Overly complex, multi-device and environmental factor modeling, introducing instability and maintenance risk

## System Overview: Event-Driven, Singleton, Predictive

### Design Principles
1. **Event-Driven Sampling**: Data is recorded exclusively on 10% battery decrements, reliable charging state transitions, or device reconnection events.
2. **Empirical Depletion Modeling**: The system computes depletion rates (minutes per 10% drop) and maintains a rolling buffer of up to 100 samples, ensuring statistical robustness.
3. **Minimalist Data Retention**: Only essential predictive data is retained—recent depletion rates, the most recent real reading, and the current statistical model.
4. **Precision Interpolation**: Median or mean depletion rates are used to interpolate 1% increments and forecast time-to-empty with high accuracy.
5. **Strict Data Scope**: Environmental, temperature, and advanced usage pattern factors are excluded due to their unavailability and unreliability in the Windows BLE context.
6. **Singleton Profile Enforcement**: The architecture supports a single device profile at any time, maximizing system stability and minimizing state management complexity.

### System Components

#### 1. BatteryIntelligence Controller (Singleton)
- Maintains a single, authoritative device profile
- Orchestrates event logging, model updates, and persistent storage

#### 2. DeviceBatteryProfile
- Records only significant battery events (10% decrements, charging transitions, reconnections)
- Maintains a rolling buffer (maximum 100) of depletion rates (minutes per 10%)
- Stores the most recent real battery reading

#### 3. BatteryEvent
- Events are logged exclusively for:
  - 10% battery decrements
  - Charging state transitions (if reliably detected)
  - Device reconnections after communication gaps

#### 4. Depletion Rate Model
- On each 10% decrement, the system records the timestamp and battery level
- Computes the elapsed time since the previous decrement
- Calculates and stores the depletion rate (minutes per 10%)
- Utilizes the median or mean of the most recent N rates for predictive modeling

#### 5. BatteryEstimate
- Interpolates 1% increments using the current statistical depletion rate
- Predicts time-to-empty and provides a confidence metric based on sample count

## Data Collection and Predictive Logic

### Event Filtering
```rust
// Log event only if:
- Battery level decreased by 10%
- Charging state changed (if reliably detected)
- Device reconnected after a gap
```

### Rolling Buffer Management
- Retain up to 100 depletion rate samples (minutes per 10%)
- Discard the oldest sample upon buffer saturation

### Depletion Rate Computation
```rust
// On each 10% decrement:
let time_elapsed = current_timestamp - previous_timestamp;
let depletion_rate = time_elapsed / 10.0; // minutes per 1%
// Store in rolling buffer
```

### 1% Increment Interpolation and Prediction
```rust
let median_rate = median(last_100_depletion_rates);
let time_per_1_percent = median_rate / 10.0;
let estimated_level = last_known_level - (elapsed_time / time_per_1_percent);
```
- Time-to-empty = estimated_level * time_per_1_percent
- Confidence metric is a function of sample count (e.g., >30 samples = high confidence)

## Data Retention Policy
- Retain only the last 100 depletion rates (timestamp, rate, start/end %)
- Store the most recent real battery reading
- Persist the current statistical prediction model (median/mean rate)

## User Interface Integration
- Display estimated battery percentage (1% increments)
- Present time-to-empty prediction
- Indicate confidence level based on statistical sample size

## Logging and Observability Protocol

All event logging, error reporting, and diagnostic output within the Battery Intelligence System must conform to the standards and protocols defined in [logging-best-practices.md](logging-best-practices.md). This ensures operational observability, facilitates debugging, and supports production monitoring.

### Logging Requirements
- Use the `debug_log!` macro for all category-specific debug output (category: `airpods` or `system`).
- Log all significant battery events, model updates, and prediction anomalies.
- Include contextual information (device ID, event type, battery levels, timestamps, computed rates).
- Avoid excessive or redundant logging; focus on meaningful state transitions and errors.

### Example Log Messages
```rust
crate::debug_log!("airpods", "Battery event: 10% drop detected, device: {}, from {}% to {}%, elapsed: {} min", device_id, prev_level, curr_level, elapsed_min);
crate::debug_log!("airpods", "Depletion rate buffer updated: {} samples, median rate: {:.2} min/10%", buffer_len, median_rate);
crate::debug_log!("airpods", "Prediction: {}% remaining, estimated time-to-empty: {} min, confidence: {}", est_level, time_to_empty, confidence);
crate::debug_log!("system", "Battery intelligence model error: {}", error_msg);
```

## Implementation Directives
- Exclude temperature, RSSI, and advanced usage pattern modeling
- Charging state is considered only if reliably detected (case open)
- Multi-device support is explicitly excluded; singleton enforcement is mandatory
- This document supersedes all prior advanced modeling proposals

## Verification and Validation
- Unit tests: depletion rate computation, rolling buffer management, interpolation logic
- Integration tests: end-to-end prediction using simulated battery drop sequences
- Logging output: verify that all significant events and errors are logged according to [logging-best-practices.md](logging-best-practices.md)

## Reference
**This document constitutes the definitive technical specification for the RustPods Battery Intelligence System as of June 2025.** 