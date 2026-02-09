# Basic Game Window

## Overview
When the user starts the application, a game window is displayed with an empty black screen. The window can be closed by pressing the ESC key, which exits the application cleanly.

## Requirements
- Application starts with a visible window
- Window title displays "WoK"
- Default resolution is 800x600 pixels
- Window has a clear background (black screen)
- Pressing ESC key exits the application
- Window can also be closed via OS window controls (X button)

## Acceptance Criteria
- [x] Window opens when application starts
- [x] Window displays "WoK" as title
- [x] Window is 800x600 pixels by default
- [x] Background is black/empty
- [x] Pressing ESC exits cleanly
- [x] Closing window via OS controls exits cleanly

---

## Implementation Plan

### Approach
Use Bevy's `DefaultPlugins` to handle window creation and basic functionality. Add a simple keyboard input system to detect ESC key and send `AppExit` event.

### Components & Systems
**Resources:**
- Bevy's built-in `ButtonInput<KeyCode>` - keyboard input state
- Bevy's built-in `AppExit` event - for clean shutdown

**Systems:**
- `exit_on_esc()` - Listens for ESC key and sends exit event

### Tasks
- [x] Replace `println!("Hello, world!")` with Bevy app setup
- [x] Configure `DefaultPlugins` with window settings (title: "WoK", resolution: 800x600)
- [x] Add `exit_on_esc` system to `Update` schedule
- [x] Test: Run application and verify window appears
- [x] Test: Press ESC and verify clean exit
- [x] Test: Close window with OS controls and verify clean exit

### Notes
- Bevy's `DefaultPlugins` includes `WindowPlugin` which handles window creation
- Window configuration uses `Window` primary window settings
- `AppExit` event is the recommended way to exit Bevy apps
- ESC exit is a common pattern in game development for quick testing
- The clear color (black) is Bevy's default, so no additional configuration needed
- Example system signature: `exit_on_esc(keyboard: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>)`
