# PowerShell script for running tarpaulin on Windows

# Check if cargo-tarpaulin is installed
$tarpaulinExists = $null -ne (Get-Command cargo-tarpaulin -ErrorAction SilentlyContinue)
if (-not $tarpaulinExists) {
    Write-Host "cargo-tarpaulin is not installed. Installing now..."
    cargo install cargo-tarpaulin
}

# Create coverage directory if it doesn't exist
if (-not (Test-Path -Path ".\coverage")) {
    New-Item -Path ".\coverage" -ItemType Directory | Out-Null
}

Write-Host "Running test coverage with tarpaulin..."
cargo tarpaulin --verbose --workspace --timeout 120 --out Html --output-dir ./coverage
cargo tarpaulin --verbose --workspace --timeout 120 --out Json --output-dir ./coverage

# Open the HTML report in the default browser
Write-Host "Opening coverage report..."
Start-Process ".\coverage\tarpaulin-report.html"

Write-Host "Coverage report has been generated in ./coverage/tarpaulin-report.html"
Write-Host "Coverage data saved to ./coverage/coverage.json for use with VS Code extensions" 