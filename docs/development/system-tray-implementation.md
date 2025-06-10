# System Tray Implementation Strategy

## Overview

This document describes the comprehensive strategy implemented for system tray minimize-to-tray functionality in RustPods. The solution ensures proper window hiding/showing while maintaining active message channels and event loop functionality.

## Problem Statement

### Initial Issue
- System tray minimize-to-tray was working (window content disappeared)
- Restore functionality was broken (clicking tray icon or right-click menu had no effect)
- Users had to force-kill the application to exit

### Root Cause Analysis
The core problem was that message channels between the system tray and main UI thread were being closed when the window was minimized, preventing restore commands from reaching the application.

## Failed Approaches and Lessons Learned

### Approach 1: Channel Architecture Redesign
**Strategy**: Modified system tray to use unbounded channels and `MessageSender` wrapper.
**Result**: Channel still closed on window minimize.
**Lesson**: The issue wasn't the channel type but how window minimization was being handled.

### Approach 2: Complete Architecture Overhaul
**Strategy**: Implemented new architecture with:
- `WindowController` struct with dedicated `mpsc::channel`
- `WindowCommand` enum (`Show`, `Hide`, `Toggle`, `Exit`)
- Bridge thread converting window commands to UI messages
- Self-contained system tray independent of UI event loop

**Result**: Channel issues persisted.
**Lesson**: Complex architectural changes didn't address the fundamental Iced window lifecycle problem.

### Approach 3: Direct Windows API Control
**Strategy**: Bypassed Iced event system entirely using Windows API:
- Added `winapi` dependency
- Used `ShowWindow`, `SW_RESTORE`, `SW_HIDE`, `SetForegroundWindow` API calls
- Implemented window handle capture via `enum_windows_proc` callback
- Created `DirectWindowController`

**Result**: Multiple issues including empty window content and event conflicts.
**Lesson**: Fighting against the UI framework's intended patterns creates more problems.

### Approach 4: Content-Based Visibility
**Strategy**: Set `visible = false` and rendered minimal 1x1 pixel transparent container.
**Result**: Window frame remained visible to user.
**Lesson**: Content hiding ≠ window hiding at the OS level.

## Final Working Solution

### Core Strategy
Use Iced's built-in window mode commands that properly hide the window at the OS level without disrupting the event loop or message channels.

### Technical Implementation

#### 1. Window Mode Commands
```rust
// Hide window (minimize to tray)
iced::window::change_mode(iced::window::Mode::Hidden)

// Show window (restore from tray)
iced::window::change_mode(iced::window::Mode::Windowed)
```

#### 2. Message Handler Updates

**Exit Handler** (`Message::Exit`):
```rust
if self.config.ui.minimize_to_tray_on_close {
    log::info!("Minimizing to tray instead of exiting (hiding window but keeping event loop alive)");
    self.visible = false;
    // Save settings when minimizing to tray
    if let Err(e) = self.config.save() {
        log::error!("Failed to save settings on minimize: {}", e);
    }
    // Use Iced's proper window hiding command that doesn't close the event loop
    return iced::window::change_mode(iced::window::Mode::Hidden);
}
```

**Show Window Handler** (`Message::ShowWindow`):
```rust
log::info!("ShowWindow message received from system tray, current visible: {}", self.visible);
self.visible = true;
log::info!("Window visibility set to true, restoring window from minimized state");
// Use Iced's window restoration command to properly show the window
return iced::window::change_mode(iced::window::Mode::Windowed);
```

**Hide Window Handler** (`Message::HideWindow`):
```rust
log::info!("HideWindow message received from system tray");
self.visible = false;
// Use Iced's window hiding to hide the window
return iced::window::change_mode(iced::window::Mode::Hidden);
```

#### 3. Critical Configuration
In `src/ui/app.rs`, ensure:
```rust
iced::Settings {
    // ... other settings
    exit_on_close_request: false, // Allow custom handling of close requests for system tray
}
```

This prevents Iced from automatically shutting down the application when the close button is clicked.

#### 4. Channel Preservation Strategy
The solution includes a `controller_subscription()` that maintains the message channel:

```rust
fn controller_subscription() -> Subscription<Message> {
    use iced::subscription;
    
    subscription::unfold(
        std::any::TypeId::of::<()>(),
        (),
        |_state| {
            Box::pin(async {
                // Access the global receiver safely
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
                        // Channel closed or no message, wait and try again
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                        // Return a dummy message to keep the subscription alive
                        (Message::Tick, ())
                    }
                }
            })
        }
    )
}
```

## Why This Solution Works

### 1. **Proper OS-Level Window Control**
- `Mode::Hidden` actually hides the window at the operating system level
- `Mode::Windowed` properly restores the window with full functionality
- No custom window frame handling required

### 2. **Event Loop Preservation**
- Iced's `Mode::Hidden` keeps the event loop running
- Message channels remain open and functional
- Background subscriptions continue operating

### 3. **Framework Alignment**
- Works with Iced's intended patterns rather than against them
- Uses built-in functionality rather than external APIs
- Reduces complexity and potential conflicts

### 4. **Cross-Platform Compatibility**
- Iced handles platform-specific window management
- No Windows-specific API dependencies
- Consistent behavior across supported platforms

## Key Files Modified

- `src/ui/state.rs`: Core message handling and window mode commands
- `src/ui/app.rs`: Application settings configuration
- `src/ui/system_tray.rs`: System tray event handlers

## Testing and Validation

### Test Scenarios
1. **Minimize to Tray**: Close button hides window completely (no frame visible)
2. **Restore from Tray**: Single-click tray icon restores window with full content
3. **Multiple Cycles**: Repeated minimize/restore operations work consistently
4. **Message Channel Integrity**: No "channel closed" errors in logs
5. **Background Operations**: Device scanning continues while minimized

### Success Criteria
- ✅ Window frame completely disappears when minimized
- ✅ Tray icon click restores window properly
- ✅ Full UI content displayed on restore
- ✅ No channel communication errors
- ✅ Background processes continue during minimize

## Future Considerations

### Potential Enhancements
1. **Window Position Memory**: Remember window position before minimize
2. **Multi-Monitor Support**: Restore to correct monitor
3. **Graceful Degradation**: Fallback behavior if window modes aren't supported
4. **Animation Support**: Smooth minimize/restore transitions

### Maintenance Notes
- Monitor Iced framework updates for window mode API changes
- Test on different Windows versions for OS-level compatibility
- Verify behavior with different window managers (if supporting Linux)

## Debugging Tips

### Common Issues
1. **Channel Errors**: Check that `controller_subscription()` is included in the subscription list
2. **Window Not Hiding**: Verify `Mode::Hidden` is being used, not content-based hiding
3. **Restore Failures**: Ensure `exit_on_close_request: false` is set in application settings

### Diagnostic Logging
Key log messages to monitor:
- "Using Iced window action to hide window"
- "Window visibility set to true, restoring window from minimized state"
- "Controller subscription received message"

Use the `--debug-ui` flag to enable system tray debug logging:
```bash
rustpods --debug-ui    # Shows UI and system tray debug messages
```

This strategy provides a robust, maintainable solution that leverages the UI framework's capabilities while ensuring reliable system tray functionality. 