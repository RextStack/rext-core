use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=frontend/src");
    println!("cargo:rerun-if-changed=frontend/package.json");
    println!("cargo:rerun-if-changed=frontend/vite.config.ts");
    println!("cargo:rerun-if-changed=frontend/tsconfig.json");
    println!("cargo:rerun-if-changed=.env");

    // Load .env file if it exists
    if let Ok(contents) = fs::read_to_string(".env") {
        for line in contents.lines() {
            let line = line.trim();
            if !line.is_empty() && !line.starts_with('#') {
                if let Some((key, value)) = line.split_once('=') {
                    let key = key.trim();
                    let value = value.trim();
                    unsafe {
                        env::set_var(key, value);
                    }
                }
            }
        }
    }

    // Only build frontend in production mode or when explicitly requested
    let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
    let force_build = env::var("BUILD_FRONTEND").unwrap_or_else(|_| "false".to_string());

    if environment == "production" || force_build == "true" {
        println!("cargo:warning=Building frontend assets for production...");
        build_frontend();
    } else {
        println!("cargo:warning=Skipping frontend build in development mode");
        println!(
            "cargo:warning=Set ENVIRONMENT=production or BUILD_FRONTEND=true to build frontend"
        );
        return;
    }
}

fn build_frontend() {
    let frontend_dir = Path::new("frontend");

    if !frontend_dir.exists() {
        panic!("Frontend directory not found at ./frontend");
    }

    // Check if Node.js is available
    if !is_command_available("npm") {
        panic!("npm is not available. Please install Node.js and npm to build the frontend.");
    }

    // Install dependencies if node_modules doesn't exist
    let node_modules = frontend_dir.join("node_modules");
    if !node_modules.exists() {
        println!("cargo:warning=Installing frontend dependencies...");
        let npm_install = Command::new("npm")
            .args(["install"])
            .current_dir(frontend_dir)
            .status()
            .expect("Failed to execute npm install");

        if !npm_install.success() {
            panic!("npm install failed");
        }
    }

    // Build frontend
    println!("cargo:warning=Building frontend with npm run build...");
    let npm_build = Command::new("npm")
        .args(["run", "build"])
        .current_dir(frontend_dir)
        .status()
        .expect("Failed to execute npm run build");

    if !npm_build.success() {
        panic!("Frontend build failed");
    }

    // Copy built frontend to dist directory
    let frontend_dist = frontend_dir.join("dist");
    let target_dist = Path::new("dist");

    if frontend_dist.exists() {
        println!("cargo:warning=Copying frontend assets to ./dist");

        // Remove existing dist directory if it exists
        if target_dist.exists() {
            fs::remove_dir_all(target_dist).expect("Failed to remove existing dist directory");
        }

        // Copy the built frontend
        copy_dir_all(&frontend_dist, target_dist).expect("Failed to copy frontend assets");

        println!("cargo:warning=Frontend build completed successfully!");
    } else {
        panic!("Frontend build completed but dist directory not found");
    }
}

fn is_command_available(command: &str) -> bool {
    Command::new("which")
        .arg(command)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;

        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }

    Ok(())
}
