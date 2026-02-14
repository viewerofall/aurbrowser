use std::process::Command;

pub fn spawn_yay_install(package_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Detect which terminal emulator is available
    let terminals = vec![
        ("alacritty", vec!["-e", "sh", "-c"]),
        ("kitty", vec!["-e", "sh", "-c"]),
        ("wezterm", vec!["start", "--", "sh", "-c"]),
        ("konsole", vec!["-e", "sh", "-c"]),
        ("gnome-terminal", vec!["--", "sh", "-c"]),
        ("xterm", vec!["-e", "sh", "-c"]),
    ];

    let yay_command = format!("yay -S --needed {}; echo '\nPress Enter to close...'; read", package_name);

    for (terminal, args) in terminals {
        if Command::new("which").arg(terminal).output()?.status.success() {
            let mut cmd = Command::new(terminal);
            for arg in args {
                cmd.arg(arg);
            }
            cmd.arg(&yay_command);
            cmd.spawn()?;
            return Ok(());
        }
    }

    Err("No supported terminal emulator found".into())
}
