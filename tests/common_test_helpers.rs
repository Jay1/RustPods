// Common test utilities shared across all test modules
use std::time::Duration;
use tokio::time::timeout;
use std::future::Future;
use futures::Stream;

/// Helper function to run an async operation with timeout
pub async fn with_timeout<T, F>(duration_secs: u64, future: F) -> Result<T, &'static str> 
where
    F: Future<Output = T>,
{
    match timeout(Duration::from_secs(duration_secs), future).await {
        Ok(result) => Ok(result),
        Err(_) => Err("Operation timed out"),
    }
}

/// Helper to convert tokio receiver to stream for testing
pub fn receiver_to_stream<T>(rx: tokio::sync::mpsc::Receiver<T>) -> impl Stream<Item = T> {
    tokio_stream::wrappers::ReceiverStream::new(rx)
}

/// Utility function to wait for a specific duration
pub async fn wait_ms(ms: u64) {
    tokio::time::sleep(Duration::from_millis(ms)).await;
}

/// Sets up test environment variables if needed
pub fn setup_test_env() {
    std::env::set_var("RUST_BACKTRACE", "1");
    std::env::set_var("RUST_LOG", "debug");
}

/// Helper function to create a temporary test directory and clean it up after the test
#[cfg(test)]
pub fn with_temp_dir<F, R>(test_fn: F) -> R
where
    F: FnOnce(&std::path::Path) -> R,
{
    let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
    let result = test_fn(temp_dir.path());
    // Temp directory will be automatically cleaned up when it goes out of scope
    result
} 