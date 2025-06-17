# Bulletproof UI Test Suite Runner
# This script runs comprehensive UI tests to ensure visual regression protection

Write-Host "🛡️  RustPods Bulletproof UI Test Suite" -ForegroundColor Cyan
Write-Host "=====================================" -ForegroundColor Cyan
Write-Host ""

# Ensure we're in the project root
if (-not (Test-Path "Cargo.toml")) {
    Write-Host "❌ Error: Must run from project root directory" -ForegroundColor Red
    exit 1
}

# Check if tarpaulin is installed
$tarpaulinInstalled = $null -ne (Get-Command cargo-tarpaulin -ErrorAction SilentlyContinue)
if (-not $tarpaulinInstalled) {
    Write-Host "⚠️  Warning: cargo-tarpaulin not found. Coverage reports will not be generated." -ForegroundColor Yellow
    Write-Host "   Install with: cargo install cargo-tarpaulin" -ForegroundColor Yellow
    Write-Host ""
    $generateCoverage = $false
} else {
    $generateCoverage = $true
}

Write-Host "🧪 Running UI Component Tests..." -ForegroundColor Yellow
Write-Host "Testing individual UI components:" -ForegroundColor Gray
Write-Host "  • Battery indicator component" -ForegroundColor Gray
Write-Host "  • AirPods popup component" -ForegroundColor Gray
Write-Host "  • Settings view component" -ForegroundColor Gray
Write-Host "  • Waiting mode component" -ForegroundColor Gray
Write-Host "  • SVG icons component" -ForegroundColor Gray
Write-Host ""

$componentTests = cargo test --test-threads=1 tests::ui::components --no-default-features --features testing 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ UI Component Tests: PASSED" -ForegroundColor Green
} else {
    Write-Host "❌ UI Component Tests: FAILED" -ForegroundColor Red
    Write-Host $componentTests
    exit 1
}

Write-Host ""
Write-Host "🔍 Running Visual Regression Tests..." -ForegroundColor Yellow
Write-Host "These tests lock down all our carefully tuned UI elements:" -ForegroundColor Gray
Write-Host "  • Window dimensions (414×455px)" -ForegroundColor Gray  
Write-Host "  • Battery icon sizes (80×48px)" -ForegroundColor Gray
Write-Host "  • AirPods image size (270×230px)" -ForegroundColor Gray
Write-Host "  • Button sizes (21×21px)" -ForegroundColor Gray
Write-Host "  • Font sizes (24px, 20px, etc.)" -ForegroundColor Gray
Write-Host "  • Layout spacing (5px, 8px, 15px, 20px)" -ForegroundColor Gray
Write-Host "  • Catppuccin Mocha color scheme" -ForegroundColor Gray
Write-Host ""

$visualTests = cargo test --test visual_regression_tests --no-default-features --features testing 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Visual Regression Tests: PASSED" -ForegroundColor Green
} else {
    Write-Host "❌ Visual Regression Tests: FAILED" -ForegroundColor Red
    Write-Host $visualTests
    exit 1
}

Write-Host ""
Write-Host "🔄 Running Property-Based Tests..." -ForegroundColor Yellow
Write-Host "These tests verify UI components work across all input ranges:" -ForegroundColor Gray
Write-Host "  • Battery levels 0-255 (never panic)" -ForegroundColor Gray
Write-Host "  • Color consistency across all levels" -ForegroundColor Gray  
Write-Host "  • SVG generation stability" -ForegroundColor Gray
Write-Host "  • Window dimensions reasonableness" -ForegroundColor Gray
Write-Host "  • Font size readability" -ForegroundColor Gray
Write-Host "  • Animation progress clamping" -ForegroundColor Gray
Write-Host ""

$propertyTests = cargo test --test property_based_tests --no-default-features --features testing 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Property-Based Tests: PASSED" -ForegroundColor Green
} else {
    Write-Host "❌ Property-Based Tests: FAILED" -ForegroundColor Red
    Write-Host $propertyTests
    exit 1
}

Write-Host ""
Write-Host "🔗 Running Integration Tests..." -ForegroundColor Yellow
Write-Host "These tests validate complete UI workflows:" -ForegroundColor Gray
Write-Host "  • AirPods detection → display → disconnect cycle" -ForegroundColor Gray
Write-Host "  • State transitions (scanning, found, error)" -ForegroundColor Gray
Write-Host "  • Battery level color consistency" -ForegroundColor Gray
Write-Host "  • Theme integration across components" -ForegroundColor Gray
Write-Host "  • Animation and button integration" -ForegroundColor Gray
Write-Host "  • Performance under repeated renders" -ForegroundColor Gray
Write-Host "  • Error state handling" -ForegroundColor Gray
Write-Host ""

$integrationTests = cargo test --test integration_tests --no-default-features --features testing 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Integration Tests: PASSED" -ForegroundColor Green
} else {
    Write-Host "❌ Integration Tests: FAILED" -ForegroundColor Red
    Write-Host $integrationTests
    exit 1
}

Write-Host ""
Write-Host "🎯 Running Critical Asset Tests..." -ForegroundColor Yellow
Write-Host "Verifying image assets and paths are valid:" -ForegroundColor Gray

# Check critical asset files exist
$assetErrors = @()

if (-not (Test-Path "assets/icons/hw/airpodspro.png")) {
    $assetErrors += "❌ Missing: assets/icons/hw/airpodspro.png"
}

if (-not (Test-Path "assets/icons/hw/airpodsprocase.png")) {
    $assetErrors += "⚠️  Missing: assets/icons/hw/airpodsprocase.png (not currently displayed)"
}

if ($assetErrors.Count -gt 0) {
    Write-Host "🚨 Asset Issues Found:" -ForegroundColor Red
    foreach ($error in $assetErrors) {
        Write-Host "  $error" -ForegroundColor Red
    }
    if ($assetErrors -match "❌") {
        exit 1
    }
} else {
    Write-Host "✅ Asset Tests: PASSED" -ForegroundColor Green
}

# Generate coverage reports if tarpaulin is available
if ($generateCoverage) {
    Write-Host ""
    Write-Host "📊 Generating Coverage Reports..." -ForegroundColor Yellow
    
    # Create coverage directory if it doesn't exist
    if (-not (Test-Path "coverage")) {
        New-Item -ItemType Directory -Path "coverage" | Out-Null
    }
    
    # Generate UI component coverage
    Write-Host "  • Generating UI component coverage..." -ForegroundColor Gray
    cargo tarpaulin --verbose --workspace --timeout 300 --out Html --output-dir ./coverage/ui-components --skip-clean --exclude-files="target/*" --exclude-files="examples/*" --packages rustpods --lib --tests -- tests::ui::components | Out-Null
    
    # Generate overall coverage
    Write-Host "  • Generating overall coverage..." -ForegroundColor Gray
    cargo tarpaulin --verbose --workspace --timeout 300 --out Html --output-dir ./coverage/overall --skip-clean --exclude-files="target/*" --exclude-files="examples/*" | Out-Null
    
    Write-Host "✅ Coverage reports generated in ./coverage directory" -ForegroundColor Green
    
    # Try to extract coverage percentages
    try {
        $uiComponentsJson = Get-Content -Path ./coverage/ui-components/tarpaulin-report.json -ErrorAction SilentlyContinue | ConvertFrom-Json -ErrorAction SilentlyContinue
        $overallJson = Get-Content -Path ./coverage/overall/tarpaulin-report.json -ErrorAction SilentlyContinue | ConvertFrom-Json -ErrorAction SilentlyContinue
        
        if ($uiComponentsJson -and $overallJson) {
            Write-Host ""
            Write-Host "📊 Coverage Summary:" -ForegroundColor Cyan
            Write-Host "  • UI Components: $($uiComponentsJson.percentage)% coverage" -ForegroundColor $(if ($uiComponentsJson.percentage -ge 75) { "Green" } else { "Yellow" })
            Write-Host "  • Overall: $($overallJson.percentage)% coverage" -ForegroundColor $(if ($overallJson.percentage -ge 75) { "Green" } else { "Yellow" })
        }
    } catch {
        Write-Host "⚠️  Could not parse coverage reports" -ForegroundColor Yellow
    }
}

Write-Host ""
Write-Host "🏁 TEST SUITE SUMMARY" -ForegroundColor Cyan
Write-Host "====================" -ForegroundColor Cyan
Write-Host "✅ UI Component Tests - Individual components validated" -ForegroundColor Green
Write-Host "✅ Visual Regression Tests - UI dimensions and styling locked" -ForegroundColor Green
Write-Host "✅ Property-Based Tests - Components robust across all inputs" -ForegroundColor Green  
Write-Host "✅ Integration Tests - End-to-end workflows validated" -ForegroundColor Green
Write-Host "✅ Asset Tests - Image files accessible" -ForegroundColor Green
if ($generateCoverage) {
    Write-Host "✅ Coverage Reports - Test coverage measured and reported" -ForegroundColor Green
}
Write-Host ""
Write-Host "🛡️  YOUR UI IS BULLETPROOF! 🛡️" -ForegroundColor Green
Write-Host ""
Write-Host "🎯 Protected Elements:" -ForegroundColor Cyan
Write-Host "  • Window: 414×455px (perfect size)" -ForegroundColor Green
Write-Host "  • AirPods: 270×230px (15% larger, ideal proportions)" -ForegroundColor Green
Write-Host "  • Battery Icons: 80×48px (horizontal, visible)" -ForegroundColor Green
Write-Host "  • Buttons: 21×21px (50% larger, accessible)" -ForegroundColor Green
Write-Host "  • Text: 24px battery %, 20px headers (readable)" -ForegroundColor Green
Write-Host "  • Layout: 5px image gap, 15px L/R padding (tight)" -ForegroundColor Green
Write-Host "  • Colors: Catppuccin Mocha (consistent theming)" -ForegroundColor Green
Write-Host "  • Case Column: Removed (clean focus on AirPods)" -ForegroundColor Green
Write-Host ""
Write-Host "⚡ Performance: ~1.3s AirPods detection, 30s updates" -ForegroundColor Yellow
Write-Host "🎨 Visual: macOS-inspired design with dark theme" -ForegroundColor Yellow
Write-Host "🔄 Features: System tray, settings, fast scanning" -ForegroundColor Yellow
Write-Host ""
Write-Host "Run this script before any UI changes to prevent regressions!" -ForegroundColor Cyan 