// Common test utilities shared across all test modules
use std::time::Duration;
use tokio::time::timeout;
use std::future::Future;
use futures::Stream;
use tokio::sync::mpsc::Receiver;
use tokio::time;
use futures::{StreamExt};
use std::pin::Pin;
use tokio_stream::wrappers::ReceiverStream;

/// Helper to convert tokio receiver to stream for testing
pub fn receiver_to_stream<T>(rx: Receiver<T>) -> impl Stream<Item = T> {
    futures::stream::unfold(rx, |mut rx| async move {
        match rx.recv().await {
            Some(value) => Some((value, rx)),
            None => None,
        }
    })
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

/// Execute an async operation with a timeout
pub async fn with_timeout<T>(duration: Duration, operation: impl std::future::Future<Output = T>) -> T {
    match time::timeout(duration, operation).await {
        Ok(result) => result,
        Err(_) => panic!("Operation timed out after {:?}", duration),
    }
}

/// Create a helper to wait for events with a default timeout
pub async fn wait_for_event<S, T>(stream: &mut S) -> Option<T> 
where
    S: StreamExt<Item = T> + Unpin,
{
    with_timeout(Duration::from_millis(500), stream.next()).await
}

/// Create a helper to assert that no more events are received
pub async fn assert_no_more_events<S, T>(stream: &mut S, timeout_ms: u64) 
where
    S: StreamExt<Item = T> + Unpin,
{
    let result = time::timeout(Duration::from_millis(timeout_ms), stream.next()).await;
    assert!(result.is_err(), "Should not receive any more events");
}

/// Sleep for a short period to allow async operations to complete
pub async fn short_delay() {
    wait_ms(100).await;
}

/// Sleep for a longer period when dealing with complex event chains
pub async fn medium_delay() {
    wait_ms(250).await;
}

/// Sleep for a long period to allow for significant operations
pub async fn long_delay() {
    wait_ms(500).await;
}

/// Sleep for an extra long period to allow for substantial operations
pub async fn very_long_delay() {
    wait_ms(1000).await;
}

/// Helper to create temporary test directory
pub fn create_temp_dir() -> tempfile::TempDir {
    let result = tempfile::tempdir().expect("Failed to create temporary directory");
    
    // Temp directory will be automatically cleaned up when it goes out of scope
    result
}

/// Convert a tokio mpsc Receiver into a Stream
/// This makes it easier to work with receivers in async code using Stream combinators
pub fn receiver_to_stream_boxed<T: Send + 'static>(receiver: Receiver<T>) -> Pin<Box<dyn Stream<Item = T> + Send>> {
    Box::pin(ReceiverStream::new(receiver))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc::channel;
    
    #[tokio::test]
    async fn test_receiver_to_stream() {
        let (tx, rx) = channel::<i32>(10);
        let stream = receiver_to_stream(rx);
        tokio::pin!(stream);
        
        tx.send(42).await.unwrap();
        
        let result = wait_for_event(&mut stream).await;
        assert_eq!(result, Some(42));
    }
    
    #[tokio::test]
    async fn test_with_timeout_success() {
        let result = with_timeout(Duration::from_millis(100), async { 42 }).await;
        assert_eq!(result, 42);
    }
    
    #[tokio::test]
    #[should_panic(expected = "Operation timed out")]
    async fn test_with_timeout_failure() {
        let _ = with_timeout(Duration::from_millis(10), async { 
            time::sleep(Duration::from_millis(100)).await;
            42
        }).await;
    }
} 