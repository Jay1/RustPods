#!/bin/bash

# Check if cargo-tarpaulin is installed
if ! command -v cargo-tarpaulin &> /dev/null; then
    echo "cargo-tarpaulin is not installed. Installing now..."
    cargo install cargo-tarpaulin
fi

# Create coverage directory if it doesn't exist
mkdir -p ./coverage

echo "Running test coverage with tarpaulin..."
cargo tarpaulin --verbose --workspace --timeout 120 --out Html --output-dir ./coverage
cargo tarpaulin --verbose --workspace --timeout 120 --out Json --output-dir ./coverage

# Open the HTML report in the default browser
echo "Opening coverage report..."
if [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
    # Windows
    start ./coverage/tarpaulin-report.html
elif [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    open ./coverage/tarpaulin-report.html
else
    # Linux
    xdg-open ./coverage/tarpaulin-report.html 2>/dev/null || sensible-browser ./coverage/tarpaulin-report.html 2>/dev/null || echo "Please open ./coverage/tarpaulin-report.html manually"
fi

echo "Coverage report has been generated in ./coverage/tarpaulin-report.html"
echo "Coverage data saved to ./coverage/coverage.json for use with VS Code extensions" 