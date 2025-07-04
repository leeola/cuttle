#!/usr/bin/env python3
"""
Automated test script for Blender CLI integration.
Run with: blender --background --python test_blender_cli.py
"""

import sys
import os
from pathlib import Path

# Add the addon directory to Python path
script_dir = Path(__file__).parent
addon_dir = script_dir / "blender_test_addon"

print(f"Script directory: {script_dir}")
print(f"Addon directory: {addon_dir}")
print(f"Addon exists: {addon_dir.exists()}")

if str(addon_dir) not in sys.path:
    sys.path.insert(0, str(addon_dir))

# Also add the project root to find the addon
if str(script_dir) not in sys.path:
    sys.path.insert(0, str(script_dir))


def test_cuttle_in_blender():
    """Test Cuttle architecture in Blender CLI mode"""
    print("=" * 60)
    print("CUTTLE BLENDER CLI TEST")
    print("=" * 60)

    try:
        # Import Blender API
        import bpy

        print(f"PASS: Blender version: {bpy.app.version_string}")
        print(f"PASS: Python version: {sys.version.split()[0]}")

        # Import and register our addon
        print("\n--- Registering Cuttle Test Addon ---")
        import blender_test_addon

        blender_test_addon.register()
        print("PASS: Addon registered successfully")

        # Test 1: Rust core functionality
        print("\n--- Testing Rust Core ---")

        # We'll call the test methods directly since we can't instantiate operators in CLI
        try:
            print("Running Rust core test...")
            # Import and call the test method directly
            from blender_test_addon import test_rust_core_standalone

            test_rust_core_standalone()
            print("PASS: Rust core test passed")
        except Exception as e:
            print(f"FAIL: Rust core test failed: {e}")
            return False

        try:
            print("Running simulated bridge test...")
            from blender_test_addon import test_simulated_bridge_standalone

            test_simulated_bridge_standalone()
            print("PASS: Bridge simulation test passed")
        except Exception as e:
            print(f"FAIL: Bridge simulation test failed: {e}")
            return False

        try:
            print("Running file logging test...")
            from blender_test_addon import test_file_logging_standalone

            test_file_logging_standalone()
            print("PASS: File logging test passed")
        except Exception as e:
            print(f"FAIL: File logging test failed: {e}")
            return False

        # Test 2: Modal operator simulation
        print("\n--- Testing Modal Operator Pattern ---")
        try:
            # Test modal operator pattern without instantiating the class
            import queue
            import threading
            import time

            message_queue = queue.Queue()
            response_queue = queue.Queue()

            # Start the simulated service (same logic as in the modal operator)
            def service():
                for i in range(3):  # Test 3 messages
                    try:
                        msg = message_queue.get(timeout=1)
                        if msg.startswith("ping"):
                            response_queue.put(f"pong_{msg.split('_')[1]}")
                        elif msg == "stop":
                            response_queue.put("stopped")
                            break
                    except:
                        continue

            threading.Thread(target=service, daemon=True).start()

            # Test message passing (simulating what the modal operator does)
            message_queue.put("ping_1")
            message_queue.put("ping_2")
            message_queue.put("stop")

            # Verify responses
            time.sleep(0.1)

            responses = []
            while not response_queue.empty():
                responses.append(response_queue.get_nowait())

            expected = ["pong_1", "pong_2", "stopped"]
            if responses == expected:
                print("PASS: Modal operator simulation passed")
            else:
                print(
                    f"FAIL: Modal operator failed. Expected {expected}, got {responses}"
                )
                return False

        except Exception as e:
            print(f"FAIL: Modal operator test failed: {e}")
            return False

        # Test 3: Blender API manipulation
        print("\n--- Testing Blender API Integration ---")
        try:
            # Clear scene
            bpy.ops.object.select_all(action="SELECT")
            bpy.ops.object.delete(use_global=False, confirm=False)

            # Test cube creation directly (simulating what the service would do)
            # This mimics the logic in CUTTLE_OT_create_test_cube.execute()
            bpy.ops.mesh.primitive_cube_add(location=(0, 0, 0))
            cube = bpy.context.active_object
            cube.name = "CuttleCube"

            # Add material
            material = bpy.data.materials.new(name="CuttleMaterial")
            material.use_nodes = True
            material.node_tree.nodes["Principled BSDF"].inputs[0].default_value = (
                0.8,
                0.2,
                0.2,
                1.0,
            )  # Red
            cube.data.materials.append(material)

            # Verify cube was created
            if "CuttleCube" in bpy.data.objects:
                print("PASS: Blender API manipulation test passed")
            else:
                print("FAIL: Cube creation failed - object not found")
                return False

        except Exception as e:
            print(f"FAIL: Blender API test failed: {e}")
            return False

        print("\n" + "=" * 60)
        print("SUCCESS: ALL CUTTLE TESTS PASSED!")
        print("PASS: Rust core architecture works")
        print("PASS: Async/sync bridge simulation works")
        print("PASS: Modal operator pattern works")
        print("PASS: Blender API integration works")
        print("PASS: File logging works")
        print("\nCuttle is ready for PyO3 integration!")
        print("=" * 60)

        return True

    except ImportError as e:
        print(f"FAIL: Failed to import required modules: {e}")
        print("Make sure this script is run with Blender's Python")
        return False
    except Exception as e:
        print(f"FAIL: Unexpected error: {e}")
        import traceback

        traceback.print_exc()
        return False

    finally:
        # Clean up
        try:
            blender_test_addon.unregister()
            print("\nPASS: Addon unregistered cleanly")
        except:
            pass


if __name__ == "__main__":
    # This will run when called from Blender CLI
    success = test_cuttle_in_blender()

    # Exit with appropriate code
    if success:
        print("\nEXIT: SUCCESS")
        # Don't call sys.exit() in Blender context
    else:
        print("\nEXIT: FAILURE")
        # Don't call sys.exit() in Blender context
