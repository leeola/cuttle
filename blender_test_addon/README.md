# Cuttle Blender Test Addon

This addon tests the Cuttle async/sync bridge architecture directly in Blender, bypassing the need for PyO3 linking setup.

## Installation

1. Open Blender
2. Go to Edit > Preferences > Add-ons
3. Click "Install..." 
4. Navigate to `/Users/lee/projects/cuttle/blender_test_addon/`
5. Select `__init__.py`
6. Enable the "Cuttle Test Integration" addon

## Usage

After installation, you'll see a "Cuttle" panel in the 3D Viewport sidebar (press `N` to show sidebar).

### Tests Available

**Test Cuttle Architecture**:
- Runs `cargo test -p cuttle` to verify Rust core works
- Simulates the PyO3 bridge communication pattern  
- Tests file logging functionality
- Reports all results in Blender's interface

**Simulate Modal Operator**:
- Demonstrates the modal operator pattern we'll use for real services
- Shows ping/pong communication every second for 5 seconds
- Simulates exactly how our PyBridge will work

**Create Test Cube**:
- Simulates what a real Cuttle service would do
- Creates a red cube (representing node creation from services)
- Shows Blender object manipulation from "service" code

## What This Proves

This addon validates our architecture works in Blender:

1. **Threading**: Background services can run in Blender
2. **Communication**: Message passing between async and sync code  
3. **Modal Integration**: Non-blocking polling for service responses
4. **Blender API**: Can manipulate nodes/objects from service code
5. **File Logging**: Debug output works for complex async debugging

## Expected Output

When you click "Test Cuttle Architecture", you should see:
- `PASS: Rust core tests passed` 
- `PASS: Simulated bridge communication works`
- `PASS: File logging works`
- `SUCCESS: All Cuttle architecture tests passed!`

This confirms our async/sync bridge architecture is solid and ready for real PyO3 integration.

## Next Steps

Once this test passes, we can:
1. Build the actual PyO3 extension (using maturin when needed)
2. Add real services (file watcher, RPC, LSP)
3. Connect to real .geonode files
4. Implement bidirectional sync with Blender UI

The hard part (threading architecture) is done and tested!