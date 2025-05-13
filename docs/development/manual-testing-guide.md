# Manual Testing Guide

This document outlines procedures for manually testing aspects of RustPods that are difficult to automate. While we aim to automate as much testing as possible, some features require human verification, especially those involving device interaction, UI responsiveness, and platform-specific behaviors.

## Table of Contents

1. [Bluetooth Device Interaction](#bluetooth-device-interaction)
2. [System Tray Testing](#system-tray-testing)
3. [UI Responsiveness Testing](#ui-responsiveness-testing)
4. [Cross-Platform Verification](#cross-platform-verification)
5. [AirPods Connection Testing](#airpods-connection-testing)
6. [Reporting Issues Found During Manual Testing](#reporting-issues)

## Bluetooth Device Interaction

Automatic testing of actual Bluetooth device interaction is challenging. Use the following procedures to verify Bluetooth functionality:

### Adapter Discovery Testing

1. **Prerequisites:**
   - At least one Bluetooth adapter physically available on the system
   - Bluetooth service enabled in Windows

2. **Test Procedure:**
   - Launch RustPods in CLI mode with: `rustpods adapters`
   - Verify that all physical Bluetooth adapters are detected and listed
   - If multiple adapters are present, verify each is correctly identified with its name and address

3. **Expected Outcome:**
   - All physical Bluetooth adapters are detected and displayed
   - Adapter information (name, address, capabilities) is accurate
   - No errors are shown in the terminal output

### Bluetooth Scanning Testing

1. **Prerequisites:**
   - Bluetooth adapter is properly functioning
   - At least one Bluetooth device (preferably multiple) in discovery mode nearby

2. **Test Procedure:**
   - Launch RustPods in CLI mode with: `rustpods scan`
   - Verify that nearby Bluetooth devices are detected
   - Check if the signal strength (RSSI) values are reasonable
   - Confirm that manufacturer data is displayed when available

3. **Expected Outcome:**
   - Nearby Bluetooth devices are detected and displayed
   - Device information is accurate (address matches the actual device)
   - Signal strength values change based on device proximity

### Testing with Real AirPods

1. **Prerequisites:**
   - AirPods paired with the Windows system
   - AirPods case with some charge

2. **Test Procedure:**
   - Ensure AirPods are in case and case is closed
   - Launch RustPods in CLI mode with: `rustpods airpods`
   - Open the AirPods case lid
   - Verify that the AirPods are detected
   - Note battery level readings and verify they seem reasonable

3. **Verification Steps:**
   - Remove one AirPod and verify the detection status changes
   - Replace it and remove the other, verifying the same
   - Test with different charge levels to ensure battery reporting is accurate

## System Tray Testing

System tray behavior can vary based on Windows version and configuration:

### Tray Icon Appearance

1. **Test Procedure:**
   - Launch RustPods with: `rustpods`
   - Verify the system tray icon appears in the taskbar
   - Ensure the icon is clear and properly sized
   - Verify icon behavior when changing display scaling settings

2. **Expected Outcome:**
   - Icon appears immediately upon application launch
   - Icon is visually clear and matches the application theme
   - Icon remains properly sized when display scaling changes

### Tray Menu Functionality

1. **Test Procedure:**
   - Right-click the system tray icon to open the context menu
   - Verify all menu items are displayed correctly
   - Test each menu item:
     - "Show" - Should bring the main window to front if minimized
     - "Settings" - Should open the settings window
     - "Exit" - Should close the application completely

2. **Expected Behavior:**
   - Menu appears without delay when right-clicking
   - All menu items are readable and properly aligned
   - Menu items function as expected when clicked
   - Menu closes properly after selection or clicking elsewhere

### Tray Icon Updates

1. **Test Procedure:**
   - Connect AirPods and verify the tray icon updates to reflect battery status
   - Allow battery to change and verify the icon updates accordingly
   - Disconnect AirPods and verify the icon returns to the default state

2. **Expected Outcome:**
   - Icon updates to reflect current battery status
   - Updates occur within a reasonable timeframe after status changes
   - Icon is clearly distinguishable at different battery levels

## UI Responsiveness Testing

Automated testing tools may not accurately measure perceived responsiveness:

### Window Opening and Transition Testing

1. **Test Procedure:**
   - Launch the application and measure time until main window appears
   - Open and close the settings window repeatedly
   - Minimize the application to tray and restore it multiple times

2. **Expected Outcome:**
   - Main window opens within 2 seconds of application launch
   - Settings window opens and closes smoothly without UI freezing
   - Application restores from minimized state quickly (under 1 second)

### Animation Smoothness Testing

1. **Test Procedure:**
   - Observe battery level indicator animations
   - Trigger state changes to test transition animations
   - Test on both high-performance and lower-end systems

2. **Expected Outcome:**
   - Animations play smoothly without stuttering
   - UI remains responsive during animations
   - Animations complete in expected timeframe

### Input Responsiveness Testing

1. **Test Procedure:**
   - Click various buttons and controls in rapid succession
   - Type quickly in text input fields in the settings window
   - Drag/resize windows rapidly

2. **Expected Outcome:**
   - UI responds to all input events without noticeable delay
   - No input events are missed or processed out of order
   - Application remains stable under rapid input

## Cross-Platform Verification

While Windows is the primary target, verify basic functionality on other platforms:

### Windows Version Testing

1. **Test Procedure:**
   - Test on multiple Windows versions:
     - Windows 10
     - Windows 11
     - Windows Server (if applicable)
   - Verify core functionality on each platform

2. **Expected Outcome:**
   - Core functionality works consistently across Windows versions
   - UI scaling behaves appropriately on each platform
   - System tray integration works on all tested versions

### Remote Desktop Testing

1. **Test Procedure:**
   - Launch application on a remote desktop session
   - Test core functionality and system tray integration
   - Verify Bluetooth access works through remote session (if applicable)

2. **Expected Outcome:**
   - Application functions properly in remote desktop environment
   - System tray integration works correctly
   - Any limitations are properly documented

## AirPods Connection Testing

Testing the full AirPods connection lifecycle is important:

### Initial Detection

1. **Test Procedure:**
   - Start with AirPods in case with lid closed
   - Launch RustPods
   - Open AirPods case lid
   - Verify detection time and accuracy

2. **Expected Outcome:**
   - AirPods are detected within 5 seconds of opening the case
   - Correct AirPods model is identified
   - Case battery level is reported accurately

### Battery Reporting Accuracy

1. **Test Procedure:**
   - Test AirPods at different battery levels:
     - Nearly full (>80%)
     - Moderate (30-80%)
     - Low (<30%)
     - Very low (<10%)
   - Compare reported battery levels with iOS/macOS device if available

2. **Expected Outcome:**
   - Battery levels are reported with reasonable accuracy
   - Low battery levels trigger appropriate notifications
   - Battery levels update at the expected refresh interval

### Disconnection and Reconnection

1. **Test Procedure:**
   - Connect AirPods and verify detection
   - Close case and move AirPods out of range
   - Verify disconnection is detected
   - Bring AirPods back in range and verify reconnection

2. **Expected Outcome:**
   - Disconnection is detected within expected timeframe
   - UI updates to show disconnection state
   - Reconnection occurs automatically when AirPods return to range

## Reporting Issues {#reporting-issues}

When manual testing reveals issues:

1. **Document the Issue:**
   - Specific steps to reproduce
   - Expected vs. actual behavior
   - System configuration details
   - Screenshots or videos if applicable

2. **Issue Severity Classification:**
   - **Critical:** Prevents core functionality from working
   - **Major:** Significantly impacts user experience but has workarounds
   - **Minor:** Cosmetic issues or minor functional problems
   - **Enhancement:** Suggestions for improvements rather than bugs

3. **Reporting Process:**
   - Create a GitHub issue with the appropriate template
   - Label with "manual-test" and appropriate severity
   - Include all reproduction steps and system details
   - If possible, include automated test cases that could have caught the issue

By following these manual testing procedures, we can ensure aspects of the application that are difficult to test automatically still maintain high quality standards. 