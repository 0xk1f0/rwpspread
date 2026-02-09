use std::collections::HashMap;
use std::process::Command;

pub struct Hyprpaper;
impl Hyprpaper {
    /// Generate and push appropriate commands to hyprpaper via hyprctl given new input wallpapers
    pub fn push(wallpapers: &HashMap<String, String>) -> Result<(), String> {
        for (monitor, path) in wallpapers {
            let output = Command::new("hyprctl")
                .args(["hyprpaper", "wallpaper", &format!("{},{}", monitor, path)])
                .output()
                .map_err(|e| format!("hyprpaper: failed to execute hyprctl: {}", e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!(
                    "hyprpaper: failed to set wallpaper for {}: {}",
                    monitor,
                    stderr.trim()
                ));
            }
        }

        Ok(())
    }
}
