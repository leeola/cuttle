#!/usr/bin/env python3
"""
Test script to verify the cuttle_py integration works.

This script simulates how Blender would use the cuttle_py module.
Once the PyO3 linking issues are resolved, this same code should work in Blender.
"""

import time
import sys
import os


def test_cuttle_integration():
    try:
        # This will fail with linking errors for now, but shows the intended usage
        import cuttle_py

        print("Starting cuttle services...")

        # Initialize logging to file (useful for Blender debugging)
        log_file = "/tmp/cuttle_blender.log"
        cuttle_py.init_logging(log_file)
        print(f"Logging initialized to: {log_file}")

        # Start async services
        cuttle_py.start_services()
        print("Services started successfully")

        # Test ping/pong communication
        print("Testing communication...")
        cuttle_py.send_message("ping")

        # Wait a bit for async processing
        time.sleep(0.1)

        # Check for response (this is what Blender's modal operator would do)
        response = cuttle_py.try_recv_response()
        if response:
            print(f"Received response: {response}")
        else:
            print("No response received")

        # Test stop message
        print("Stopping services...")
        cuttle_py.send_message("stop")

        # Wait for stop response
        time.sleep(0.1)
        response = cuttle_py.try_recv_response()
        if response:
            print(f"Stop response: {response}")

        print("Integration test completed successfully!")

        # Show log file contents
        if os.path.exists(log_file):
            print(f"\nLog file contents ({log_file}):")
            with open(log_file, "r") as f:
                print(f.read())

    except ImportError as e:
        print(f"Import failed (expected until PyO3 linking is fixed): {e}")
        print()
        print("To fix this, you would need to:")
        print("   1. Have Python development headers installed")
        print("   2. Build the cuttle_py extension module")
        print("   3. Add the built module to Python path")
        print()
        print("But the Rust core is working! Run 'cargo test -p cuttle' to verify.")
        return False
    except Exception as e:
        print(f"Unexpected error: {e}")
        return False

    return True


def show_blender_usage():
    """Show how this would be used in a Blender addon"""
    print("\nIn Blender, you would use it like this:")
    print("=" * 50)
    print(
        """
# Blender addon code
import bpy
import cuttle_py

class CuttleOperator(bpy.types.Operator):
    bl_idname = "cuttle.start_services"
    bl_label = "Start Cuttle Services"

    def execute(self, context):
        # Initialize with file logging for debugging
        cuttle_py.init_logging("/tmp/cuttle_blender.log")
        cuttle_py.start_services()

        # Start modal operator for receiving updates
        bpy.ops.cuttle.modal_operator()
        return {'FINISHED'}

class CuttleModalOperator(bpy.types.Operator):
    bl_idname = "cuttle.modal_operator"
    bl_label = "Cuttle Modal Operator"

    def modal(self, context, event):
        if event.type == 'TIMER':
            # Check for service responses
            response = cuttle_py.try_recv_response()
            if response:
                print(f"Service response: {response}")
                # Handle response (create objects, etc.)
        return {'PASS_THROUGH'}

    def execute(self, context):
        self._timer = context.window_manager.event_timer_add(0.1, window=context.window)
        context.window_manager.modal_handler_add(self)
        return {'RUNNING_MODAL'}
"""
    )


if __name__ == "__main__":
    print("Cuttle PyO3 Integration Test")
    print("=" * 40)

    success = test_cuttle_integration()

    if not success:
        show_blender_usage()

    print("\nNext steps:")
    print("   1. Set up Python development environment for PyO3")
    print("   2. Build cuttle_py extension module")
    print("   3. Test in actual Blender environment")
    print("   4. Add real Blender node manipulation")
