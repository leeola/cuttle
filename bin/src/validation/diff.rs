use anyhow::{Context, Result};
use serde_json::{Map, Value};
use std::fs;
use std::path::PathBuf;

pub async fn compare_states(
    baseline: PathBuf,
    current: PathBuf,
    format: String,
    output: Option<PathBuf>,
) -> Result<()> {
    println!("Comparing states:");
    println!("  Baseline: {}", baseline.display());
    println!("  Current: {}", current.display());

    // Load both state files
    let baseline_state = load_state_file(&baseline)?;
    let current_state = load_state_file(&current)?;

    // Perform comparison
    let diff_result = compare_json_states(&baseline_state, &current_state)?;

    // Format output
    let formatted_output = match format.as_str() {
        "json" => format_diff_as_json(&diff_result)?,
        "yaml" => format_diff_as_yaml(&diff_result)?,
        "text" => format_diff_as_text(&diff_result)?,
        _ => format_diff_as_text(&diff_result)?,
    };

    // Write output
    if let Some(output_path) = output {
        fs::write(&output_path, formatted_output).with_context(|| {
            format!("Failed to write diff output to: {}", output_path.display())
        })?;
        println!("Diff written to: {}", output_path.display());
    } else {
        println!("\n{formatted_output}");
    }

    // Print summary
    if diff_result.differences.is_empty() {
        println!("PASS: No differences found");
    } else {
        println!("FAIL: Found {} differences", diff_result.differences.len());
    }

    Ok(())
}

#[derive(Debug)]
pub struct DiffResult {
    pub differences: Vec<Difference>,
    pub baseline_only: Vec<String>,
    pub current_only: Vec<String>,
}

#[derive(Debug)]
pub struct Difference {
    pub path: String,
    pub baseline_value: Value,
    pub current_value: Value,
    pub diff_type: DiffType,
}

#[derive(Debug)]
pub enum DiffType {
    ValueChanged,
    TypeChanged,
    Added,
    Removed,
}

fn load_state_file(path: &PathBuf) -> Result<Value> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read state file: {}", path.display()))?;

    serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse JSON from: {}", path.display()))
}

fn compare_json_states(baseline: &Value, current: &Value) -> Result<DiffResult> {
    let mut result = DiffResult {
        differences: Vec::new(),
        baseline_only: Vec::new(),
        current_only: Vec::new(),
    };

    compare_values("", baseline, current, &mut result);

    Ok(result)
}

fn compare_values(path: &str, baseline: &Value, current: &Value, result: &mut DiffResult) {
    match (baseline, current) {
        (Value::Object(baseline_obj), Value::Object(current_obj)) => {
            compare_objects(path, baseline_obj, current_obj, result);
        }
        (Value::Array(baseline_arr), Value::Array(current_arr)) => {
            compare_arrays(path, baseline_arr, current_arr, result);
        }
        (baseline_val, current_val) => {
            if baseline_val != current_val {
                let diff_type = if std::mem::discriminant(baseline_val)
                    != std::mem::discriminant(current_val)
                {
                    DiffType::TypeChanged
                } else {
                    DiffType::ValueChanged
                };

                result.differences.push(Difference {
                    path: path.to_string(),
                    baseline_value: baseline_val.clone(),
                    current_value: current_val.clone(),
                    diff_type,
                });
            }
        }
    }
}

fn compare_objects(
    path: &str,
    baseline: &Map<String, Value>,
    current: &Map<String, Value>,
    result: &mut DiffResult,
) {
    // Find keys only in baseline
    for key in baseline.keys() {
        if !current.contains_key(key) {
            result.baseline_only.push(format!("{path}.{key}"));
        }
    }

    // Find keys only in current
    for key in current.keys() {
        if !baseline.contains_key(key) {
            result.current_only.push(format!("{path}.{key}"));
        }
    }

    // Compare common keys
    for (key, baseline_value) in baseline {
        if let Some(current_value) = current.get(key) {
            let new_path = if path.is_empty() {
                key.clone()
            } else {
                format!("{path}.{key}")
            };
            compare_values(&new_path, baseline_value, current_value, result);
        }
    }
}

fn compare_arrays(path: &str, baseline: &[Value], current: &[Value], result: &mut DiffResult) {
    // Simple array comparison by index
    let max_len = baseline.len().max(current.len());

    for i in 0..max_len {
        let new_path = format!("{path}[{i}]");

        match (baseline.get(i), current.get(i)) {
            (Some(baseline_val), Some(current_val)) => {
                compare_values(&new_path, baseline_val, current_val, result);
            }
            (Some(baseline_val), None) => {
                result.differences.push(Difference {
                    path: new_path,
                    baseline_value: baseline_val.clone(),
                    current_value: Value::Null,
                    diff_type: DiffType::Removed,
                });
            }
            (None, Some(current_val)) => {
                result.differences.push(Difference {
                    path: new_path,
                    baseline_value: Value::Null,
                    current_value: current_val.clone(),
                    diff_type: DiffType::Added,
                });
            }
            (None, None) => unreachable!(),
        }
    }
}

fn format_diff_as_text(diff: &DiffResult) -> Result<String> {
    let mut output = String::new();

    output.push_str("=== BLENDER STATE DIFF ===\n\n");

    if diff.differences.is_empty() && diff.baseline_only.is_empty() && diff.current_only.is_empty()
    {
        output.push_str("No differences found.\n");
        return Ok(output);
    }

    if !diff.differences.is_empty() {
        output.push_str("--- VALUE CHANGES ---\n");
        for diff in &diff.differences {
            output.push_str(&format!(
                "Path: {}\n  Baseline: {}\n  Current:  {}\n  Type: {:?}\n\n",
                diff.path, diff.baseline_value, diff.current_value, diff.diff_type
            ));
        }
    }

    if !diff.baseline_only.is_empty() {
        output.push_str("--- ONLY IN BASELINE ---\n");
        for path in &diff.baseline_only {
            output.push_str(&format!("  {path}\n"));
        }
        output.push('\n');
    }

    if !diff.current_only.is_empty() {
        output.push_str("--- ONLY IN CURRENT ---\n");
        for path in &diff.current_only {
            output.push_str(&format!("  {path}\n"));
        }
        output.push('\n');
    }

    output.push_str(&format!(
        "Summary: {} changes, {} baseline-only, {} current-only\n",
        diff.differences.len(),
        diff.baseline_only.len(),
        diff.current_only.len()
    ));

    Ok(output)
}

fn format_diff_as_json(diff: &DiffResult) -> Result<String> {
    let mut json_diff = Map::new();

    json_diff.insert(
        "differences".to_string(),
        Value::Array(
            diff.differences
                .iter()
                .map(|d| {
                    let mut obj = Map::new();
                    obj.insert("path".to_string(), Value::String(d.path.clone()));
                    obj.insert("baseline_value".to_string(), d.baseline_value.clone());
                    obj.insert("current_value".to_string(), d.current_value.clone());
                    obj.insert(
                        "diff_type".to_string(),
                        Value::String(format!("{:?}", d.diff_type)),
                    );
                    Value::Object(obj)
                })
                .collect(),
        ),
    );

    json_diff.insert(
        "baseline_only".to_string(),
        Value::Array(
            diff.baseline_only
                .iter()
                .map(|s| Value::String(s.clone()))
                .collect(),
        ),
    );

    json_diff.insert(
        "current_only".to_string(),
        Value::Array(
            diff.current_only
                .iter()
                .map(|s| Value::String(s.clone()))
                .collect(),
        ),
    );

    serde_json::to_string_pretty(&Value::Object(json_diff))
        .context("Failed to serialize diff as JSON")
}

fn format_diff_as_yaml(diff: &DiffResult) -> Result<String> {
    // For now, just convert JSON to YAML-like format
    // In a real implementation, you'd use a proper YAML library
    format_diff_as_json(diff)
}
