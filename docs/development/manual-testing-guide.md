# RustPods Manual Testing Protocol: Technical Reference and Verification Procedures

## Preamble: Document Scope and Relevance

This document defines the formal procedures for manual verification of RustPods functionality that cannot be fully automated. It is intended for engineers and testers responsible for validating device interaction, user interface responsiveness, and platform-specific behaviors. All procedures are current as of the latest system architecture. Legacy or deprecated steps are clearly marked.

## Table of Contents

1. [Bluetooth Device Interaction](#bluetooth-device-interaction)
2. [System Tray Verification](#system-tray-verification)
3. [User Interface Responsiveness](#user-interface-responsiveness)
4. [Cross-Platform and Remote Desktop Validation](#cross-platform-and-remote-desktop-validation)
5. [AirPods Connection Lifecycle Testing](#airpods-connection-lifecycle-testing)
6. [Issue Reporting and Documentation](#issue-reporting-and-documentation)

## 1. Bluetooth Device Interaction

### 1.1 Adapter Discovery Verification
- **Prerequisites:**
  - At least one physical Bluetooth adapter present
  - Bluetooth service enabled in Windows
- **Procedure:**
  1. Launch RustPods in CLI mode: `rustpods adapters`
  2. Confirm all physical Bluetooth adapters are detected and listed
  3. For systems with multiple adapters, verify each is correctly identified (name, address)
- **Expected Results:**
  - All adapters are detected and displayed with accurate information
  - No errors or spurious warnings in terminal output

### 1.2 Paired Device Polling Verification
- **Prerequisites:**
  - Functional Bluetooth adapter
  - At least one paired Bluetooth device
- **Procedure:**
  1. Launch RustPods and allow automatic polling for paired devices
  2. Confirm all paired devices are detected and displayed in the UI
  3. Verify device information (name, address, type) for accuracy
- **Expected Results:**
  - All paired devices are detected and displayed
  - Device information is accurate and updates as devices are paired/unpaired

### 1.3 AirPods Device Detection
- **Prerequisites:**
  - AirPods paired with Windows
  - AirPods case with sufficient charge
- **Procedure:**
  1. Ensure AirPods are in the case with lid closed
  2. Launch RustPods
  3. Open the case lid or remove AirPods
  4. Confirm AirPods detection in the UI and verify battery level readings
  5. Remove and replace each AirPod to verify detection status changes
- **Expected Results:**
  - AirPods are detected within 5 seconds of activation
  - Battery levels are reported accurately and update as expected
  - Device status changes are reflected in the UI

## 2. System Tray Verification

### 2.1 Tray Icon Appearance
- **Procedure:**
  1. Launch RustPods
  2. Confirm the system tray icon appears in the Windows taskbar
  3. Verify icon clarity and correct sizing under various display scaling settings
- **Expected Results:**
  - Icon appears immediately and is visually clear
  - Icon remains properly sized and themed under all scaling configurations

### 2.2 Tray Menu Functionality
- **Procedure:**
  1. Right-click the tray icon to open the context menu
  2. Verify all menu items are present and readable
  3. Test each menu item:
     - "Show": Brings main window to foreground
     - "Settings": Opens settings window
     - "Exit": Closes the application
- **Expected Results:**
  - Menu appears without delay
  - All items function as specified
  - Menu closes properly after selection or loss of focus

### 2.3 Tray Icon State Updates
- **Procedure:**
  1. Connect AirPods and verify tray icon updates to reflect battery status
  2. Allow battery state to change and confirm icon updates accordingly
  3. Disconnect AirPods and verify icon returns to default state
- **Expected Results:**
  - Icon accurately reflects current battery status and connection state
  - Updates occur within a reasonable timeframe

## 3. User Interface Responsiveness

### 3.1 Window and Transition Performance
- **Procedure:**
  1. Launch RustPods and measure time to main window appearance
  2. Open and close the settings window repeatedly
  3. Minimize to tray and restore multiple times
- **Expected Results:**
  - Main window opens within 2 seconds
  - Settings window transitions are smooth and free of UI freezing
  - Application restores from minimized state in under 1 second

### 3.2 Animation and State Transition Smoothness
- **Procedure:**
  1. Observe battery indicator and other UI animations
  2. Trigger state changes to test transition animations
  3. Test on both high-performance and lower-end systems
- **Expected Results:**
  - Animations are smooth and free of stutter
  - UI remains responsive during all transitions

### 3.3 Input Responsiveness
- **Procedure:**
  1. Interact with buttons and controls in rapid succession
  2. Type in text input fields in the settings window
  3. Drag and resize windows rapidly
- **Expected Results:**
  - UI responds to all input events without delay
  - No input events are missed or processed out of order
  - Application remains stable under rapid input

## 4. Cross-Platform and Remote Desktop Validation

### 4.1 Windows Version Compatibility
- **Procedure:**
  1. Test on Windows 10, Windows 11, and Windows Server (if applicable)
  2. Verify core functionality and UI scaling on each platform
- **Expected Results:**
  - Core functionality and UI scaling are consistent across all tested Windows versions
  - System tray integration is reliable on all platforms

### 4.2 Remote Desktop Session Testing
- **Procedure:**
  1. Launch RustPods in a remote desktop session
  2. Test core functionality and system tray integration
  3. Verify Bluetooth access (if supported in remote session)
- **Expected Results:**
  - Application functions correctly in remote desktop environments
  - System tray integration is reliable
  - Any limitations are documented

## 5. AirPods Connection Lifecycle Testing

### 5.1 Initial Detection and Model Identification
- **Procedure:**
  1. Start with AirPods in case, lid closed
  2. Launch RustPods
  3. Open case lid or remove AirPods
  4. Measure detection time and verify model identification
- **Expected Results:**
  - AirPods are detected within 5 seconds
  - Correct model is identified
  - Case battery level is reported accurately

### 5.2 Battery Reporting Accuracy
- **Procedure:**
  1. Test AirPods at various battery levels: >80%, 30-80%, <30%, <10%
  2. Compare reported levels with iOS/macOS device if available
- **Expected Results:**
  - Battery levels are reported with reasonable accuracy
  - Low battery triggers appropriate notifications
  - Battery levels update at expected intervals

### 5.3 Disconnection and Reconnection Handling
- **Procedure:**
  1. Connect AirPods and verify detection
  2. Close case and move AirPods out of range
  3. Confirm disconnection is detected
  4. Return AirPods to range and verify reconnection
- **Expected Results:**
  - Disconnection is detected within expected timeframe
  - UI updates to reflect disconnection
  - Reconnection occurs automatically when AirPods return to range

## 6. Issue Reporting and Documentation

### 6.1 Issue Documentation Protocol
- **Required Information:**
  - Step-by-step reproduction instructions
  - Expected vs. actual behavior
  - System configuration details (OS version, hardware, Bluetooth adapter)
  - Screenshots or video evidence (if applicable)

### 6.2 Severity Classification
- **Critical:** Prevents core functionality
- **Major:** Significantly impacts user experience, but has workarounds
- **Minor:** Cosmetic or minor functional issues
- **Enhancement:** Suggestions for improvement

### 6.3 Reporting Process
- Create a GitHub issue using the appropriate template
- Label with "manual-test" and the correct severity
- Include all reproduction steps and system details
- If possible, propose or include automated test cases that could detect the issue

---

**This protocol is maintained as a living reference. Update as the application evolves or as new manual verification requirements are identified.** 