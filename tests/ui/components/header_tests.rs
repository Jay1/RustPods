#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::components::header::Header;

    #[test]
    fn test_header_creation() {
        // Create with scanning active
        let header = Header::new(true, true);
        assert!(header.is_scanning);
        assert!(header.auto_scan);
        // Create with scanning inactive
        let header = Header::new(false, false);
        assert!(!header.is_scanning);
        assert!(!header.auto_scan);
    }
} 