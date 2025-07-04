#![allow(clippy::useless_conversion)]
#![allow(unsafe_op_in_unsafe_fn)]

use cuttle::{PyBridge, ServiceMessage, ServiceResponse};
use pyo3::prelude::*;
use std::sync::{Arc, Mutex, OnceLock};

// Global PyBridge instance
static BRIDGE: OnceLock<Arc<Mutex<PyBridge>>> = OnceLock::new();

#[pyfunction]
#[pyo3(signature = (log_file=None))]
fn init_logging(log_file: Option<&str>) -> PyResult<()> {
    cuttle::init_logging(log_file);
    Ok(())
}

#[pyfunction]
fn start_services() -> PyResult<()> {
    let (mut bridge, async_bridge) = PyBridge::new();
    bridge.start_runtime(async_bridge);

    BRIDGE.set(Arc::new(Mutex::new(bridge))).map_err(|_| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Services already started")
    })?;

    Ok(())
}

#[pyfunction]
fn send_message(msg: String) -> PyResult<()> {
    let bridge = BRIDGE
        .get()
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Services not started"))?;

    let bridge = bridge
        .lock()
        .map_err(|_| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Failed to lock bridge"))?;

    let service_msg = match msg.as_str() {
        "ping" => ServiceMessage::Ping,
        "stop" => ServiceMessage::Stop,
        _ => {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Unknown message: {msg}"
            )));
        }
    };

    bridge.send(service_msg).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Send failed: {e}"))
    })?;

    Ok(())
}

#[pyfunction]
fn try_recv_response() -> PyResult<Option<String>> {
    let bridge = BRIDGE
        .get()
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Services not started"))?;

    let bridge = bridge
        .lock()
        .map_err(|_| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Failed to lock bridge"))?;

    let response = bridge.try_recv();

    let result = response.map(|resp| match resp {
        ServiceResponse::Pong => "pong".to_string(),
        ServiceResponse::Stopped => "stopped".to_string(),
        ServiceResponse::Error(msg) => format!("error: {msg}"),
    });

    Ok(result)
}

#[pymodule]
fn cuttle_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(init_logging, m)?)?;
    m.add_function(wrap_pyfunction!(start_services, m)?)?;
    m.add_function(wrap_pyfunction!(send_message, m)?)?;
    m.add_function(wrap_pyfunction!(try_recv_response, m)?)?;
    Ok(())
}
