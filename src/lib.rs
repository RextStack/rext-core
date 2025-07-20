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

use crate::error::RextCoreError;

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

    // Confirm a rust project does not already exist in this directory
    // (helps prevent accidental overwriting of existing projects, including rext-core, oopsies)
    if current_dir.join("Cargo.toml").exists() {
        return Err(RextCoreError::AppAlreadyExists);
    }

    // Check if rext.toml already exists
    if current_dir.join("rext.toml").exists() {
        return Err(RextCoreError::AppAlreadyExists);
    }

    // Create basic directory structure
    let src_dir = current_dir.join("src");
    let public_dir = current_dir.join("public");
    let templates_dir = current_dir.join("templates");

    // Create directories
    std::fs::create_dir_all(&src_dir).map_err(RextCoreError::DirectoryCreation)?;
    std::fs::create_dir_all(&public_dir).map_err(RextCoreError::DirectoryCreation)?;
    std::fs::create_dir_all(&templates_dir).map_err(RextCoreError::DirectoryCreation)?;

    // Create rext.toml configuration file
    let rext_toml_content = r#"[app]
name = "my-rext-app"
version = "0.1.0"
description = "A new Rext application"

[server]
host = "0.0.0.0"
port = 3000

[database]
url = "sqlite://rext.db"

[static]
directory = "public"

[templates]
directory = "templates"
"#;

    let rext_toml_path = current_dir.join("rext.toml");
    std::fs::write(&rext_toml_path, rext_toml_content)
        .map_err(|e| RextCoreError::FileWrite(format!("rext.toml: {}", e)))?;

    // Create a basic Cargo.toml file
    let cargo_toml_content = format!(
        r#"
[package]
name = "{}"
version = "0.1.0"
description = "A new Rext application"

[dependencies]
rext-core = "0.1.0"
"#,
        current_dir.to_str().unwrap()
    );

    let cargo_toml_path = current_dir.join("Cargo.toml");
    std::fs::write(&cargo_toml_path, cargo_toml_content)
        .map_err(|e| RextCoreError::FileWrite(format!("Cargo.toml: {}", e)))?;

    // Create a basic main.rs file
    let main_rs_content = r#"

fn main() {
    println!("Welcome to your new Rext app!");
}
"#;

    let main_rs_path = src_dir.join("main.rs");
    std::fs::write(&main_rs_path, main_rs_content)
        .map_err(|e| RextCoreError::FileWrite(format!("src/main.rs: {}", e)))?;

    // Create a basic index.html template
    let index_html_content = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>My Rext App</title>
</head>
<body>
    <h1>Welcome to Rext!</h1>
    <p>Your fullstack Rust web application is ready.</p>
</body>
</html>
"#;

    let index_html_path = templates_dir.join("index.html");
    std::fs::write(&index_html_path, index_html_content)
        .map_err(|e| RextCoreError::FileWrite(format!("templates/index.html: {}", e)))?;

    // Create a basic CSS file
    let style_css_content = r#"body {
    font-family: Arial, sans-serif;
    max-width: 800px;
    margin: 0 auto;
    padding: 2rem;
    line-height: 1.6;
}

h1 {
    color: #333;
    text-align: center;
}

p {
    color: #666;
    text-align: center;
}
"#;

    let style_css_path = public_dir.join("style.css");
    std::fs::write(&style_css_path, style_css_content)
        .map_err(|e| RextCoreError::FileWrite(format!("public/style.css: {}", e)))?;

    Ok(())
}

/// Completely destroys a Rext application in the current directory
///
/// Removes all files and directories created by the scaffold_rext_app function.
///
/// Returns an error if there's an I/O error during destruction.
pub fn destroy_rext_app() -> Result<(), RextCoreError> {
    let current_dir = std::env::current_dir().map_err(RextCoreError::CurrentDir)?;

    // Files and directories that scaffold_rext_app creates
    let rext_toml_path = current_dir.join("rext.toml");
    let cargo_toml_path = current_dir.join("Cargo.toml");
    let src_dir = current_dir.join("src");
    let public_dir = current_dir.join("public");
    let templates_dir = current_dir.join("templates");
    let main_rs_path = src_dir.join("main.rs");
    let index_html_path = templates_dir.join("index.html");
    let style_css_path = public_dir.join("style.css");

    // Safety check: Verify that directories only contain expected files

    // Check src/ directory
    if src_dir.exists() {
        let src_entries: Result<Vec<_>, _> = std::fs::read_dir(&src_dir)
            .map_err(RextCoreError::DirectoryRead)?
            .collect();
        let src_entries = src_entries.map_err(RextCoreError::DirectoryRead)?;

        if src_entries.len() != 1
            || !src_entries.iter().any(|entry| {
                entry.file_name() == "main.rs"
                    && entry.file_type().map(|ft| ft.is_file()).unwrap_or(false)
            })
        {
            return Err(RextCoreError::SafetyCheck(
                "src directory contains unexpected files".to_string(),
            ));
        }
    }

    // Check public/ directory
    if public_dir.exists() {
        let public_entries: Result<Vec<_>, _> = std::fs::read_dir(&public_dir)
            .map_err(RextCoreError::DirectoryRead)?
            .collect();
        let public_entries = public_entries.map_err(RextCoreError::DirectoryRead)?;

        if public_entries.len() != 1
            || !public_entries.iter().any(|entry| {
                entry.file_name() == "style.css"
                    && entry.file_type().map(|ft| ft.is_file()).unwrap_or(false)
            })
        {
            return Err(RextCoreError::SafetyCheck(
                "public directory contains unexpected files".to_string(),
            ));
        }
    }

    // Check templates/ directory
    if templates_dir.exists() {
        let templates_entries: Result<Vec<_>, _> = std::fs::read_dir(&templates_dir)
            .map_err(RextCoreError::DirectoryRead)?
            .collect();
        let templates_entries = templates_entries.map_err(RextCoreError::DirectoryRead)?;

        if templates_entries.len() != 1
            || !templates_entries.iter().any(|entry| {
                entry.file_name() == "index.html"
                    && entry.file_type().map(|ft| ft.is_file()).unwrap_or(false)
            })
        {
            return Err(RextCoreError::SafetyCheck(
                "templates directory contains unexpected files".to_string(),
            ));
        }
    }

    // If we've reached here, all directories contain only expected files
    // Now remove all files and directories in reverse order of creation

    // Remove files first
    if style_css_path.exists() {
        std::fs::remove_file(&style_css_path)
            .map_err(|e| RextCoreError::FileRemoval(format!("public/style.css: {}", e)))?;
    }

    if index_html_path.exists() {
        std::fs::remove_file(&index_html_path)
            .map_err(|e| RextCoreError::FileRemoval(format!("templates/index.html: {}", e)))?;
    }

    if main_rs_path.exists() {
        std::fs::remove_file(&main_rs_path)
            .map_err(|e| RextCoreError::FileRemoval(format!("src/main.rs: {}", e)))?;
    }

    if cargo_toml_path.exists() {
        std::fs::remove_file(&cargo_toml_path)
            .map_err(|e| RextCoreError::FileRemoval(format!("Cargo.toml: {}", e)))?;
    }

    if rext_toml_path.exists() {
        std::fs::remove_file(&rext_toml_path)
            .map_err(|e| RextCoreError::FileRemoval(format!("rext.toml: {}", e)))?;
    }

    // Remove directories (they should now be empty)
    if templates_dir.exists() {
        std::fs::remove_dir(&templates_dir)
            .map_err(|e| RextCoreError::DirectoryRemoval(format!("templates: {}", e)))?;
    }

    if public_dir.exists() {
        std::fs::remove_dir(&public_dir)
            .map_err(|e| RextCoreError::DirectoryRemoval(format!("public: {}", e)))?;
    }

    if src_dir.exists() {
        std::fs::remove_dir(&src_dir)
            .map_err(|e| RextCoreError::DirectoryRemoval(format!("src: {}", e)))?;
    }

    Ok(())
}
