//! # rext_core
//!
//! The `rext_core` crate is the library that powers Rext, the fullstack, batteries included Rust framework for developing web applications.
//!
//! It handles the absolute most basic requirements nearly all web apps will share, such as routing, API documentation, and the front-end.
//!
//! Status: 0%
//!
//! [Visit Rext](https://rextstack.org)
//!

mod error;
mod files;

use crate::error::RextCoreError;

// Re-export files module types and functions for public use
pub use crate::files::{
    FileCreationConfig, RextFile, RextFileType, RextModule, create_rext_app, get_rext_files,
};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::process::Command;

/// Constant list of data types to target (easily expandable)
pub const TYPES_TO_WRAP: [&str; 2] = ["Uuid", "DateTimeWithTimeZone"];

/// Directory containing generated sea-orm entity files
pub const ENTITIES_DIR: &str = "backend/entity/models";

/// Configuration for the server
pub struct ServerConfig {
    pub host: [u8; 4],
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: [0, 0, 0, 0],
            port: 3000,
        }
    }
}

/// Check if a Rext app has been initialized in the current directory by looking for the rext_app directory
///
/// Returns true if the rext_app directory exists, false otherwise.
///
/// # Example
///
/// ```rust
/// use rext_core::check_for_rext_app;
///
/// let is_rext_app = check_for_rext_app();
/// // This should be false as there is no Rext app here
/// assert!(!is_rext_app);
/// ```
pub fn check_for_rext_app() -> bool {
    let current_dir = std::env::current_dir().unwrap();
    let rext_app_dir = current_dir.join("rext.toml");
    rext_app_dir.exists()
}

/// Scaffold a new Rext application in the current directory
///
/// Creates the basic directory structure and config files needed for a Rext app.
/// This includes:
/// - rext.toml configuration file
/// - src/ directory for application code
/// - public/ directory for static assets
/// - templates/ directory for HTML templates
///
/// Returns an error if the app already exists or if there's an I/O error during creation.
///
/// # Example
///
/// ```rust
/// use rext_core::scaffold_rext_app;
///
/// match scaffold_rext_app() {
///     Ok(_) => println!("Rext app created successfully!"),
///     Err(e) => eprintln!("Failed to create app: {}", e),
/// }
/// ```
pub fn scaffold_rext_app() -> Result<(), RextCoreError> {
    let current_dir = std::env::current_dir().map_err(RextCoreError::CurrentDir)?;

    let new_app_name = current_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("my-rext-app")
        .to_string();

    // Create configuration with default settings
    let config = FileCreationConfig {
        app_name: new_app_name,
        modules: vec![RextModule::RextCore],
    };

    // Use the new files module to create the application
    create_rext_app(&current_dir, config)
}

/// Completely destroys a Rext application in the current directory
///
/// Removes all files and directories created by the scaffold_rext_app function.
///
/// Returns an error if there's an I/O error during destruction.
pub fn destroy_rext_app() -> Result<(), RextCoreError> {
    Ok(())
}

/// Generates the SeaORM entities with OpenAPI support
///
/// Adds the derive ToSchema and #[schema(value_type = String)] to unsupported data types
///
/// Returns a RextCoreError if an error occurs during the generation process
pub fn generate_sea_orm_entities_with_open_api_schema() -> Result<(), RextCoreError> {
    // run the see-orm-cli command with serde and utoipa derives
    let output = Command::new("sea-orm-cli")
        .args(&[
            "generate",
            "entity",
            "-u",
            "sqlite:./sqlite.db?mode=rwc",
            "-o",
            format!("{}", ENTITIES_DIR).as_str(),
            "--model-extra-derives",
            "utoipa::ToSchema",
            "--with-serde",
            "both",
        ])
        .output()
        .map_err(RextCoreError::SeaOrmCliGenerateEntities)?;

    if !output.status.success() {
        return Err(RextCoreError::SeaOrmCliGenerateEntities(
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("sea-orm-cli command failed with status: {}", output.status),
            ),
        ));
    }

    // Process each .rs file in the entities directory
    for entry in fs::read_dir(ENTITIES_DIR)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().map_or(false, |ext| ext == "rs") {
            // Check if this is a SeaORM entity file
            let file = File::open(&path)?;
            let reader = BufReader::new(file);
            let first_line = reader.lines().next().transpose()?;

            if let Some(line) = first_line {
                if !line.trim().starts_with("//! `SeaORM` Entity") {
                    continue;
                }
            } else {
                continue;
            }

            // Re-open file to process it
            let file = File::open(&path)?;
            let reader = BufReader::new(file);
            let mut output_lines: Vec<String> = Vec::new();

            for line_result in reader.lines() {
                let line = line_result?;
                let trimmed_line = line.trim_start();

                // Check if the line is a public field with a target type
                let mut add_schema = false;
                for dtype in &TYPES_TO_WRAP {
                    if trimmed_line.starts_with("pub ") && trimmed_line.contains(dtype) {
                        add_schema = true;
                        break;
                    }
                }

                // Insert the schema attribute if matched
                if add_schema {
                    output_lines.push("    #[schema(value_type = String)]".to_string());
                }

                output_lines.push(line);
            }

            // Write the modified content back to the file
            let mut file = File::create(&path)?;
            for line in &output_lines {
                writeln!(file, "{}", line)?;
            }
        }
    }

    Ok(())
}
