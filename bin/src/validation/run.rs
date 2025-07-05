use crate::validation::suite::{
    ValidationCase, ValidationStep, get_validation_by_name, get_validation_suite,
};
use anyhow::{Context, Result};
use cuttle::{PyBridge, ServiceMessage, ServiceResponse};
use cuttle_blender_api::{
    AssignMaterialParams, CreateCubeParams, CreateMaterialParams, CreateSphereParams,
    GetObjectParams,
};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use tokio::time::{Duration, timeout};

pub async fn run_validations(
    name: Option<String>,
    output: PathBuf,
    compare_baseline: bool,
    timeout_seconds: u64,
) -> Result<()> {
    println!("Running validations...");
    println!("Output directory: {}", output.display());

    // Create output directory if it doesn't exist
    fs::create_dir_all(&output)
        .with_context(|| format!("Failed to create output directory: {}", output.display()))?;

    // Get validations to run
    let validations = if let Some(validation_name) = name {
        if let Some(validation) = get_validation_by_name(&validation_name) {
            vec![validation]
        } else {
            return Err(anyhow::anyhow!(
                "Validation '{}' not found. Use 'cuttle validation list' to see available validations.",
                validation_name
            ));
        }
    } else {
        get_validation_suite()
    };

    println!("Running {} validation(s)", validations.len());

    // Start Cuttle service
    let (mut bridge, async_bridge) = PyBridge::new();
    bridge.start_runtime(async_bridge);

    // Give the runtime a moment to start up
    tokio::time::sleep(Duration::from_millis(100)).await;

    let mut all_passed = true;
    let mut results = Vec::new();

    // Run each validation
    for validation in validations {
        println!("\n--- Running validation: {} ---", validation.name);
        println!("Description: {}", validation.description);

        let result = run_validation(&mut bridge, &validation, &output, timeout_seconds).await?;

        if result.success {
            println!("PASS: {} completed successfully", result.name);
        } else {
            println!("FAIL: {} failed", result.name);
            if let Some(error) = &result.error {
                println!("Error: {error}");
            }
            all_passed = false;
        }

        results.push(result);
    }

    // Clean shutdown
    bridge.stop();

    // Summary
    println!("\n=== Validation Summary ===");
    let passed = results.iter().filter(|r| r.success).count();
    let total = results.len();
    println!("Passed: {passed}/{total}");

    for result in &results {
        let status = if result.success { "PASS" } else { "FAIL" };
        println!("  {} {}", status, result.name);
    }

    if compare_baseline && all_passed {
        println!("\nComparing against baseline...");
        // TODO: Implement baseline comparison
    }

    if !all_passed {
        return Err(anyhow::anyhow!("{} validation(s) failed", total - passed));
    }

    println!("\nAll validations passed!");
    Ok(())
}

pub struct ValidationResult {
    pub name: String,
    pub success: bool,
    pub state_file: Option<PathBuf>,
    pub error: Option<String>,
    pub duration: Duration,
}

async fn run_validation(
    bridge: &mut PyBridge,
    validation: &ValidationCase,
    output_dir: &Path,
    timeout_seconds: u64,
) -> Result<ValidationResult> {
    let start_time = std::time::Instant::now();

    // Execute validation steps
    let mut success = true;
    let mut error_message = None;

    for (i, step) in validation.steps.iter().enumerate() {
        match execute_validation_step(bridge, step.clone(), timeout_seconds).await {
            Ok(_) => {
                println!("  Step {}/{}: PASS", i + 1, validation.steps.len());
            }
            Err(e) => {
                success = false;
                error_message = Some(e.to_string());
                println!("  Step {}/{}: FAIL - {}", i + 1, validation.steps.len(), e);
                break;
            }
        }
    }

    // Capture final state if successful
    let state_file = if success {
        match capture_scene_state(
            bridge,
            output_dir,
            &format!("{}_state.json", validation.name),
            timeout_seconds,
        )
        .await
        {
            Ok(file) => Some(file),
            Err(e) => {
                println!("Warning: Failed to capture scene state: {e}");
                None
            }
        }
    } else {
        None
    };

    // Validate expectations if successful
    if success {
        if let Err(e) = validate_expectations(bridge, validation, timeout_seconds).await {
            success = false;
            error_message = Some(format!("Expectation validation failed: {e}"));
        }
    }

    let duration = start_time.elapsed();

    Ok(ValidationResult {
        name: validation.name.to_string(),
        success,
        state_file,
        error: error_message,
        duration,
    })
}

async fn execute_validation_step(
    bridge: &mut PyBridge,
    step: ValidationStep,
    timeout_seconds: u64,
) -> Result<()> {
    let message = match step {
        ValidationStep::ClearScene => ServiceMessage::ClearScene,
        ValidationStep::CreateCube {
            name,
            location,
            size,
        } => ServiceMessage::CreateCube(CreateCubeParams {
            name,
            location,
            size,
        }),
        ValidationStep::CreateSphere {
            name,
            location,
            radius,
            subdivisions,
        } => ServiceMessage::CreateSphere(CreateSphereParams {
            name,
            location,
            radius,
            subdivisions,
        }),
        ValidationStep::CreateMaterial {
            name,
            color,
            metallic,
            roughness,
        } => ServiceMessage::CreateMaterial(CreateMaterialParams {
            name,
            base_color: color,
            metallic,
            roughness,
        }),
        ValidationStep::AssignMaterial {
            object_name,
            material_name,
        } => ServiceMessage::AssignMaterial(AssignMaterialParams {
            object_name,
            material_name,
        }),
    };

    // Send message
    bridge
        .send(message)
        .context("Failed to send message to service")?;

    // Wait for response with timeout
    let response = timeout(Duration::from_secs(timeout_seconds), async {
        loop {
            if let Some(response) = bridge.try_recv() {
                return response;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .context("Validation step timed out")?;

    // Check response
    match response {
        ServiceResponse::Created | ServiceResponse::SceneCleared => Ok(()),
        ServiceResponse::Error(e) => Err(anyhow::anyhow!("Service error: {}", e)),
        _ => Err(anyhow::anyhow!("Unexpected response: {:?}", response)),
    }
}

async fn validate_expectations(
    bridge: &mut PyBridge,
    validation: &ValidationCase,
    timeout_seconds: u64,
) -> Result<()> {
    // Check expected objects exist
    for expected_object in &validation.expected_objects {
        bridge
            .send(ServiceMessage::GetObject(GetObjectParams {
                name: expected_object.to_string(),
            }))
            .context("Failed to send get object message")?;

        let response = timeout(Duration::from_secs(timeout_seconds), async {
            loop {
                if let Some(response) = bridge.try_recv() {
                    return response;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .context("Get object timed out")?;

        match response {
            ServiceResponse::ObjectData(_) => {
                println!("    Expected object '{expected_object}': FOUND");
            }
            ServiceResponse::Error(_) => {
                return Err(anyhow::anyhow!(
                    "Expected object '{}' not found",
                    expected_object
                ));
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "Unexpected response when checking object '{}'",
                    expected_object
                ));
            }
        }
    }

    // Check expected materials exist
    for expected_material in &validation.expected_materials {
        bridge
            .send(ServiceMessage::GetMaterial(
                cuttle_blender_api::GetMaterialParams {
                    name: expected_material.to_string(),
                },
            ))
            .context("Failed to send get material message")?;

        let response = timeout(Duration::from_secs(timeout_seconds), async {
            loop {
                if let Some(response) = bridge.try_recv() {
                    return response;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .context("Get material timed out")?;

        match response {
            ServiceResponse::MaterialData(_) => {
                println!("    Expected material '{expected_material}': FOUND");
            }
            ServiceResponse::Error(_) => {
                return Err(anyhow::anyhow!(
                    "Expected material '{}' not found",
                    expected_material
                ));
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "Unexpected response when checking material '{}'",
                    expected_material
                ));
            }
        }
    }

    Ok(())
}

async fn capture_scene_state(
    bridge: &mut PyBridge,
    output_dir: &Path,
    filename: &str,
    timeout_seconds: u64,
) -> Result<PathBuf> {
    // Query objects and materials
    let objects = query_objects(bridge, timeout_seconds).await?;
    let materials = query_materials(bridge, timeout_seconds).await?;

    // Get detailed object data
    let mut object_data = Vec::new();
    for object_name in &objects {
        match query_object_details(bridge, object_name, timeout_seconds).await {
            Ok(data) => object_data.push(data),
            Err(e) => println!("Warning: Failed to get details for object {object_name}: {e}"),
        }
    }

    // Get detailed material data
    let mut material_data = Vec::new();
    for material_name in &materials {
        match query_material_details(bridge, material_name, timeout_seconds).await {
            Ok(data) => material_data.push(data),
            Err(e) => println!("Warning: Failed to get details for material {material_name}: {e}"),
        }
    }

    // Create state JSON
    let state = serde_json::json!({
        "objects": object_data,
        "materials": material_data,
        "object_count": objects.len(),
        "material_count": materials.len(),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    // Write state to file
    let state_file = output_dir.join(filename);
    let state_content =
        serde_json::to_string_pretty(&state).context("Failed to serialize state to JSON")?;

    fs::write(&state_file, state_content)
        .with_context(|| format!("Failed to write state file: {}", state_file.display()))?;

    println!("  Scene state captured to: {}", state_file.display());
    Ok(state_file)
}

async fn query_objects(bridge: &mut PyBridge, timeout_seconds: u64) -> Result<Vec<String>> {
    bridge
        .send(ServiceMessage::ListObjects)
        .context("Failed to send list objects message")?;

    let response = timeout(Duration::from_secs(timeout_seconds), async {
        loop {
            if let Some(response) = bridge.try_recv() {
                return response;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .context("List objects timed out")?;

    match response {
        ServiceResponse::ObjectList(objects) => Ok(objects),
        ServiceResponse::Error(e) => Err(anyhow::anyhow!("Service error: {}", e)),
        _ => Err(anyhow::anyhow!("Unexpected response: {:?}", response)),
    }
}

async fn query_materials(bridge: &mut PyBridge, timeout_seconds: u64) -> Result<Vec<String>> {
    bridge
        .send(ServiceMessage::ListMaterials)
        .context("Failed to send list materials message")?;

    let response = timeout(Duration::from_secs(timeout_seconds), async {
        loop {
            if let Some(response) = bridge.try_recv() {
                return response;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .context("List materials timed out")?;

    match response {
        ServiceResponse::MaterialList(materials) => Ok(materials),
        ServiceResponse::Error(e) => Err(anyhow::anyhow!("Service error: {}", e)),
        _ => Err(anyhow::anyhow!("Unexpected response: {:?}", response)),
    }
}

async fn query_object_details(
    bridge: &mut PyBridge,
    object_name: &str,
    timeout_seconds: u64,
) -> Result<Value> {
    bridge
        .send(ServiceMessage::GetObject(GetObjectParams {
            name: object_name.to_string(),
        }))
        .context("Failed to send get object message")?;

    let response = timeout(Duration::from_secs(timeout_seconds), async {
        loop {
            if let Some(response) = bridge.try_recv() {
                return response;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .context("Get object timed out")?;

    match response {
        ServiceResponse::ObjectData(data) => {
            serde_json::to_value(data).context("Failed to serialize object data")
        }
        ServiceResponse::Error(e) => Err(anyhow::anyhow!("Service error: {}", e)),
        _ => Err(anyhow::anyhow!("Unexpected response: {:?}", response)),
    }
}

async fn query_material_details(
    bridge: &mut PyBridge,
    material_name: &str,
    timeout_seconds: u64,
) -> Result<Value> {
    bridge
        .send(ServiceMessage::GetMaterial(
            cuttle_blender_api::GetMaterialParams {
                name: material_name.to_string(),
            },
        ))
        .context("Failed to send get material message")?;

    let response = timeout(Duration::from_secs(timeout_seconds), async {
        loop {
            if let Some(response) = bridge.try_recv() {
                return response;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .context("Get material timed out")?;

    match response {
        ServiceResponse::MaterialData(data) => {
            serde_json::to_value(data).context("Failed to serialize material data")
        }
        ServiceResponse::Error(e) => Err(anyhow::anyhow!("Service error: {}", e)),
        _ => Err(anyhow::anyhow!("Unexpected response: {:?}", response)),
    }
}
