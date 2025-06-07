#!/usr/bin/env pwsh
# Test Performance Monitoring Script for RustPods
# This script runs tests with timing and provides performance insights

param(
    [switch]$All,
    [switch]$LibOnly,
    [switch]$Fast,
    [switch]$Verbose,
    [string]$Filter = ""
)

Write-Host "🚀 RustPods Test Performance Monitor" -ForegroundColor Green
Write-Host "====================================" -ForegroundColor Green

# Performance tracking function
function Measure-TestSuite {
    param(
        [string]$TestName,
        [string]$Command,
        [string]$Description
    )
    
    Write-Host "`n⏱️  Running: $TestName" -ForegroundColor Yellow
    Write-Host "   $Description" -ForegroundColor Gray
    
    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
    
    try {
        $result = Invoke-Expression $Command 2>&1
        $stopwatch.Stop()
        
        $duration = $stopwatch.ElapsedMilliseconds
        $status = if ($LASTEXITCODE -eq 0) { "✅ PASSED" } else { "❌ FAILED" }
        
        # Categorize performance
        $perfCategory = switch ($duration) {
            { $_ -le 100 } { "🟢 EXCELLENT" }
            { $_ -le 500 } { "🟡 GOOD" }
            { $_ -le 2000 } { "🟠 ACCEPTABLE" }
            default { "🔴 SLOW" }
        }
        
        Write-Host "   $status in ${duration}ms ($perfCategory)" -ForegroundColor $(if ($LASTEXITCODE -eq 0) { "Green" } else { "Red" })
        
        # Show output if verbose or failed
        if ($Verbose -or $LASTEXITCODE -ne 0) {
            Write-Host "   Output:" -ForegroundColor Gray
            $result | ForEach-Object { Write-Host "     $_" -ForegroundColor Gray }
        }
        
        return @{
            Name = $TestName
            Duration = $duration
            Success = ($LASTEXITCODE -eq 0)
            Category = $perfCategory
            Output = $result
        }
    }
    catch {
        $stopwatch.Stop()
        Write-Host "   ❌ CRASHED in $($stopwatch.ElapsedMilliseconds)ms" -ForegroundColor Red
        Write-Host "   Error: $_" -ForegroundColor Red
        
        return @{
            Name = $TestName
            Duration = $stopwatch.ElapsedMilliseconds
            Success = $false
            Category = "🔴 CRASHED"
            Output = $_
        }
    }
}

# Test configurations
$testSuites = @()

if ($LibOnly -or $Fast) {
    $testSuites += @{
        Name = "Library Tests (Fast)"
        Command = "cargo test --lib --no-default-features"
        Description = "Core library unit tests only"
    }
}

if ($Fast) {
    $testSuites += @{
        Name = "Quick Smoke Tests"
        Command = "cargo test test_config_manager_default test_default_config test_scanner_new --lib --no-default-features"
        Description = "Essential functionality verification"
    }
}

if ($All) {
    $testSuites += @{
        Name = "Library Tests"
        Command = "cargo test --lib"
        Description = "All unit tests in src/"
    }
    
    $testSuites += @{
        Name = "Integration Tests (Lightweight)"
        Command = "cargo test --test '*' --no-default-features"
        Description = "Integration tests without heavy async operations"
    }
    
    $testSuites += @{
        Name = "Full Test Suite"
        Command = "cargo test"
        Description = "All tests including integration tests"
    }
}

# Default: run fast tests
if (-not $LibOnly -and -not $All -and -not $Fast) {
    $testSuites += @{
        Name = "Default Test Suite"
        Command = "cargo test --lib --no-default-features"
        Description = "Library tests only (recommended)"
    }
}

# Apply filter if specified
if ($Filter) {
    $testSuites = $testSuites | ForEach-Object {
        $_.Command += " $Filter"
        $_
    }
}

# Run test suites and collect results
$results = @()
foreach ($suite in $testSuites) {
    $result = Measure-TestSuite -TestName $suite.Name -Command $suite.Command -Description $suite.Description
    $results += $result
}

# Performance summary
Write-Host "`n📊 PERFORMANCE SUMMARY" -ForegroundColor Cyan
Write-Host "======================" -ForegroundColor Cyan

$totalDuration = ($results | Measure-Object -Property Duration -Sum).Sum
$successCount = ($results | Where-Object { $_.Success }).Count
$totalCount = $results.Count

Write-Host "Total Duration: ${totalDuration}ms" -ForegroundColor White
Write-Host "Success Rate: $successCount/$totalCount tests" -ForegroundColor $(if ($successCount -eq $totalCount) { "Green" } else { "Yellow" })

# Sort by duration (slowest first)
$sortedResults = $results | Sort-Object Duration -Descending

Write-Host "`nPerformance Breakdown:" -ForegroundColor White
foreach ($result in $sortedResults) {
    $status = if ($result.Success) { "✅" } else { "❌" }
    Write-Host "  $status $($result.Category) $($result.Name): $($result.Duration)ms" -ForegroundColor White
}

# Performance recommendations
Write-Host "`n💡 PERFORMANCE RECOMMENDATIONS:" -ForegroundColor Magenta

$slowTests = $results | Where-Object { $_.Duration -gt 2000 }
if ($slowTests.Count -gt 0) {
    Write-Host "• The following tests are slow (>2s):" -ForegroundColor Yellow
    $slowTests | ForEach-Object { Write-Host "  - $($_.Name): $($_.Duration)ms" -ForegroundColor Yellow }
    Write-Host "  Consider disabling heavy integration tests or optimizing async operations" -ForegroundColor Yellow
}

$fastTests = $results | Where-Object { $_.Duration -le 100 }
if ($fastTests.Count -gt 0) {
    Write-Host "• These tests are excellent (≤100ms): $($fastTests.Count)" -ForegroundColor Green
}

Write-Host "• For daily development, use: ./scripts/test_performance.ps1 -Fast" -ForegroundColor Cyan
Write-Host "• For comprehensive testing, use: ./scripts/test_performance.ps1 -All" -ForegroundColor Cyan
Write-Host "• Library tests only: ./scripts/test_performance.ps1 -LibOnly" -ForegroundColor Cyan

# Exit with appropriate code
if ($successCount -eq $totalCount) {
    Write-Host "`n🎉 All tests passed!" -ForegroundColor Green
    exit 0
} else {
    Write-Host "`n⚠️  Some tests failed. Check output above." -ForegroundColor Red
    exit 1
} 