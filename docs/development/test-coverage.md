# Test Coverage Guidelines

This document outlines our test coverage goals and identifies critical paths in the codebase that require high coverage.

## Coverage Reporting

We use [cargo-tarpaulin](https://github.com/xd009642/tarpaulin) for generating test coverage reports. Coverage reports are automatically generated as part of our CI process for all pull requests and merges to the main branch, and the results are published to [Codecov](https://codecov.io).

### Running Coverage Locally

You can generate coverage reports locally with:

```bash
# Install cargo-tarpaulin (if not already installed)
cargo install cargo-tarpaulin

# Generate coverage report in terminal
cargo tarpaulin --verbose

# Generate HTML report
cargo tarpaulin --verbose --out Html --output-dir ./coverage
```

## Coverage Goals

Our general coverage goals are:

| Component Type | Target Coverage |
|----------------|----------------|
| Core logic     | > 85%          |
| UI components  | > 70%          |
| Test utilities | > 90%          |
| Overall        | > 75%          |

## Critical Paths

The following components are considered critical paths and require high test coverage (>85%):

### Bluetooth Core (`src/bluetooth/`)

- `scanner.rs` - Core scanning functionality
- `adapter.rs` - Bluetooth adapter management
- `scanner_config.rs` - Scanning configuration

### AirPods Detection (`src/airpods/`)

- `detector.rs` - AirPods detection from advertisements
- `battery.rs` - Battery status extraction

### Configuration (`src/config/`)

- `app_config.rs` - Application configuration management

### State Management (`src/ui/state_manager.rs`)

- State transitions
- Event handling
- Action dispatching

## UI Component Testing

While UI components cannot always achieve the same level of coverage as core logic, we aim to test the following aspects:

1. **Component initialization** - Ensure components initialize correctly with various props
2. **State changes** - Test that components update their state correctly
3. **Event handlers** - Verify event handlers are connected and working
4. **Rendering logic** - Test conditional rendering paths

## Integration Components (Tasks 8-10)

The integration components from Tasks 8-10 require special attention to testing:

### Settings UI Window (Task 8)

- Form validation logic
- Settings persistence
- Settings loading
- Change event propagation

### System Tray Integration (Task 9)

- Menu item functionality
- Icon updates
- Event handling for tray actions

### UI/Application State Integration (Task 10)

- State flow between components
- Message handling
- Window visibility management

## Excluded from Coverage Goals

The following are recognized as challenging to test and may be excluded from coverage calculations:

1. Platform-specific code that cannot be tested on CI
2. Event handlers for user input that require actual UI interaction
3. Components that directly interact with hardware
4. Third-party library integrations where mocking is impractical

## Improving Coverage

When adding new features or modifying existing code:

1. Write tests before or alongside the implementation
2. Use mocks and fixtures to isolate system dependencies
3. Structure code to be testable by separating core logic from UI
4. Consider test coverage when reviewing pull requests

By following these guidelines, we aim to maintain high-quality test coverage that ensures application reliability without adding unnecessary testing overhead. 