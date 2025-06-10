//! Build script for RustPods
//!
//! This script automatically builds the native CLI scanner during compilation,
//! ensuring the project is fully self-contained and buildable from source.
//! Optimized to only rebuild when source files have actually changed.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::path::Path;

#[cfg(target_os = "windows")]
extern crate winres;

fn main() {
    // Compile Windows resource file to embed icon and version info
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icons/app/logo_ring.ico")
           .set_version_info(winres::VersionInfo::PRODUCTVERSION, 0x0001000000000000)
           .set_manifest_file("app.rc");
        if let Err(e) = res.compile() {
            println!("cargo:warning=Failed to compile Windows resources: {}", e);
        }
    }

    // Only build CLI scanner on Windows (target functionality)
    if cfg!(target_os = "windows") {
        build_cli_scanner();
    } else {
        println!("cargo:warning=CLI scanner only available on Windows - skipping build");
    }

    // Setup automatic CLI scanner copying
    setup_cli_scanner_distribution();
}

fn build_cli_scanner() {
    // More comprehensive dependency tracking - watch specific source files (v6 modular)
    println!("cargo:rerun-if-changed=scripts/airpods_battery_cli/CMakeLists.txt");
    println!("cargo:rerun-if-changed=scripts/airpods_battery_cli/Source/");
    println!("cargo:rerun-if-changed=third_party/spdlog/include/");
    println!("cargo:rerun-if-changed=third_party/spdlog/src/");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let cli_source_dir = PathBuf::from(&manifest_dir)
        .join("scripts")
        .join("airpods_battery_cli");

    let cli_build_dir = cli_source_dir.join("build");

    // Determine build configuration
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let cmake_build_type = if profile == "release" {
        "Release"
    } else {
        "Debug"
    };

    // Check if v6 modular executable exists and is up-to-date
    let exe_path = cli_build_dir
        .join(cmake_build_type)
        .join("airpods_battery_cli.exe");

    let exe_exists = exe_path.exists();
    let exe_path_to_check = &exe_path;

    if exe_exists {
        // Check if we need to rebuild by comparing timestamps
        if let Ok(exe_metadata) = fs::metadata(exe_path_to_check) {
            if let Ok(exe_modified) = exe_metadata.modified() {
                let mut needs_rebuild = false;

                // Check if any source files are newer than the executable
                let source_paths = [
                    cli_source_dir.join("CMakeLists.txt"),
                    cli_source_dir.join("Source"),
                    PathBuf::from(&manifest_dir)
                        .join("third_party")
                        .join("spdlog")
                        .join("include"),
                ];

                for source_path in &source_paths {
                    if let Ok(newer) = is_path_newer_than(source_path, exe_modified) {
                        if newer {
                            needs_rebuild = true;
                            break;
                        }
                    }
                }

                if !needs_rebuild {
                    println!("cargo:warning=CLI scanner is up-to-date, skipping build");
                    // Still set the environment variables
                    println!(
                        "cargo:rustc-env=CLI_SCANNER_PATH={}",
                        exe_path_to_check.display()
                    );
                    println!("cargo:rustc-env=CLI_SCANNER_AVAILABLE=true");
                    println!("cargo:rustc-env=CLI_BUILD_TYPE={}", cmake_build_type);
                    println!(
                        "cargo:rustc-env=CLI_SOURCE_DIR={}",
                        cli_source_dir.display()
                    );
                    println!("cargo:rustc-env=CLI_BUILD_DIR={}", cli_build_dir.display());
                    return;
                }
            }
        }
    }

    println!("cargo:warning=Building AirPods CLI Scanner...");

    // Initialize submodules if needed (only if spdlog directory doesn't exist)
    let spdlog_include = PathBuf::from(&manifest_dir)
        .join("third_party")
        .join("spdlog")
        .join("include");
    if !spdlog_include.exists() {
        println!("cargo:warning=Initializing Git submodules...");
        let status = Command::new("git")
            .current_dir(&manifest_dir)
            .args(["submodule", "update", "--init", "--recursive"])
            .status();

        match status {
            Ok(status) if status.success() => {
                println!("cargo:warning=Git submodules initialized successfully");
            }
            Ok(_) => {
                println!("cargo:warning=Git submodule initialization failed - continuing anyway");
            }
            Err(_) => {
                println!(
                    "cargo:warning=Git not found - assuming submodules are already initialized"
                );
            }
        }
    }

    // Only run CMake configure if build directory doesn't exist or CMakeCache.txt is missing
    let cmake_cache = cli_build_dir.join("CMakeCache.txt");
    if !cmake_cache.exists() {
        println!("cargo:warning=Configuring CLI scanner with CMake...");
        let cmake_configure = Command::new("cmake")
            .current_dir(&cli_source_dir)
            .args([
                "-B",
                "build",
                "-S",
                ".",
                "-G",
                "Visual Studio 17 2022",
                "-A",
                "x64",
                &format!("-DCMAKE_BUILD_TYPE={}", cmake_build_type),
            ])
            .status();

        match cmake_configure {
            Ok(status) if status.success() => {
                println!("cargo:warning=CMake configuration successful");
            }
            Ok(_) => {
                // Try alternative CMake generator
                println!("cargo:warning=Visual Studio 2022 not found, trying 2019...");
                let cmake_configure_alt = Command::new("cmake")
                    .current_dir(&cli_source_dir)
                    .args([
                        "-B",
                        "build",
                        "-S",
                        ".",
                        "-G",
                        "Visual Studio 16 2019",
                        "-A",
                        "x64",
                        &format!("-DCMAKE_BUILD_TYPE={}", cmake_build_type),
                    ])
                    .status();

                match cmake_configure_alt {
                    Ok(status) if status.success() => {
                        println!("cargo:warning=CMake configuration successful with VS2019");
                    }
                    _ => {
                        // Try Ninja as fallback
                        println!("cargo:warning=Visual Studio generators failed, trying Ninja...");
                        let cmake_ninja = Command::new("cmake")
                            .current_dir(&cli_source_dir)
                            .args([
                                "-B",
                                "build",
                                "-S",
                                ".",
                                "-G",
                                "Ninja",
                                &format!("-DCMAKE_BUILD_TYPE={}", cmake_build_type),
                            ])
                            .status();

                        if cmake_ninja.is_err() || !cmake_ninja.unwrap().success() {
                            println!("cargo:warning=CMake configuration failed - CLI scanner will not be available");
                            println!("cargo:warning=Please install Visual Studio 2019/2022 with C++ workload or CMake with Ninja");
                            return;
                        }
                    }
                }
            }
            Err(_) => {
                println!("cargo:warning=CMake not found - CLI scanner will not be available");
                println!("cargo:warning=Please install CMake to enable CLI scanner build");
                return;
            }
        }
    } else {
        println!("cargo:warning=CMake already configured, using existing build directory");
    }

    // Run CMake build
    println!(
        "cargo:warning=Building CLI scanner ({} mode)...",
        cmake_build_type
    );
    let cmake_build = Command::new("cmake")
        .current_dir(&cli_source_dir)
        .args(["--build", "build", "--config", cmake_build_type])
        .status();

    match cmake_build {
        Ok(status) if status.success() => {
            println!("cargo:warning=CLI scanner built successfully");

            // Verify the executable exists
            if exe_path.exists() {
                println!(
                    "cargo:warning=CLI scanner executable: {}",
                    exe_path.display()
                );

                // Tell Cargo about the output
                println!("cargo:rustc-env=CLI_SCANNER_PATH={}", exe_path.display());

                // Set environment variable for runtime
                println!("cargo:rustc-env=CLI_SCANNER_AVAILABLE=true");
            } else {
                println!("cargo:warning=CLI scanner executable not found at expected location");
                println!("cargo:rustc-env=CLI_SCANNER_AVAILABLE=false");
            }
        }
        Ok(_) => {
            println!("cargo:warning=CLI scanner build failed");
            println!("cargo:rustc-env=CLI_SCANNER_AVAILABLE=false");
        }
        Err(e) => {
            println!("cargo:warning=CLI scanner build error: {}", e);
            println!("cargo:rustc-env=CLI_SCANNER_AVAILABLE=false");
        }
    }

    // Output build information
    println!("cargo:rustc-env=CLI_BUILD_TYPE={}", cmake_build_type);
    println!(
        "cargo:rustc-env=CLI_SOURCE_DIR={}",
        cli_source_dir.display()
    );
    println!("cargo:rustc-env=CLI_BUILD_DIR={}", cli_build_dir.display());
}

/// Check if a path (file or directory) is newer than the given timestamp
fn is_path_newer_than(
    path: &PathBuf,
    target_time: std::time::SystemTime,
) -> Result<bool, std::io::Error> {
    if path.is_file() {
        let metadata = fs::metadata(path)?;
        let modified = metadata.modified()?;
        Ok(modified > target_time)
    } else if path.is_dir() {
        // For directories, check all files recursively
        let entries = fs::read_dir(path)?;
        for entry in entries {
            let entry = entry?;
            let entry_path = entry.path();
            if is_path_newer_than(&entry_path, target_time)? {
                return Ok(true);
            }
        }
        Ok(false)
    } else {
        Ok(false) // Path doesn't exist
    }
}

fn setup_cli_scanner_distribution() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    
    // Source CLI scanner path
    let cli_scanner_path = Path::new(&manifest_dir)
        .join("scripts")
        .join("airpods_battery_cli")
        .join("build")
        .join("Release")
        .join("airpods_battery_cli.exe");
    
    println!("cargo:rerun-if-changed=scripts/airpods_battery_cli/build/Release/airpods_battery_cli.exe");
    
    if cli_scanner_path.exists() {
        // Create bin directory in project root
        let bin_dir = Path::new(&manifest_dir).join("bin");
        if !bin_dir.exists() {
            std::fs::create_dir_all(&bin_dir).unwrap_or_else(|e| {
                println!("cargo:warning=Failed to create bin directory: {}", e);
            });
        }
        
        // Copy CLI scanner to bin/
        let bin_target = bin_dir.join("airpods_battery_cli.exe");
        if let Err(e) = std::fs::copy(&cli_scanner_path, &bin_target) {
            println!("cargo:warning=Failed to copy CLI scanner to bin/: {}", e);
        } else {
            println!("cargo:warning=CLI scanner copied to bin/airpods_battery_cli.exe");
        }
        
        // Also copy to target/release for direct distribution
        if let Ok(target_dir) = env::var("CARGO_TARGET_DIR") {
            let release_dir = Path::new(&target_dir).join("release");
            if release_dir.exists() {
                let release_target = release_dir.join("airpods_battery_cli.exe");
                if let Err(e) = std::fs::copy(&cli_scanner_path, &release_target) {
                    println!("cargo:warning=Failed to copy CLI scanner to target/release: {}", e);
                } else {
                    println!("cargo:warning=CLI scanner copied to target/release/airpods_battery_cli.exe");
                }
            }
        } else {
            // Fallback to default target directory
            let default_target = Path::new(&manifest_dir).join("target").join("release");
            if default_target.exists() {
                let release_target = default_target.join("airpods_battery_cli.exe");
                if let Err(e) = std::fs::copy(&cli_scanner_path, &release_target) {
                    println!("cargo:warning=Failed to copy CLI scanner to target/release: {}", e);
                } else {
                    println!("cargo:warning=CLI scanner copied to target/release/airpods_battery_cli.exe");
                }
            }
        }
    } else {
        println!("cargo:warning=CLI scanner not found at expected path: {}", cli_scanner_path.display());
        println!("cargo:warning=Make sure to build the CLI scanner first with: cmake --build scripts/airpods_battery_cli/build --config Release");
    }
}
