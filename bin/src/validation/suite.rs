use cuttle_blender_api::{Color, Vec3};

#[derive(Debug, Clone)]
pub struct ValidationCase {
    pub name: &'static str,
    pub description: &'static str,
    pub steps: Vec<ValidationStep>,
    pub expected_objects: Vec<&'static str>,
    pub expected_materials: Vec<&'static str>,
}

#[derive(Debug, Clone)]
pub enum ValidationStep {
    ClearScene,
    CreateCube {
        name: String,
        location: Vec3,
        size: f32,
    },
    CreateSphere {
        name: String,
        location: Vec3,
        radius: f32,
        subdivisions: u32,
    },
    CreateMaterial {
        name: String,
        color: Color,
        metallic: f32,
        roughness: f32,
    },
    AssignMaterial {
        object_name: String,
        material_name: String,
    },
}

pub fn get_validation_suite() -> Vec<ValidationCase> {
    vec![
        ValidationCase {
            name: "basic_geometry",
            description: "Validate basic cube creation with material assignment",
            steps: vec![
                ValidationStep::ClearScene,
                ValidationStep::CreateCube {
                    name: "TestCube".to_string(),
                    location: Vec3::new(0.0, 0.0, 0.0),
                    size: 2.0,
                },
                ValidationStep::CreateMaterial {
                    name: "TestMaterial".to_string(),
                    color: Color::red(),
                    metallic: 0.0,
                    roughness: 0.5,
                },
                ValidationStep::AssignMaterial {
                    object_name: "TestCube".to_string(),
                    material_name: "TestMaterial".to_string(),
                },
            ],
            expected_objects: vec!["TestCube"],
            expected_materials: vec!["TestMaterial"],
        },
        ValidationCase {
            name: "multi_object",
            description: "Validate multiple objects with different materials",
            steps: vec![
                ValidationStep::ClearScene,
                ValidationStep::CreateCube {
                    name: "RedCube".to_string(),
                    location: Vec3::new(-2.0, 0.0, 0.0),
                    size: 1.0,
                },
                ValidationStep::CreateSphere {
                    name: "BlueSphere".to_string(),
                    location: Vec3::new(2.0, 0.0, 0.0),
                    radius: 1.0,
                    subdivisions: 3,
                },
                ValidationStep::CreateMaterial {
                    name: "RedMaterial".to_string(),
                    color: Color::new(0.8, 0.2, 0.2, 1.0),
                    metallic: 0.0,
                    roughness: 0.4,
                },
                ValidationStep::CreateMaterial {
                    name: "BlueMaterial".to_string(),
                    color: Color::new(0.2, 0.2, 0.8, 1.0),
                    metallic: 0.1,
                    roughness: 0.3,
                },
                ValidationStep::AssignMaterial {
                    object_name: "RedCube".to_string(),
                    material_name: "RedMaterial".to_string(),
                },
                ValidationStep::AssignMaterial {
                    object_name: "BlueSphere".to_string(),
                    material_name: "BlueMaterial".to_string(),
                },
            ],
            expected_objects: vec!["RedCube", "BlueSphere"],
            expected_materials: vec!["RedMaterial", "BlueMaterial"],
        },
        ValidationCase {
            name: "material_properties",
            description: "Validate different material properties and metallic/roughness values",
            steps: vec![
                ValidationStep::ClearScene,
                ValidationStep::CreateCube {
                    name: "MetallicCube".to_string(),
                    location: Vec3::new(0.0, 0.0, 0.0),
                    size: 1.0,
                },
                ValidationStep::CreateMaterial {
                    name: "MetallicMaterial".to_string(),
                    color: Color::new(0.7, 0.7, 0.7, 1.0),
                    metallic: 1.0,
                    roughness: 0.1,
                },
                ValidationStep::AssignMaterial {
                    object_name: "MetallicCube".to_string(),
                    material_name: "MetallicMaterial".to_string(),
                },
            ],
            expected_objects: vec!["MetallicCube"],
            expected_materials: vec!["MetallicMaterial"],
        },
    ]
}

pub fn list_validations() {
    let suite = get_validation_suite();

    println!("Available validations:");
    println!("{:<20} Description", "Name");
    println!("{:-<70}", "");

    for validation in suite {
        println!("{:<20} {}", validation.name, validation.description);
    }

    println!("\nUsage:");
    println!("  cuttle validation run                    # Run all validations");
    println!("  cuttle validation run basic_geometry     # Run specific validation");
}

pub fn get_validation_by_name(name: &str) -> Option<ValidationCase> {
    get_validation_suite().into_iter().find(|v| v.name == name)
}
