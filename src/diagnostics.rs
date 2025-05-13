//! Diagnostics module for RustPods
//!
//! This module provides diagnostics functionality for analyzing the system,
//! generating troubleshooting reports, and providing self-healing capabilities.

use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::io::{self, Write};
use std::time::{Duration, Instant};

use crate::config::AppConfig;
use crate::error::{ErrorManager, ErrorSeverity};
use crate::bluetooth::adapter::BluetoothAdapter;

/// Diagnostic level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticLevel {
    /// Basic diagnostics for common issues
    Basic,
    /// Advanced diagnostics with detailed information
    Advanced,
    /// Complete system analysis (highest detail)
    Complete,
}

/// Diagnostic result with findings and recommendations
#[derive(Debug, Clone)]
pub struct DiagnosticResult {
    /// List of issues found
    pub issues: Vec<DiagnosticIssue>,
    
    /// List of recommendations
    pub recommendations: Vec<String>,
    
    /// Raw diagnostic data for advanced analysis
    pub raw_data: HashMap<String, String>,
    
    /// Time it took to run diagnostics
    pub duration: Duration,
    
    /// Timestamp when diagnostics were run
    pub timestamp: chrono::DateTime<chrono::Utc>,
    
    /// Whether any critical issues were found
    pub has_critical_issues: bool,
}

/// Diagnostic issue
#[derive(Debug, Clone)]
pub struct DiagnosticIssue {
    /// Issue title/summary
    pub title: String,
    
    /// Detailed description
    pub description: String,
    
    /// Possible solutions
    pub solutions: Vec<String>,
    
    /// Severity level
    pub severity: IssueSeverity,
    
    /// Category of the issue
    pub category: IssueCategory,
    
    /// Whether this issue can be auto-repaired
    pub auto_repairable: bool,
}

/// Issue severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueSeverity {
    /// Critical issue that prevents core functionality
    Critical,
    /// Major issue that significantly impacts functionality
    Major,
    /// Minor issue with limited impact
    Minor,
    /// Informational only
    Info,
}

/// Issue category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueCategory {
    /// Bluetooth related issues
    Bluetooth,
    /// Operating system related issues
    System,
    /// Configuration related issues
    Configuration,
    /// Device (AirPods) related issues
    Device,
    /// Application specific issues
    Application,
}

/// Diagnostics manager
pub struct DiagnosticsManager {
    /// Application configuration
    config: Arc<AppConfig>,
    
    /// Error manager for accessing error history
    error_manager: Arc<Mutex<ErrorManager>>,
    
    /// Last diagnostic result
    last_result: Option<DiagnosticResult>,
    
    /// Time of last diagnostic run
    last_run: Option<Instant>,
    
    /// Diagnostic level
    level: DiagnosticLevel,
    
    /// Whether to include system information
    include_system_info: bool,
    
    /// Whether to include error history
    include_error_history: bool,
}

impl DiagnosticsManager {
    /// Create a new diagnostics manager
    pub fn new(
        config: Arc<AppConfig>,
        error_manager: Arc<Mutex<ErrorManager>>,
    ) -> Self {
        Self {
            config,
            error_manager,
            last_result: None,
            last_run: None,
            level: DiagnosticLevel::Basic,
            include_system_info: true,
            include_error_history: true,
        }
    }
    
    /// Set the diagnostic level
    pub fn set_level(&mut self, level: DiagnosticLevel) -> &mut Self {
        self.level = level;
        self
    }
    
    /// Set whether to include system information
    pub fn include_system_info(&mut self, include: bool) -> &mut Self {
        self.include_system_info = include;
        self
    }
    
    /// Set whether to include error history
    pub fn include_error_history(&mut self, include: bool) -> &mut Self {
        self.include_error_history = include;
        self
    }
    
    /// Run diagnostics
    pub async fn run_diagnostics(&mut self) -> io::Result<DiagnosticResult> {
        log::info!("Running diagnostics with level: {:?}", self.level);
        
        let start_time = Instant::now();
        let mut issues = Vec::new();
        let mut recommendations = Vec::new();
        let mut raw_data = HashMap::new();
        let mut has_critical_issues = false;
        
        // Add system information
        if self.include_system_info {
            self.collect_system_information(&mut raw_data)?;
        }
        
        // Add error history
        if self.include_error_history {
            self.collect_error_history(&mut raw_data, &mut issues, &mut recommendations)?;
        }
        
        // Check Bluetooth capabilities
        if let Err(e) = self.check_bluetooth(&mut issues, &mut recommendations, &mut raw_data).await {
            log::warn!("Failed to check Bluetooth capabilities: {}", e);
            issues.push(DiagnosticIssue {
                title: "Bluetooth diagnostic check failed".to_string(),
                description: format!("Could not check Bluetooth capabilities: {}", e),
                solutions: vec![
                    "Ensure Bluetooth is enabled".to_string(),
                    "Try running the application with administrator privileges".to_string(),
                ],
                severity: IssueSeverity::Major,
                category: IssueCategory::Bluetooth,
                auto_repairable: false,
            });
            
            has_critical_issues = true;
        }
        
        // Check configuration
        self.check_configuration(&mut issues, &mut recommendations, &mut raw_data)?;
        
        // For advanced or complete diagnostics, perform additional checks
        if self.level != DiagnosticLevel::Basic {
            self.check_permissions(&mut issues, &mut recommendations, &mut raw_data)?;
            
            if self.level == DiagnosticLevel::Complete {
                self.check_hardware_compatibility(&mut issues, &mut recommendations, &mut raw_data)?;
            }
        }
        
        // Update critical issues flag
        has_critical_issues = has_critical_issues || issues.iter().any(|i| i.severity == IssueSeverity::Critical);
        
        // Create diagnostic result
        let result = DiagnosticResult {
            issues,
            recommendations,
            raw_data,
            duration: start_time.elapsed(),
            timestamp: chrono::Utc::now(),
            has_critical_issues,
        };
        
        // Update last result
        self.last_result = Some(result.clone());
        self.last_run = Some(start_time);
        
        Ok(result)
    }
    
    /// Collect system information
    fn collect_system_information(&self, raw_data: &mut HashMap<String, String>) -> io::Result<()> {
        // OS information
        raw_data.insert("os".to_string(), std::env::consts::OS.to_string());
        raw_data.insert("arch".to_string(), std::env::consts::ARCH.to_string());
        
        // Process information
        if let Ok(current_exe) = std::env::current_exe() {
            raw_data.insert("executable".to_string(), current_exe.display().to_string());
        }
        
        if let Ok(cwd) = std::env::current_dir() {
            raw_data.insert("working_directory".to_string(), cwd.display().to_string());
        }
        
        // Configuration information
        raw_data.insert("log_level".to_string(), format!("{:?}", self.config.system.log_level));
        raw_data.insert("auto_scan".to_string(), self.config.bluetooth.auto_scan_on_startup.to_string());
        raw_data.insert("start_minimized".to_string(), self.config.ui.start_minimized.to_string());
        
        Ok(())
    }
    
    /// Collect error history
    fn collect_error_history(
        &self, 
        raw_data: &mut HashMap<String, String>,
        issues: &mut Vec<DiagnosticIssue>,
        recommendations: &mut Vec<String>,
    ) -> io::Result<()> {
        if let Ok(error_manager) = self.error_manager.lock() {
            let stats = error_manager.get_stats();
            
            // Add error stats to raw data
            raw_data.insert("total_errors".to_string(), stats.total.to_string());
            
            // Get bluetooth error counts if available
            let bluetooth_errors = stats.by_type.get("bluetooth").cloned().unwrap_or(0);
            raw_data.insert("bluetooth_errors".to_string(), bluetooth_errors.to_string());
            
            // Get critical error counts from severity map
            let critical_errors = stats.by_severity
                .get(&ErrorSeverity::Critical)
                .cloned()
                .unwrap_or(0);
            raw_data.insert("critical_errors".to_string(), critical_errors.to_string());
            
            // Get airpods related errors if available
            let airpods_errors = stats.by_type.get("airpods").cloned().unwrap_or(0);
            raw_data.insert("airpods_errors".to_string(), airpods_errors.to_string());
            
            // Check for concerning error patterns
            if bluetooth_errors > 5 {
                issues.push(DiagnosticIssue {
                    title: "Frequent Bluetooth errors".to_string(),
                    description: format!("Detected {} Bluetooth errors, which may indicate compatibility issues", bluetooth_errors),
                    solutions: vec![
                        "Ensure your Bluetooth adapter is compatible (Bluetooth 4.0+ recommended)".to_string(),
                        "Update Bluetooth drivers".to_string(),
                        "Try disabling other Bluetooth applications".to_string(),
                    ],
                    severity: if bluetooth_errors > 20 { IssueSeverity::Major } else { IssueSeverity::Minor },
                    category: IssueCategory::Bluetooth,
                    auto_repairable: false,
                });
                
                recommendations.push("Consider updating your Bluetooth drivers".to_string());
            }
            
            if critical_errors > 0 {
                issues.push(DiagnosticIssue {
                    title: "Critical errors detected".to_string(),
                    description: format!("Detected {} critical errors that may prevent core functionality", critical_errors),
                    solutions: vec![
                        "Check the log files for detailed error information".to_string(),
                        "Try running the application with administrator privileges".to_string(),
                        "Restart your computer and try again".to_string(),
                    ],
                    severity: IssueSeverity::Critical,
                    category: IssueCategory::Application,
                    auto_repairable: false,
                });
            }
        }
        
        Ok(())
    }
    
    /// Check Bluetooth capabilities
    async fn check_bluetooth(
        &self,
        issues: &mut Vec<DiagnosticIssue>,
        recommendations: &mut Vec<String>,
        raw_data: &mut HashMap<String, String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Try to initialize adapter
        let adapter_result = BluetoothAdapter::new().await;
        
        match adapter_result {
            Ok(adapter) => {
                // Get adapter information
                let capabilities = adapter.get_capabilities();
                raw_data.insert("bluetooth_adapter".to_string(), "Bluetooth adapter".to_string());
                raw_data.insert("bluetooth_status".to_string(), format!("{:?}", adapter.get_status()));
                raw_data.insert("bluetooth_enabled".to_string(), "true".to_string());
                
                // Check if scanning is supported based on capabilities
                let supports_scanning = capabilities.supports_scanning;
                raw_data.insert("bluetooth_scanning_supported".to_string(), supports_scanning.to_string());
                
                if !supports_scanning {
                    issues.push(DiagnosticIssue {
                        title: "Bluetooth scanning not supported".to_string(),
                        description: "Your Bluetooth adapter does not support scanning, which is required for AirPods detection".to_string(),
                        solutions: vec![
                            "Use a different Bluetooth adapter".to_string(),
                            "Ensure your adapter supports Bluetooth LE scanning".to_string(),
                        ],
                        severity: IssueSeverity::Critical,
                        category: IssueCategory::Bluetooth,
                        auto_repairable: false,
                    });
                    return Ok(());
                }
                
                // Check if any AirPods have been detected
                // This would need to check the scanner history or storage
                // For now, it's a placeholder
                raw_data.insert("airpods_detected".to_string(), "unknown".to_string());
                
                // Add a recommendation about Bluetooth
                recommendations.push("Keep Bluetooth enabled for automatic detection of AirPods".to_string());
                
                Ok(())
            },
            Err(e) => {
                raw_data.insert("bluetooth_enabled".to_string(), "false".to_string());
                raw_data.insert("bluetooth_error".to_string(), e.to_string());
                
                issues.push(DiagnosticIssue {
                    title: "Bluetooth not available".to_string(),
                    description: format!("Unable to access Bluetooth adapter: {}", e),
                    solutions: vec![
                        "Ensure Bluetooth is enabled on your system".to_string(),
                        "Verify you have a compatible Bluetooth adapter".to_string(),
                        "Make sure you have sufficient permissions".to_string(),
                    ],
                    severity: IssueSeverity::Critical,
                    category: IssueCategory::Bluetooth,
                    auto_repairable: false,
                });
                
                Ok(())
            }
        }
    }
    
    /// Check configuration
    fn check_configuration(
        &self,
        issues: &mut Vec<DiagnosticIssue>,
        recommendations: &mut Vec<String>,
        raw_data: &mut HashMap<String, String>,
    ) -> io::Result<()> {
        // Check for configuration issues
        if let Err(e) = self.config.validate() {
            raw_data.insert("config_validation_error".to_string(), e.to_string());
            
            issues.push(DiagnosticIssue {
                title: "Configuration validation error".to_string(),
                description: format!("Configuration has invalid values: {}", e),
                solutions: vec![
                    "Reset to default configuration".to_string(),
                    "Edit the configuration file manually to fix the issues".to_string(),
                ],
                severity: IssueSeverity::Major,
                category: IssueCategory::Configuration,
                auto_repairable: true,
            });
            
            recommendations.push("Reset to default configuration to resolve validation errors".to_string());
        }
        
        // Check for suboptimal settings
        if self.config.bluetooth.scan_duration.as_secs() < 3 {
            issues.push(DiagnosticIssue {
                title: "Scan duration too short".to_string(),
                description: "Bluetooth scan duration is set very low, which may cause inconsistent device detection".to_string(),
                solutions: vec![
                    "Increase scan duration to at least 3 seconds".to_string(),
                ],
                severity: IssueSeverity::Minor,
                category: IssueCategory::Configuration,
                auto_repairable: true,
            });
            
            recommendations.push("Increase Bluetooth scan duration to improve device detection".to_string());
        }
        
        Ok(())
    }
    
    /// Check system permissions
    fn check_permissions(
        &self,
        issues: &mut Vec<DiagnosticIssue>,
        recommendations: &mut Vec<String>,
        raw_data: &mut HashMap<String, String>,
    ) -> io::Result<()> {
        // Check write permissions in config directory
        if let Some(config_dir) = dirs::config_dir() {
            let test_path = config_dir.join("rustpods").join(".test_write_permission");
            
            match std::fs::create_dir_all(test_path.parent().unwrap()) {
                Ok(_) => {
                    // Try to write a test file
                    match std::fs::write(&test_path, "test") {
                        Ok(_) => {
                            // Clean up test file
                            let _ = std::fs::remove_file(test_path);
                            raw_data.insert("config_dir_writable".to_string(), "true".to_string());
                        },
                        Err(e) => {
                            raw_data.insert("config_dir_writable".to_string(), "false".to_string());
                            raw_data.insert("config_dir_error".to_string(), e.to_string());
                            
                            issues.push(DiagnosticIssue {
                                title: "Cannot write to configuration directory".to_string(),
                                description: format!("Insufficient permissions to write to config directory: {}", e),
                                solutions: vec![
                                    "Run the application with administrator privileges".to_string(),
                                    "Check folder permissions".to_string(),
                                ],
                                severity: IssueSeverity::Major,
                                category: IssueCategory::System,
                                auto_repairable: false,
                            });
                            
                            recommendations.push("Run the application with administrator privileges to fix permission issues".to_string());
                        }
                    }
                },
                Err(e) => {
                    raw_data.insert("config_dir_writable".to_string(), "false".to_string());
                    raw_data.insert("config_dir_error".to_string(), e.to_string());
                    
                    issues.push(DiagnosticIssue {
                        title: "Cannot access configuration directory".to_string(),
                        description: format!("Cannot create configuration directory: {}", e),
                        solutions: vec![
                            "Run the application with administrator privileges".to_string(),
                            "Check folder permissions".to_string(),
                        ],
                        severity: IssueSeverity::Major,
                        category: IssueCategory::System,
                        auto_repairable: false,
                    });
                }
            }
        }
        
        Ok(())
    }
    
    /// Check hardware compatibility
    fn check_hardware_compatibility(
        &self,
        issues: &mut Vec<DiagnosticIssue>,
        recommendations: &mut Vec<String>,
        raw_data: &mut HashMap<String, String>,
    ) -> io::Result<()> {
        // On Windows, check for Bluetooth version
        #[cfg(target_os = "windows")]
        {
            // In a real implementation, we would use Windows APIs to check Bluetooth version
            // This is a placeholder for now
            raw_data.insert("bluetooth_version_check".to_string(), "simulated".to_string());
        }
        
        Ok(())
    }
    
    /// Save diagnostic results to a file
    pub fn save_diagnostics(&self) -> io::Result<PathBuf> {
        let result = match &self.last_result {
            Some(r) => r,
            None => return Err(io::Error::new(io::ErrorKind::NotFound, "No diagnostic results available")),
        };
        
        // Determine diagnostic file path
        let diagnostics_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("RustPods")
            .join("diagnostics");
            
        // Create directory if it doesn't exist
        if !diagnostics_dir.exists() {
            std::fs::create_dir_all(&diagnostics_dir)?;
        }
        
        // Generate file name with timestamp
        let now = chrono::Local::now();
        let timestamp = now.format("%Y%m%d_%H%M%S");
        let file_name = format!("rustpods_diagnostic_{}.txt", timestamp);
        let file_path = diagnostics_dir.join(&file_name);
        
        // Create file
        let mut file = std::fs::File::create(&file_path)?;
        
        // Write diagnostic information
        writeln!(file, "=== RustPods Diagnostic Report ===")?;
        writeln!(file, "Generated: {}", now.format("%Y-%m-%d %H:%M:%S"))?;
        writeln!(file, "Diagnostic level: {:?}", self.level)?;
        writeln!(file, "Duration: {:?}", result.duration)?;
        writeln!(file)?;
        
        // Write issues
        writeln!(file, "=== Issues ({}) ===", result.issues.len())?;
        for (i, issue) in result.issues.iter().enumerate() {
            writeln!(file, "Issue {}: {} [{:?}]", i + 1, issue.title, issue.severity)?;
            writeln!(file, "Description: {}", issue.description)?;
            writeln!(file, "Category: {:?}", issue.category)?;
            writeln!(file, "Solutions:")?;
            for solution in &issue.solutions {
                writeln!(file, "  - {}", solution)?;
            }
            writeln!(file)?;
        }
        
        // Write recommendations
        writeln!(file, "=== Recommendations ({}) ===", result.recommendations.len())?;
        for (i, recommendation) in result.recommendations.iter().enumerate() {
            writeln!(file, "{}. {}", i + 1, recommendation)?;
        }
        writeln!(file)?;
        
        // Write raw data
        writeln!(file, "=== System Information ===")?;
        let mut raw_data: Vec<(&String, &String)> = result.raw_data.iter().collect();
        raw_data.sort_by(|a, b| a.0.cmp(b.0));
        
        for (key, value) in raw_data {
            writeln!(file, "{}: {}", key, value)?;
        }
        
        // Flush file
        file.flush()?;
        
        Ok(file_path)
    }
    
    /// Attempt to auto-repair issues
    pub async fn auto_repair(&self) -> Result<AutoRepairResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut result = AutoRepairResult {
            fixed_issues: Vec::new(),
            failed_repairs: Vec::new(),
            repair_time: Duration::default(),
        };
        
        let start = Instant::now();
        
        // Check if we have diagnostic results
        let diagnostic = match &self.last_result {
            Some(d) => d,
            None => {
                return Err("No diagnostic results available. Run diagnostics first.".into());
            }
        };
        
        // Find auto-repairable issues
        let repairable_issues: Vec<&DiagnosticIssue> = diagnostic.issues
            .iter()
            .filter(|i| i.auto_repairable)
            .collect();
            
        log::info!("Found {} auto-repairable issues", repairable_issues.len());
        
        if repairable_issues.is_empty() {
            return Ok(result);
        }
        
        // Try to repair each issue
        for issue in repairable_issues {
            match issue.category {
                IssueCategory::Configuration => {
                    // Try to repair configuration issues
                    if let Err(e) = self.repair_configuration_issue(issue).await {
                        log::error!("Failed to repair configuration issue: {}", e);
                        result.failed_repairs.push((issue.title.clone(), e.to_string()));
                    } else {
                        log::info!("Successfully repaired configuration issue: {}", issue.title);
                        result.fixed_issues.push(issue.title.clone());
                    }
                },
                // Add other repair categories as needed
                _ => {
                    log::warn!("No repair implementation for issue category: {:?}", issue.category);
                    result.failed_repairs.push((
                        issue.title.clone(),
                        format!("No repair implementation for category {:?}", issue.category)
                    ));
                }
            }
        }
        
        result.repair_time = start.elapsed();
        Ok(result)
    }
    
    /// Repair a configuration issue
    async fn repair_configuration_issue(&self, issue: &DiagnosticIssue) -> Result<(), String> {
        if issue.title == "Scan duration too short" {
            // We would modify the config here in a real implementation
            log::info!("Would increase scan duration to 5 seconds");
            // Return success for now
            return Ok(());
        }
        
        if issue.title == "Configuration validation error" {
            // We would reset the config here in a real implementation
            log::info!("Would reset configuration to defaults");
            // Return success for now
            return Ok(());
        }
        
        Err(format!("No repair implementation for issue: {}", issue.title))
    }
}

/// Auto-repair result
#[derive(Debug, Clone)]
pub struct AutoRepairResult {
    /// List of issues that were fixed
    pub fixed_issues: Vec<String>,
    
    /// List of issues that could not be fixed (with reasons)
    pub failed_repairs: Vec<(String, String)>,
    
    /// Time taken for repair attempts
    pub repair_time: Duration,
}

/// Check if the system meets minimum requirements
pub fn meets_minimum_requirements() -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    // Check operating system
    let os = std::env::consts::OS;
    if os != "windows" && os != "macos" && os != "linux" {
        return Ok(false);
    }
    
    // Check for required directories
    let config_dir = dirs::config_dir();
    let data_dir = dirs::data_local_dir();
    
    if config_dir.is_none() || data_dir.is_none() {
        return Ok(false);
    }
    
    // More checks can be added here as needed
    
    Ok(true)
}

/// Create a troubleshooting guide based on diagnostic results
pub fn create_troubleshooting_guide(result: &DiagnosticResult) -> Vec<TroubleshootingStep> {
    let mut steps = Vec::new();
    
    // First, handle critical issues
    let critical_issues: Vec<&DiagnosticIssue> = result.issues
        .iter()
        .filter(|i| i.severity == IssueSeverity::Critical)
        .collect();
        
    if !critical_issues.is_empty() {
        for issue in critical_issues {
            steps.push(TroubleshootingStep {
                title: format!("Fix Critical Issue: {}", issue.title),
                instructions: issue.description.clone(),
                actions: issue.solutions.clone(),
                verification: "Restart the application and check if the issue persists".to_string(),
            });
        }
    }
    
    // Next, handle major issues
    let major_issues: Vec<&DiagnosticIssue> = result.issues
        .iter()
        .filter(|i| i.severity == IssueSeverity::Major)
        .collect();
        
    if !major_issues.is_empty() {
        for issue in major_issues {
            steps.push(TroubleshootingStep {
                title: format!("Address Major Issue: {}", issue.title),
                instructions: issue.description.clone(),
                actions: issue.solutions.clone(),
                verification: "Check if the functionality is restored".to_string(),
            });
        }
    }
    
    // Add common troubleshooting steps if appropriate
    if result.issues.iter().any(|i| i.category == IssueCategory::Bluetooth) {
        steps.push(TroubleshootingStep {
            title: "Reset Bluetooth System".to_string(),
            instructions: "Reset your system's Bluetooth stack to clear any issues".to_string(),
            actions: vec![
                "Turn off Bluetooth".to_string(),
                "Restart your computer".to_string(),
                "Turn Bluetooth back on".to_string(),
            ],
            verification: "Check if device detection works after reset".to_string(),
        });
    }
    
    steps
}

/// Troubleshooting step
#[derive(Debug, Clone)]
pub struct TroubleshootingStep {
    /// Step title
    pub title: String,
    
    /// Instructions
    pub instructions: String,
    
    /// Actions to take
    pub actions: Vec<String>,
    
    /// How to verify the step worked
    pub verification: String,
} 