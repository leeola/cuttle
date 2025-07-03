use cuttle_lang::{BlenderNodeGraph, parse_geometry_nodes, parse_geometry_nodes_with_errors};

fn main() {
    // Parse a simple cube node
    let cube_input = "cube { size: 2.0 }";
    let cube_graph = parse_geometry_nodes(cube_input).expect("Failed to parse cube");
    println!("Parsed cube node: {cube_graph:?}");

    // Convert to Blender format
    let blender_cube: BlenderNodeGraph = cube_graph.into();
    println!("Blender format: {blender_cube:?}");

    // Parse a value node
    let value_input = "value 42";
    let value_graph = parse_geometry_nodes(value_input).expect("Failed to parse value");
    println!("Parsed value node: {value_graph:?}");

    // Convert to Blender format
    let blender_value: BlenderNodeGraph = value_graph.into();
    println!("Blender format: {blender_value:?}");

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&blender_value).expect("Failed to serialize to JSON");
    println!("JSON representation:\n{json}");

    // Demonstrate error reporting
    println!("\n--- Error Reporting Examples ---");

    // Invalid syntax
    let invalid_input = "invalid syntax here";
    match parse_geometry_nodes_with_errors(invalid_input) {
        Ok(_) => println!("Unexpected success"),
        Err(error_report) => {
            println!("Error parsing '{invalid_input}':");
            println!("{error_report}");
        }
    }

    // Invalid vector
    let invalid_vector = "value (1, 2)";
    match parse_geometry_nodes_with_errors(invalid_vector) {
        Ok(_) => println!("Unexpected success"),
        Err(error_report) => {
            println!("Error parsing '{invalid_vector}':");
            println!("{error_report}");
        }
    }

    // Invalid number
    let invalid_number = "value abc";
    match parse_geometry_nodes_with_errors(invalid_number) {
        Ok(_) => println!("Unexpected success"),
        Err(error_report) => {
            println!("Error parsing '{invalid_number}':");
            println!("{error_report}");
        }
    }
}
