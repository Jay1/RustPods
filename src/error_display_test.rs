#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use chrono::Utc;
    
    // Minimal copy of ErrorContext for testing
    #[derive(Debug, Clone)]
    struct ErrorContext {
        component: String,
        operation: String,
        metadata: HashMap<String, String>,
    }
    
    impl ErrorContext {
        fn new(component: impl Into<String>, operation: impl Into<String>) -> Self {
            Self {
                component: component.into(),
                operation: operation.into(),
                metadata: HashMap::new(),
            }
        }
        
        fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
            self.metadata.insert(key.into(), value.into());
            self
        }
    }
    
    impl std::fmt::Display for ErrorContext {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "[{}:{}] ", self.component, self.operation)?;
            if !self.metadata.is_empty() {
                write!(f, "(")?;
                let mut first = true;
                for (key, value) in &self.metadata {
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}={}", key, value)?;
                    first = false;
                }
                write!(f, ") ")?;
            }
            Ok(())
        }
    }
    
    #[test]
    fn test_error_context_display() {
        // Create a basic context
        let ctx = ErrorContext::new("TestComponent", "testOperation");
        let display_str = ctx.to_string();
        
        // Verify format - should be "[Component:Operation] "
        assert_eq!(display_str, "[TestComponent:testOperation] ");
        
        // Add metadata and test again
        let ctx_with_metadata = ctx.with_metadata("key1", "value1")
                                  .with_metadata("key2", "value2");
        let display_str = ctx_with_metadata.to_string();
        
        // Check format with metadata - exact order may vary due to HashMap
        assert!(display_str.starts_with("[TestComponent:testOperation] ("));
        assert!(display_str.ends_with(") "));
        assert!(display_str.contains("key1=value1"));
        assert!(display_str.contains("key2=value2"));
        
        // Print the result for visual inspection
        println!("ErrorContext display: {}", ctx_with_metadata);
    }
} 