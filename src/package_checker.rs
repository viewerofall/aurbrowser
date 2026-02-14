use std::process::Command;
use std::collections::HashSet;

pub fn get_installed_packages() -> HashSet<String> {
    let mut installed = HashSet::new();
    
    if let Ok(output) = Command::new("pacman").arg("-Qq").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                installed.insert(line.trim().to_string());
            }
        }
    }
    
    installed
}

pub fn is_package_installed(package_name: &str) -> bool {
    Command::new("pacman")
        .arg("-Qq")
        .arg(package_name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
