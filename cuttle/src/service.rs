use crate::bridge::{ServiceMessage, ServiceResponse};
use async_trait::async_trait;
use tracing::{info, warn};

#[async_trait]
pub trait Service: Send + Sync {
    async fn start(&mut self) -> Result<(), ServiceError>;
    async fn handle_message(&mut self, msg: ServiceMessage) -> ServiceResponse;
    async fn stop(&mut self) -> Result<(), ServiceError>;
}

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Service failed to start: {0}")]
    StartupError(String),
    #[error("Service failed to stop: {0}")]
    ShutdownError(String),
    #[error("Service error: {0}")]
    RuntimeError(String),
}

pub struct ServiceManager {
    services: Vec<Box<dyn Service>>,
}

impl ServiceManager {
    pub fn new() -> Self {
        Self {
            services: Vec::new(),
        }
    }

    pub fn add_service(&mut self, service: Box<dyn Service>) {
        self.services.push(service);
    }

    pub async fn start_all(&mut self) -> Result<(), ServiceError> {
        info!("Starting {} services", self.services.len());

        for service in &mut self.services {
            service.start().await?;
        }

        info!("All services started successfully");
        Ok(())
    }

    pub async fn stop_all(&mut self) -> Result<(), ServiceError> {
        info!("Stopping {} services", self.services.len());

        for service in &mut self.services {
            if let Err(e) = service.stop().await {
                warn!("Failed to stop service: {}", e);
            }
        }

        info!("All services stopped");
        Ok(())
    }

    pub async fn handle_message(&mut self, msg: ServiceMessage) -> ServiceResponse {
        // Route messages to appropriate services
        // For now, handle basic messages here and route Blender messages to services
        match msg {
            ServiceMessage::Ping => ServiceResponse::Pong,
            ServiceMessage::Stop => ServiceResponse::Stopped,
            // Route Blender messages to the first available service that can handle them
            blender_msg => {
                for service in &mut self.services {
                    let response = service.handle_message(blender_msg.clone()).await;
                    // If the service handled it (didn't return Error), return the response
                    if !matches!(response, ServiceResponse::Error(_)) {
                        return response;
                    }
                }
                // If no service handled it, return an error
                ServiceResponse::Error("No service available to handle message".to_string())
            }
        }
    }
}

impl Default for ServiceManager {
    fn default() -> Self {
        Self::new()
    }
}

// Basic ping service for testing
pub struct PingService {
    name: String,
}

impl PingService {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[async_trait]
impl Service for PingService {
    async fn start(&mut self) -> Result<(), ServiceError> {
        info!("Starting PingService: {}", self.name);
        Ok(())
    }

    async fn handle_message(&mut self, msg: ServiceMessage) -> ServiceResponse {
        info!("PingService {} handling message: {:?}", self.name, msg);
        match msg {
            ServiceMessage::Ping => ServiceResponse::Pong,
            ServiceMessage::Stop => ServiceResponse::Stopped,
            // PingService doesn't handle Blender operations
            _ => ServiceResponse::Error("PingService doesn't handle this message type".to_string()),
        }
    }

    async fn stop(&mut self) -> Result<(), ServiceError> {
        info!("Stopping PingService: {}", self.name);
        Ok(())
    }
}

// BlenderService implementation
pub struct BlenderService {
    name: String,
    api: Box<dyn cuttle_blender_api::BlenderApi + Send + Sync>,
}

impl BlenderService {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            // Use mock implementation for now
            api: Box::new(cuttle_blender_api::MockBlenderApi::new()),
        }
    }
}

#[async_trait]
impl Service for BlenderService {
    async fn start(&mut self) -> Result<(), ServiceError> {
        info!("Starting BlenderService: {}", self.name);
        Ok(())
    }

    async fn handle_message(&mut self, msg: ServiceMessage) -> ServiceResponse {
        info!("BlenderService {} handling message: {:?}", self.name, msg);

        match msg {
            ServiceMessage::CreateCube(params) => match self.api.create_cube(params) {
                Ok(()) => ServiceResponse::Created,
                Err(e) => ServiceResponse::Error(e.to_string()),
            },
            ServiceMessage::CreateSphere(params) => match self.api.create_sphere(params) {
                Ok(()) => ServiceResponse::Created,
                Err(e) => ServiceResponse::Error(e.to_string()),
            },
            ServiceMessage::CreateMaterial(params) => match self.api.create_material(params) {
                Ok(()) => ServiceResponse::Created,
                Err(e) => ServiceResponse::Error(e.to_string()),
            },
            ServiceMessage::AssignMaterial(params) => match self.api.assign_material(params) {
                Ok(()) => ServiceResponse::Created,
                Err(e) => ServiceResponse::Error(e.to_string()),
            },
            ServiceMessage::GetObject(params) => match self.api.get_object(params) {
                Ok(data) => ServiceResponse::ObjectData(data),
                Err(e) => ServiceResponse::Error(e.to_string()),
            },
            ServiceMessage::GetMaterial(params) => match self.api.get_material(params) {
                Ok(data) => ServiceResponse::MaterialData(data),
                Err(e) => ServiceResponse::Error(e.to_string()),
            },
            ServiceMessage::ListObjects => match self.api.list_objects() {
                Ok(objects) => ServiceResponse::ObjectList(objects),
                Err(e) => ServiceResponse::Error(e.to_string()),
            },
            ServiceMessage::ListMaterials => match self.api.list_materials() {
                Ok(materials) => ServiceResponse::MaterialList(materials),
                Err(e) => ServiceResponse::Error(e.to_string()),
            },
            ServiceMessage::ListMeshes => match self.api.list_meshes() {
                Ok(meshes) => ServiceResponse::MeshList(meshes),
                Err(e) => ServiceResponse::Error(e.to_string()),
            },
            ServiceMessage::ClearScene => match self.api.clear_scene() {
                Ok(()) => ServiceResponse::SceneCleared,
                Err(e) => ServiceResponse::Error(e.to_string()),
            },
            // BlenderService doesn't handle basic messages
            _ => ServiceResponse::Error(
                "BlenderService doesn't handle this message type".to_string(),
            ),
        }
    }

    async fn stop(&mut self) -> Result<(), ServiceError> {
        info!("Stopping BlenderService: {}", self.name);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_manager() {
        let mut manager = ServiceManager::new();

        // Add a test service
        manager.add_service(Box::new(PingService::new("test")));

        // Start all services
        manager.start_all().await.expect("Failed to start services");

        // Test message handling
        let response = manager.handle_message(ServiceMessage::Ping).await;
        match response {
            ServiceResponse::Pong => println!("Got pong response"),
            _ => panic!("Expected pong response"),
        }

        // Stop all services
        manager.stop_all().await.expect("Failed to stop services");
    }

    #[tokio::test]
    async fn test_ping_service() {
        let mut service = PingService::new("test");

        service.start().await.expect("Failed to start ping service");

        let response = service.handle_message(ServiceMessage::Ping).await;
        match response {
            ServiceResponse::Pong => println!("PingService responded correctly"),
            _ => panic!("Expected pong response"),
        }

        service.stop().await.expect("Failed to stop ping service");
    }
}
