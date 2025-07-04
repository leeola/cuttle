pub mod msgbus;

use crate::service::{PingService, ServiceManager};
use flume::{Receiver, Sender};
use serde::{Deserialize, Serialize};
use std::thread;
use tokio::runtime::Runtime;
use tracing::{error, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceMessage {
    Ping,
    Stop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceResponse {
    Pong,
    Stopped,
    Error(String),
}

pub struct PyBridge {
    to_async: Sender<ServiceMessage>,
    from_async: Receiver<ServiceResponse>,
    runtime_handle: Option<thread::JoinHandle<()>>,
}

pub struct PyBridgeAsync {
    pub rx: Receiver<ServiceMessage>,
    pub tx: Sender<ServiceResponse>,
}

impl PyBridge {
    pub fn new() -> (Self, PyBridgeAsync) {
        let (to_async, async_rx) = flume::unbounded();
        let (async_tx, from_async) = flume::unbounded();

        let sync_side = PyBridge {
            to_async,
            from_async,
            runtime_handle: None,
        };

        let async_side = PyBridgeAsync {
            rx: async_rx,
            tx: async_tx,
        };

        (sync_side, async_side)
    }

    pub fn send(&self, msg: ServiceMessage) -> Result<(), flume::SendError<ServiceMessage>> {
        self.to_async.send(msg)
    }

    pub fn try_recv(&self) -> Option<ServiceResponse> {
        self.from_async.try_recv().ok()
    }

    pub fn start_runtime(&mut self, async_bridge: PyBridgeAsync) {
        info!("Starting async runtime");

        let handle = thread::spawn(move || {
            let rt = Runtime::new().expect("Failed to create tokio runtime");

            rt.block_on(async move {
                info!("Async runtime started");

                // Initialize service manager with basic services
                let mut service_manager = ServiceManager::new();
                service_manager.add_service(Box::new(PingService::new("main")));

                if let Err(e) = service_manager.start_all().await {
                    error!("Failed to start services: {}", e);
                    return;
                }

                // Message handling loop
                loop {
                    if let Ok(msg) = async_bridge.rx.recv_async().await {
                        info!("Received message: {:?}", msg);

                        let should_stop = matches!(msg, ServiceMessage::Stop);

                        let response = if should_stop {
                            info!("Stopping async runtime");
                            if let Err(e) = service_manager.stop_all().await {
                                error!("Failed to stop services: {}", e);
                            }
                            ServiceResponse::Stopped
                        } else {
                            service_manager.handle_message(msg).await
                        };

                        if let Err(e) = async_bridge.tx.send_async(response).await {
                            error!("Failed to send response: {}", e);
                            break;
                        }

                        if should_stop {
                            break;
                        }
                    } else {
                        info!("Channel closed, stopping runtime");
                        if let Err(e) = service_manager.stop_all().await {
                            error!("Failed to stop services: {}", e);
                        }
                        break;
                    }
                }
            });
        });

        self.runtime_handle = Some(handle);
    }

    pub fn stop(&mut self) {
        if let Err(e) = self.send(ServiceMessage::Stop) {
            error!("Failed to send stop message: {}", e);
        }

        if let Some(handle) = self.runtime_handle.take() {
            if let Err(e) = handle.join() {
                error!("Failed to join runtime thread: {:?}", e);
            }
        }
    }
}

impl Drop for PyBridge {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_ping_pong() {
        let (mut bridge, async_bridge) = PyBridge::new();
        bridge.start_runtime(async_bridge);

        // Send ping
        bridge
            .send(ServiceMessage::Ping)
            .expect("Failed to send ping message");

        // Wait a bit for async processing
        thread::sleep(Duration::from_millis(10));

        // Check for pong response
        if let Some(response) = bridge.try_recv() {
            match response {
                ServiceResponse::Pong => println!("Received pong!"),
                _ => panic!("Expected pong response"),
            }
        } else {
            panic!("No response received");
        }

        // Clean shutdown
        bridge.stop();
    }
}
