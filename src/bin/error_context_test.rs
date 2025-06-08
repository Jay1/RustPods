use std::collections::HashMap;
use std::fmt;

// Simple ErrorContext struct for testing
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

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

fn main() {
    // Create a basic context
    let ctx = ErrorContext::new("TestComponent", "testOperation");
    let display_str = ctx.to_string();

    // Verify format
    assert_eq!(display_str, "[TestComponent:testOperation] ");
    println!("Basic context: {}", display_str);

    // Add metadata and test again
    let ctx_with_metadata = ctx
        .with_metadata("key1", "value1")
        .with_metadata("key2", "value2");
    let display_str = ctx_with_metadata.to_string();

    // For HashMap, we can't predict exact order
    println!("Context with metadata: {}", display_str);
    assert!(display_str.starts_with("[TestComponent:testOperation] ("));
    assert!(display_str.ends_with(") "));
    assert!(display_str.contains("key1=value1"));
    assert!(display_str.contains("key2=value2"));

    println!("All tests passed!");
}
