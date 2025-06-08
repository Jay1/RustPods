#[cfg(test)]
mod tests {
    use rustpods::error::ErrorContext;

    #[test]
    fn test_error_context_display() {
        // Create a basic context
        let ctx = ErrorContext::new("TestComponent", "testOperation");
        let display_str = ctx.to_string();

        // Verify format - should be "[Component:Operation] "
        assert_eq!(display_str, "[TestComponent:testOperation] ");

        // Add metadata and test again
        let ctx_with_metadata = ctx
            .with_metadata("key1", "value1")
            .with_metadata("key2", "value2");
        let display_str = ctx_with_metadata.to_string();

        // Check format with metadata - exact order may vary due to HashMap
        assert!(display_str.starts_with("[TestComponent:testOperation] ("));
        assert!(display_str.ends_with(") "));
        assert!(display_str.contains("key1=value1"));
        assert!(display_str.contains("key2=value2"));

        // The output is used in log statements
        println!("ErrorContext display: {}", ctx_with_metadata);
    }
}
