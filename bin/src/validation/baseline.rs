use crate::cli::BaselineCommands;
use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub async fn handle_baseline_command(command: BaselineCommands) -> Result<()> {
    match command {
        BaselineCommands::Set { source, name } => set_baseline(source, name).await,
        BaselineCommands::List => list_baselines().await,
        BaselineCommands::Show { name } => show_baseline(name).await,
        BaselineCommands::Remove { name } => remove_baseline(name).await,
    }
}

async fn set_baseline(source: PathBuf, name: String) -> Result<()> {
    println!("Setting baseline '{}' from: {}", name, source.display());

    // Verify source file exists and is valid JSON
    if !source.exists() {
        return Err(anyhow::anyhow!(
            "Source file does not exist: {}",
            source.display()
        ));
    }

    let content = fs::read_to_string(&source)
        .with_context(|| format!("Failed to read source file: {}", source.display()))?;

    // Validate JSON
    let _: Value = serde_json::from_str(&content)
        .with_context(|| format!("Source file is not valid JSON: {}", source.display()))?;

    // Create baselines directory
    let baselines_dir = get_baselines_dir()?;
    fs::create_dir_all(&baselines_dir).with_context(|| {
        format!(
            "Failed to create baselines directory: {}",
            baselines_dir.display()
        )
    })?;

    // Copy to baseline location
    let baseline_path = baselines_dir.join(format!("{name}.json"));
    fs::copy(&source, &baseline_path).with_context(|| {
        format!(
            "Failed to copy baseline file to: {}",
            baseline_path.display()
        )
    })?;

    // Update metadata
    update_baseline_metadata(&name, &source)?;

    println!("Baseline '{name}' set successfully");
    println!("Stored at: {}", baseline_path.display());

    Ok(())
}

async fn list_baselines() -> Result<()> {
    let baselines_dir = get_baselines_dir()?;

    if !baselines_dir.exists() {
        println!(
            "No baselines directory found. Use 'cuttle test baseline set' to create a baseline."
        );
        return Ok(());
    }

    let entries = fs::read_dir(&baselines_dir).with_context(|| {
        format!(
            "Failed to read baselines directory: {}",
            baselines_dir.display()
        )
    })?;

    let mut baselines = Vec::new();

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.extension().is_some_and(|ext| ext == "json") {
            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                let metadata = load_baseline_metadata(name).unwrap_or_default();
                baselines.push((name.to_string(), metadata));
            }
        }
    }

    if baselines.is_empty() {
        println!("No baselines found.");
        return Ok(());
    }

    println!("Available baselines:");
    println!("{:<20} {:<30} Source", "Name", "Created");
    println!("{:-<70}", "");

    for (name, metadata) in baselines {
        println!(
            "{:<20} {:<30} {}",
            name,
            metadata.get("created").unwrap_or(&"unknown".to_string()),
            metadata.get("source").unwrap_or(&"unknown".to_string())
        );
    }

    Ok(())
}

async fn show_baseline(name: String) -> Result<()> {
    let baselines_dir = get_baselines_dir()?;
    let baseline_path = baselines_dir.join(format!("{name}.json"));

    if !baseline_path.exists() {
        return Err(anyhow::anyhow!("Baseline '{}' not found", name));
    }

    let content = fs::read_to_string(&baseline_path)
        .with_context(|| format!("Failed to read baseline: {}", baseline_path.display()))?;

    let state: Value = serde_json::from_str(&content)
        .with_context(|| format!("Invalid JSON in baseline: {}", baseline_path.display()))?;

    println!("Baseline: {name}");
    println!("Path: {}", baseline_path.display());

    // Load and show metadata
    let metadata = load_baseline_metadata(&name).unwrap_or_default();
    if !metadata.is_empty() {
        println!("\nMetadata:");
        for (key, value) in metadata {
            println!("  {key}: {value}");
        }
    }

    // Show summary statistics
    println!("\nContent Summary:");
    show_state_summary(&state);

    Ok(())
}

async fn remove_baseline(name: String) -> Result<()> {
    let baselines_dir = get_baselines_dir()?;
    let baseline_path = baselines_dir.join(format!("{name}.json"));
    let metadata_path = baselines_dir.join(format!("{name}.meta"));

    if !baseline_path.exists() {
        return Err(anyhow::anyhow!("Baseline '{}' not found", name));
    }

    fs::remove_file(&baseline_path).with_context(|| {
        format!(
            "Failed to remove baseline file: {}",
            baseline_path.display()
        )
    })?;

    // Remove metadata if it exists
    if metadata_path.exists() {
        fs::remove_file(&metadata_path).with_context(|| {
            format!(
                "Failed to remove metadata file: {}",
                metadata_path.display()
            )
        })?;
    }

    println!("Baseline '{name}' removed successfully");

    Ok(())
}

fn get_baselines_dir() -> Result<PathBuf> {
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;

    Ok(current_dir.join("baselines"))
}

fn update_baseline_metadata(name: &str, source: &Path) -> Result<()> {
    let baselines_dir = get_baselines_dir()?;
    let metadata_path = baselines_dir.join(format!("{name}.meta"));

    let mut metadata = HashMap::new();
    metadata.insert("name".to_string(), name.to_string());
    metadata.insert("source".to_string(), source.display().to_string());
    metadata.insert(
        "created".to_string(),
        chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string(),
    );

    let metadata_content =
        serde_json::to_string_pretty(&metadata).context("Failed to serialize metadata")?;

    fs::write(&metadata_path, metadata_content)
        .with_context(|| format!("Failed to write metadata: {}", metadata_path.display()))?;

    Ok(())
}

fn load_baseline_metadata(name: &str) -> Result<HashMap<String, String>> {
    let baselines_dir = get_baselines_dir()?;
    let metadata_path = baselines_dir.join(format!("{name}.meta"));

    if !metadata_path.exists() {
        return Ok(HashMap::new());
    }

    let content = fs::read_to_string(&metadata_path)
        .with_context(|| format!("Failed to read metadata: {}", metadata_path.display()))?;

    serde_json::from_str(&content).context("Failed to parse metadata JSON")
}

fn show_state_summary(state: &Value) {
    match state {
        Value::Object(obj) => {
            println!("  Type: Object");
            println!("  Keys: {}", obj.len());

            // Show specific Blender data if present
            if let Some(objects) = obj.get("objects").and_then(|v| v.as_array()) {
                println!("  Objects: {}", objects.len());
            }
            if let Some(materials) = obj.get("materials").and_then(|v| v.as_array()) {
                println!("  Materials: {}", materials.len());
            }
            if let Some(meshes) = obj.get("meshes").and_then(|v| v.as_array()) {
                println!("  Meshes: {}", meshes.len());
            }
        }
        Value::Array(arr) => {
            println!("  Type: Array");
            println!("  Length: {}", arr.len());
        }
        _ => {
            println!(
                "  Type: {}",
                match state {
                    Value::String(_) => "String",
                    Value::Number(_) => "Number",
                    Value::Bool(_) => "Boolean",
                    Value::Null => "Null",
                    _ => "Unknown",
                }
            );
        }
    }
}
