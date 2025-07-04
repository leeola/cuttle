bl_info = {
    "name": "Cuttle Test Integration",
    "blender": (3, 0, 0),
    "category": "Development",
    "description": "Test Cuttle async/sync architecture in Blender",
    "author": "Cuttle Project",
    "version": (0, 1, 0),
}

import bpy
import sys
import os
import subprocess
import tempfile
import time
from pathlib import Path

# Add this addon's directory to Python path for imports
addon_dir = Path(__file__).parent
if str(addon_dir) not in sys.path:
    sys.path.append(str(addon_dir))


class CUTTLE_OT_test_architecture(bpy.types.Operator):
    bl_idname = "cuttle.test_architecture"
    bl_label = "Test Cuttle Architecture"
    bl_description = "Test the async/sync bridge architecture"

    def execute(self, context):
        self.report({"INFO"}, "Testing Cuttle architecture...")

        # Test 1: Build and run core Rust functionality
        try:
            self.test_rust_core()
            self.report({"INFO"}, "PASS: Rust core tests passed")
        except Exception as e:
            self.report({"ERROR"}, f"FAIL: Rust core failed: {e}")
            return {"CANCELLED"}

        # Test 2: Simulate the PyO3 bridge behavior
        try:
            self.test_simulated_bridge()
            self.report({"INFO"}, "PASS: Simulated bridge communication works")
        except Exception as e:
            self.report({"ERROR"}, f"FAIL: Simulated bridge failed: {e}")
            return {"CANCELLED"}

        # Test 3: Test file logging
        try:
            self.test_file_logging()
            self.report({"INFO"}, "PASS: File logging works")
        except Exception as e:
            self.report({"ERROR"}, f"FAIL: File logging failed: {e}")
            return {"CANCELLED"}

        self.report({"INFO"}, "SUCCESS: All Cuttle architecture tests passed!")
        return {"FINISHED"}

    def test_rust_core(self):
        """Test that our Rust core compiles and tests pass"""
        # Get the project root (parent of addon directory)
        project_root = addon_dir.parent

        # Run cargo test on the cuttle crate
        result = subprocess.run(
            ["cargo", "test", "-p", "cuttle"],
            cwd=project_root,
            capture_output=True,
            text=True,
            timeout=30,
        )

        if result.returncode != 0:
            raise Exception(f"Cargo test failed: {result.stderr}")

        print("Cargo test output:")
        print(result.stdout)

    def test_simulated_bridge(self):
        """Simulate what the PyO3 bridge would do"""
        # This simulates the async/sync communication pattern
        # that our PyBridge implements

        import threading
        import queue
        import json

        # Simulate the message passing that PyBridge does
        to_service = queue.Queue()
        from_service = queue.Queue()

        def simulated_async_service():
            """Simulate an async service running in background"""
            while True:
                try:
                    # Wait for message (simulating async recv)
                    message = to_service.get(timeout=1)
                    print(f"Service received: {message}")

                    # Process message
                    if message == "ping":
                        response = "pong"
                    elif message == "stop":
                        response = "stopped"
                        from_service.put(response)
                        break
                    else:
                        response = f"unknown: {message}"

                    # Send response (simulating async send)
                    from_service.put(response)

                except queue.Empty:
                    continue

        # Start simulated service in background thread
        service_thread = threading.Thread(target=simulated_async_service, daemon=True)
        service_thread.start()

        # Test communication (simulating PyO3 functions)
        def send_message(msg):
            """Simulate cuttle_py.send_message()"""
            to_service.put(msg)

        def try_recv_response():
            """Simulate cuttle_py.try_recv_response()"""
            try:
                return from_service.get_nowait()
            except queue.Empty:
                return None

        # Test the communication pattern
        send_message("ping")
        time.sleep(0.1)  # Brief wait for processing

        response = try_recv_response()
        if response != "pong":
            raise Exception(f"Expected 'pong', got '{response}'")

        # Test stop
        send_message("stop")
        time.sleep(0.1)

        response = try_recv_response()
        if response != "stopped":
            raise Exception(f"Expected 'stopped', got '{response}'")

        print("Simulated bridge communication: PASSED")

    def test_file_logging(self):
        """Test file logging functionality"""
        log_file = tempfile.mktemp(suffix=".log", prefix="cuttle_test_")

        try:
            # Test that we can write to the log location
            with open(log_file, "w") as f:
                f.write("Test log entry from Blender\n")
                f.write(f"Blender version: {bpy.app.version_string}\n")
                f.write(f"Python version: {sys.version}\n")

            # Verify we can read it back
            with open(log_file, "r") as f:
                content = f.read()

            if "Test log entry" not in content:
                raise Exception("Log file content not found")

            print(f"Log file test successful: {log_file}")
            print("Log content:")
            print(content)

        finally:
            # Clean up
            if os.path.exists(log_file):
                os.unlink(log_file)


class CUTTLE_OT_simulate_modal(bpy.types.Operator):
    bl_idname = "cuttle.simulate_modal"
    bl_label = "Simulate Modal Operator"
    bl_description = "Simulate the modal operator pattern for service communication"

    _timer = None
    _message_queue = None
    _response_queue = None
    _counter = 0

    def modal(self, context, event):
        if event.type == "TIMER":
            # Simulate checking for service responses
            if self._response_queue and not self._response_queue.empty():
                try:
                    response = self._response_queue.get_nowait()
                    self.report({"INFO"}, f"Received: {response}")
                except:
                    pass

            # Send periodic ping messages
            self._counter += 1
            if self._counter % 10 == 0:  # Every 1 second (10 * 0.1s)
                if self._message_queue:
                    self._message_queue.put(f"ping_{self._counter//10}")
                    self.report({"INFO"}, f"Sent: ping_{self._counter//10}")

            # Stop after 5 seconds
            if self._counter >= 50:
                if self._message_queue:
                    self._message_queue.put("stop")
                return {"FINISHED"}

        return {"PASS_THROUGH"}

    def execute(self, context):
        # Set up queues for simulation
        import queue
        import threading

        self._message_queue = queue.Queue()
        self._response_queue = queue.Queue()
        self._counter = 0

        # Start simulated service
        def service():
            while True:
                try:
                    msg = self._message_queue.get(timeout=1)
                    if msg.startswith("ping"):
                        self._response_queue.put(f"pong_{msg.split('_')[1]}")
                    elif msg == "stop":
                        self._response_queue.put("stopped")
                        break
                except:
                    continue

        threading.Thread(target=service, daemon=True).start()

        # Start modal operator
        wm = context.window_manager
        self._timer = wm.event_timer_add(0.1, window=context.window)
        wm.modal_handler_add(self)

        self.report({"INFO"}, "Started modal operator simulation")
        return {"RUNNING_MODAL"}


class CUTTLE_OT_create_test_cube(bpy.types.Operator):
    bl_idname = "cuttle.create_test_cube"
    bl_label = "Create Test Cube"
    bl_description = "Create a cube to test Blender node manipulation"

    def execute(self, context):
        # This simulates what a real Cuttle service would do
        # when it receives a "create cube" message

        # Clear existing mesh objects
        bpy.ops.object.select_all(action="SELECT")
        bpy.ops.object.delete(use_global=False, confirm=False)

        # Create cube
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

        self.report({"INFO"}, "Created test cube (simulating service response)")
        return {"FINISHED"}


class CUTTLE_PT_test_panel(bpy.types.Panel):
    bl_label = "Cuttle Architecture Test"
    bl_idname = "CUTTLE_PT_test_panel"
    bl_space_type = "VIEW_3D"
    bl_region_type = "UI"
    bl_category = "Cuttle"

    def draw(self, context):
        layout = self.layout

        layout.label(text="Test Cuttle Integration:")
        layout.operator("cuttle.test_architecture")
        layout.separator()

        layout.label(text="Modal Operator Pattern:")
        layout.operator("cuttle.simulate_modal")
        layout.separator()

        layout.label(text="Simulate Node Creation:")
        layout.operator("cuttle.create_test_cube")


def register():
    bpy.utils.register_class(CUTTLE_OT_test_architecture)
    bpy.utils.register_class(CUTTLE_OT_simulate_modal)
    bpy.utils.register_class(CUTTLE_OT_create_test_cube)
    bpy.utils.register_class(CUTTLE_PT_test_panel)


def unregister():
    bpy.utils.unregister_class(CUTTLE_PT_test_panel)
    bpy.utils.unregister_class(CUTTLE_OT_create_test_cube)
    bpy.utils.unregister_class(CUTTLE_OT_simulate_modal)
    bpy.utils.unregister_class(CUTTLE_OT_test_architecture)


# Standalone test functions for CLI mode
def test_rust_core_standalone():
    """Test that our Rust core compiles and tests pass - standalone version"""
    from pathlib import Path
    import subprocess

    # Get the project root (parent of addon directory)
    addon_dir = Path(__file__).parent
    project_root = addon_dir.parent

    # Run cargo test on the cuttle crate
    result = subprocess.run(
        ["cargo", "test", "-p", "cuttle"],
        cwd=project_root,
        capture_output=True,
        text=True,
        timeout=30,
    )

    if result.returncode != 0:
        raise Exception(f"Cargo test failed: {result.stderr}")

    print("Cargo test output:")
    print(result.stdout)


def test_simulated_bridge_standalone():
    """Simulate what the PyO3 bridge would do - standalone version"""
    import threading
    import queue
    import time

    # Simulate the message passing that PyBridge does
    to_service = queue.Queue()
    from_service = queue.Queue()

    def simulated_async_service():
        """Simulate an async service running in background"""
        while True:
            try:
                # Wait for message (simulating async recv)
                message = to_service.get(timeout=1)
                print(f"Service received: {message}")

                # Process message
                if message == "ping":
                    response = "pong"
                elif message == "stop":
                    response = "stopped"
                    from_service.put(response)
                    break
                else:
                    response = f"unknown: {message}"

                # Send response (simulating async send)
                from_service.put(response)

            except queue.Empty:
                continue

    # Start simulated service in background thread
    service_thread = threading.Thread(target=simulated_async_service, daemon=True)
    service_thread.start()

    # Test communication (simulating PyO3 functions)
    def send_message(msg):
        """Simulate cuttle_py.send_message()"""
        to_service.put(msg)

    def try_recv_response():
        """Simulate cuttle_py.try_recv_response()"""
        try:
            return from_service.get_nowait()
        except queue.Empty:
            return None

    # Test the communication pattern
    send_message("ping")
    time.sleep(0.1)  # Brief wait for processing

    response = try_recv_response()
    if response != "pong":
        raise Exception(f"Expected 'pong', got '{response}'")

    # Test stop
    send_message("stop")
    time.sleep(0.1)

    response = try_recv_response()
    if response != "stopped":
        raise Exception(f"Expected 'stopped', got '{response}'")

    print("Simulated bridge communication: PASSED")


def test_file_logging_standalone():
    """Test file logging functionality - standalone version"""
    import tempfile
    import os
    import sys
    import bpy

    log_file = tempfile.mktemp(suffix=".log", prefix="cuttle_test_")

    try:
        # Test that we can write to the log location
        with open(log_file, "w") as f:
            f.write("Test log entry from Blender\n")
            f.write(f"Blender version: {bpy.app.version_string}\n")
            f.write(f"Python version: {sys.version}\n")

        # Verify we can read it back
        with open(log_file, "r") as f:
            content = f.read()

        if "Test log entry" not in content:
            raise Exception("Log file content not found")

        print(f"Log file test successful: {log_file}")
        print("Log content:")
        print(content)

    finally:
        # Clean up
        if os.path.exists(log_file):
            os.unlink(log_file)


if __name__ == "__main__":
    register()
