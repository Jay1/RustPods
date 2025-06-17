# RustPods System Tray Integration: Technical Architecture and Implementation Protocol

## Preamble: Document Scope and Relevance

This document defines the technical architecture, implementation strategy, and operational protocols for system tray integration and minimize-to-tray functionality in RustPods. It is intended for engineers responsible for maintaining or extending window management, event loop preservation, and user interface responsiveness. All recommendations and examples are current as of the latest system architecture. Legacy or deprecated approaches are clearly marked.

## 1. Problem Statement and Root Cause Analysis

### 1.1 Initial Issue
- Minimize-to-tray functionality caused the window content to disappear, but restore operations failed.
- Users were unable to restore the application from the tray and had to force-terminate the process.

### 1.2 Root Cause
- Message channels between the system tray and main UI thread were closed when the window was minimized, preventing restore commands from reaching the application.

## 2. Legacy Approaches and Lessons Learned

### 2.1 Channel Architecture Redesign (Deprecated)
- Modified system tray to use unbounded channels and `MessageSender` wrappers.
- Result: Channel still closed on window minimize.
- Lesson: Channel type was not the root cause; window lifecycle management was.

### 2.2 Complete Architecture Overhaul (Deprecated)
- Introduced `WindowController` struct, dedicated `mpsc::channel`, and bridge thread.
- Result: Channel issues persisted.
- Lesson: Architectural complexity did not resolve the underlying Iced window lifecycle problem.

### 2.3 Direct Windows API Control (Deprecated)
- Used `winapi` for direct OS-level window control (`ShowWindow`, `SW_RESTORE`, `SW_HIDE`).
- Result: Window content and event conflicts, cross-platform issues.
- Lesson: Bypassing the UI framework introduced instability and platform-specific bugs.

### 2.4 Content-Based Visibility (Deprecated)
- Rendered a minimal transparent container instead of hiding the window.
- Result: Window frame remained visible.
- Lesson: Content hiding does not equate to OS-level window hiding.

## 3. Final Working Solution: Iced Window Mode Commands

### 3.1 Core Strategy
- Use Iced's built-in window mode commands to hide and restore the window at the OS level, preserving the event loop and message channels.

### 3.2 Technical Implementation

#### 3.2.1 Window Mode Commands
```rust
// Hide window (minimize to tray)
iced::window::change_mode(iced::window::Mode::Hidden)

// Show window (restore from tray)
iced::window::change_mode(iced::window::Mode::Windowed)
```

#### 3.2.2 Message Handler Updates

**Exit Handler (`Message::Exit`):**
```rust
if self.config.ui.minimize_to_tray_on_close {
    log::info!("Minimizing to tray instead of exiting (hiding window but keeping event loop alive)");
    self.visible = false;
    if let Err(e) = self.config.save() {
        log::error!("Failed to save settings on minimize: {}", e);
    }
    return iced::window::change_mode(iced::window::Mode::Hidden);
}
```

**Show Window Handler (`Message::ShowWindow`):**
```rust
log::info!("ShowWindow message received from system tray, current visible: {}", self.visible);
self.visible = true;
log::info!("Window visibility set to true, restoring window from minimized state");
return iced::window::change_mode(iced::window::Mode::Windowed);
```

**Hide Window Handler (`Message::HideWindow`):**
```rust
log::info!("HideWindow message received from system tray");
self.visible = false;
return iced::window::change_mode(iced::window::Mode::Hidden);
```

#### 3.2.3 Application Settings Configuration
In `src/ui/app.rs`:
```rust
iced::Settings {
    // ... other settings
    exit_on_close_request: false, // Custom close handling for system tray
}
```

#### 3.2.4 Channel Preservation Strategy
Maintain the message channel using a `controller_subscription()`:
```rust
fn controller_subscription() -> Subscription<Message> {
    use iced::subscription;
    subscription::unfold(
        std::any::TypeId::of::<()>(),
        (),
        |_state| {
            Box::pin(async {
                let message = unsafe {
                    if let Some(ref mut receiver) = CONTROLLER_RECEIVER {
                        receiver.recv().await
                    } else {
                        None
                    }
                };
                match message {
                    Some(msg) => {
                        crate::debug_log!("ui", "Controller subscription received message: {:?}", msg);
                        (msg, ())
                    }
                    None => {
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                        (Message::Tick, ())
                    }
                }
            })
        }
    )
}
```

## 4. Rationale for the Final Solution

### 4.1 OS-Level Window Control
- `Mode::Hidden` and `Mode::Windowed` provide true OS-level window hiding and restoration.
- No custom window frame or platform-specific code required.

### 4.2 Event Loop and Channel Preservation
- Iced's window mode commands keep the event loop and message channels alive.
- Background operations and subscriptions continue during minimize.

### 4.3 Framework Alignment and Cross-Platform Support
- Solution leverages Iced's intended patterns and cross-platform abstractions.
- No Windows-specific dependencies; consistent behavior across supported platforms.

## 5. Key Files and Modules
- `src/ui/state.rs`: Message handling and window mode commands
- `src/ui/app.rs`: Application settings configuration
- `src/ui/system_tray.rs`: System tray event handlers

## 6. Testing and Validation Protocols

### 6.1 Test Scenarios
1. Minimize to tray: Close button hides window completely
2. Restore from tray: Tray icon click restores window with full content
3. Multiple cycles: Repeated minimize/restore operations are reliable
4. Message channel integrity: No channel closed errors in logs
5. Background operations: Device scanning continues while minimized

### 6.2 Success Criteria
- Window frame is fully hidden when minimized
- Tray icon restores window with full UI content
- No channel communication errors
- Background processes continue during minimize

## 7. Future Enhancements and Maintenance

### 7.1 Potential Enhancements
- Window position memory and multi-monitor support
- Graceful degradation if window modes are unsupported
- Animation support for minimize/restore transitions

### 7.2 Maintenance Notes
- Monitor Iced framework updates for window mode API changes
- Test on multiple Windows versions for OS-level compatibility
- Verify behavior with different window managers (if supporting Linux)

## 8. Debugging and Diagnostics

### 8.1 Common Issues
- Channel errors: Ensure `controller_subscription()` is active
- Window not hiding: Verify use of `Mode::Hidden`
- Restore failures: Confirm `exit_on_close_request: false` is set

### 8.2 Diagnostic Logging
- Monitor log messages:
  - "Using Iced window action to hide window"
  - "Window visibility set to true, restoring window from minimized state"
  - "Controller subscription received message"
- Use `--debug-ui` flag for system tray debug output:
```bash
rustpods --debug-ui
```

---

**This protocol is maintained as a living reference. Update as the application or UI framework evolves.** 