use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Core data types for Blender objects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn red() -> Self {
        Self::new(0.8, 0.2, 0.2, 1.0)
    }

    pub fn white() -> Self {
        Self::new(1.0, 1.0, 1.0, 1.0)
    }
}

// Blender object data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectData {
    pub name: String,
    pub object_type: String,
    pub location: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
    pub materials: Vec<String>,
    pub vertex_count: Option<usize>,
    pub face_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialData {
    pub name: String,
    pub use_nodes: bool,
    pub base_color: Color,
    pub metallic: f32,
    pub roughness: f32,
    pub node_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshData {
    pub name: String,
    pub vertex_count: usize,
    pub edge_count: usize,
    pub face_count: usize,
}

// Operation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCubeParams {
    pub location: Vec3,
    pub name: String,
    pub size: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSphereParams {
    pub location: Vec3,
    pub name: String,
    pub radius: f32,
    pub subdivisions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMaterialParams {
    pub name: String,
    pub base_color: Color,
    pub metallic: f32,
    pub roughness: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignMaterialParams {
    pub object_name: String,
    pub material_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetObjectParams {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetMaterialParams {
    pub name: String,
}

// Error types
#[derive(Debug, thiserror::Error)]
pub enum BlenderApiError {
    #[error("Object not found: {name}")]
    ObjectNotFound { name: String },
    #[error("Material not found: {name}")]
    MaterialNotFound { name: String },
    #[error("Operation failed: {message}")]
    OperationFailed { message: String },
    #[error("Invalid parameters: {message}")]
    InvalidParameters { message: String },
}

// The actual API trait - this will be implemented by the service
pub trait BlenderApi {
    fn create_cube(&mut self, params: CreateCubeParams) -> Result<(), BlenderApiError>;
    fn create_sphere(&mut self, params: CreateSphereParams) -> Result<(), BlenderApiError>;
    fn create_material(&mut self, params: CreateMaterialParams) -> Result<(), BlenderApiError>;
    fn assign_material(&mut self, params: AssignMaterialParams) -> Result<(), BlenderApiError>;
    fn get_object(&self, params: GetObjectParams) -> Result<ObjectData, BlenderApiError>;
    fn get_material(&self, params: GetMaterialParams) -> Result<MaterialData, BlenderApiError>;
    fn list_objects(&self) -> Result<Vec<String>, BlenderApiError>;
    fn list_materials(&self) -> Result<Vec<String>, BlenderApiError>;
    fn list_meshes(&self) -> Result<Vec<String>, BlenderApiError>;
    fn clear_scene(&mut self) -> Result<(), BlenderApiError>;
}

// Mock implementation for testing
pub struct MockBlenderApi {
    objects: HashMap<String, ObjectData>,
    materials: HashMap<String, MaterialData>,
}

impl MockBlenderApi {
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
            materials: HashMap::new(),
        }
    }
}

impl Default for MockBlenderApi {
    fn default() -> Self {
        Self::new()
    }
}

impl BlenderApi for MockBlenderApi {
    fn create_cube(&mut self, params: CreateCubeParams) -> Result<(), BlenderApiError> {
        let object = ObjectData {
            name: params.name.clone(),
            object_type: "MESH".to_string(),
            location: params.location,
            rotation: Vec3::zero(),
            scale: Vec3::new(params.size, params.size, params.size),
            materials: Vec::new(),
            vertex_count: Some(8),
            face_count: Some(6),
        };

        self.objects.insert(params.name, object);
        Ok(())
    }

    fn create_sphere(&mut self, params: CreateSphereParams) -> Result<(), BlenderApiError> {
        let vertex_count = (params.subdivisions * params.subdivisions * 4) as usize;
        let face_count = (params.subdivisions * params.subdivisions * 4) as usize;

        let object = ObjectData {
            name: params.name.clone(),
            object_type: "MESH".to_string(),
            location: params.location,
            rotation: Vec3::zero(),
            scale: Vec3::new(params.radius, params.radius, params.radius),
            materials: Vec::new(),
            vertex_count: Some(vertex_count),
            face_count: Some(face_count),
        };

        self.objects.insert(params.name, object);
        Ok(())
    }

    fn create_material(&mut self, params: CreateMaterialParams) -> Result<(), BlenderApiError> {
        let material = MaterialData {
            name: params.name.clone(),
            use_nodes: true,
            base_color: params.base_color,
            metallic: params.metallic,
            roughness: params.roughness,
            node_count: 1, // Basic principled BSDF
        };

        self.materials.insert(params.name, material);
        Ok(())
    }

    fn assign_material(&mut self, params: AssignMaterialParams) -> Result<(), BlenderApiError> {
        if !self.materials.contains_key(&params.material_name) {
            return Err(BlenderApiError::MaterialNotFound {
                name: params.material_name,
            });
        }

        if let Some(object) = self.objects.get_mut(&params.object_name) {
            if !object.materials.contains(&params.material_name) {
                object.materials.push(params.material_name);
            }
            Ok(())
        } else {
            Err(BlenderApiError::ObjectNotFound {
                name: params.object_name,
            })
        }
    }

    fn get_object(&self, params: GetObjectParams) -> Result<ObjectData, BlenderApiError> {
        self.objects
            .get(&params.name)
            .cloned()
            .ok_or(BlenderApiError::ObjectNotFound { name: params.name })
    }

    fn get_material(&self, params: GetMaterialParams) -> Result<MaterialData, BlenderApiError> {
        self.materials
            .get(&params.name)
            .cloned()
            .ok_or(BlenderApiError::MaterialNotFound { name: params.name })
    }

    fn list_objects(&self) -> Result<Vec<String>, BlenderApiError> {
        Ok(self.objects.keys().cloned().collect())
    }

    fn list_materials(&self) -> Result<Vec<String>, BlenderApiError> {
        Ok(self.materials.keys().cloned().collect())
    }

    fn list_meshes(&self) -> Result<Vec<String>, BlenderApiError> {
        Ok(self
            .objects
            .values()
            .filter(|obj| obj.object_type == "MESH")
            .map(|obj| obj.name.clone())
            .collect())
    }

    fn clear_scene(&mut self) -> Result<(), BlenderApiError> {
        self.objects.clear();
        // Note: materials are typically not cleared when clearing scene
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_cube() {
        let mut api = MockBlenderApi::new();

        let params = CreateCubeParams {
            location: Vec3::new(1.0, 2.0, 3.0),
            name: "TestCube".to_string(),
            size: 2.0,
        };

        api.create_cube(params).expect("Failed to create cube");

        let objects = api.list_objects().expect("Failed to list objects");
        assert_eq!(objects, vec!["TestCube"]);

        let cube = api
            .get_object(GetObjectParams {
                name: "TestCube".to_string(),
            })
            .expect("Failed to get cube");
        assert_eq!(cube.name, "TestCube");
        assert_eq!(cube.location.x, 1.0);
        assert_eq!(cube.vertex_count, Some(8));
    }

    #[test]
    fn test_create_material_and_assign() {
        let mut api = MockBlenderApi::new();

        // Create cube
        api.create_cube(CreateCubeParams {
            location: Vec3::zero(),
            name: "TestCube".to_string(),
            size: 1.0,
        })
        .expect("Failed to create cube");

        // Create material
        api.create_material(CreateMaterialParams {
            name: "TestMaterial".to_string(),
            base_color: Color::red(),
            metallic: 0.0,
            roughness: 0.5,
        })
        .expect("Failed to create material");

        // Assign material
        api.assign_material(AssignMaterialParams {
            object_name: "TestCube".to_string(),
            material_name: "TestMaterial".to_string(),
        })
        .expect("Failed to assign material");

        // Verify assignment
        let cube = api
            .get_object(GetObjectParams {
                name: "TestCube".to_string(),
            })
            .expect("Failed to get cube");
        assert_eq!(cube.materials, vec!["TestMaterial"]);
    }

    #[test]
    fn test_clear_scene() {
        let mut api = MockBlenderApi::new();

        // Create some objects
        api.create_cube(CreateCubeParams {
            location: Vec3::zero(),
            name: "Cube1".to_string(),
            size: 1.0,
        })
        .expect("Failed to create cube1");

        api.create_sphere(CreateSphereParams {
            location: Vec3::new(2.0, 0.0, 0.0),
            name: "Sphere1".to_string(),
            radius: 1.0,
            subdivisions: 2,
        })
        .expect("Failed to create sphere");

        let objects_before = api.list_objects().expect("Failed to list objects");
        assert_eq!(objects_before.len(), 2);

        // Clear scene
        api.clear_scene().expect("Failed to clear scene");

        let objects_after = api.list_objects().expect("Failed to list objects");
        assert_eq!(objects_after.len(), 0);
    }
}
