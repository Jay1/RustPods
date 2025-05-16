#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::components::connection_status::ConnectionStatus;

    #[test]
    fn test_connection_status_text() {
        // Connected state
        let connected = ConnectionStatus::new(true, false);
        assert_eq!(connected.status_text(), "Connected");
        // Disconnected state
        let disconnected = ConnectionStatus::new(false, false);
        assert_eq!(disconnected.status_text(), "No device connected");
        // Scanning state (takes precedence over connected state)
        let scanning = ConnectionStatus::new(false, true);
        assert_eq!(scanning.status_text(), "Scanning for devices...");
        // Scanning while connected (scanning takes precedence)
        let scanning_connected = ConnectionStatus::new(true, true);
        assert_eq!(scanning_connected.status_text(), "Scanning for devices...");
    }

    #[test]
    fn test_animation_progress() {
        let status = ConnectionStatus::new(true, false)
            .with_animation_progress(0.5);
        assert_eq!(status.animation_progress, 0.5);
    }
} 