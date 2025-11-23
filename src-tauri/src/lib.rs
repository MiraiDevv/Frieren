use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Debug, Serialize, Deserialize)]
struct VideoFormat {
    format_id: String,
    format_note: Option<String>,
    ext: String,
    resolution: Option<String>,
    height: Option<u32>,
    width: Option<u32>,
    vcodec: Option<String>,
    acodec: Option<String>,
}

#[derive(Debug, Serialize)]
struct QualityOption {
    id: String,
    label: String,
    format_type: String, // "video+audio", "video", "audio"
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn get_video_info(url: String) -> Result<Vec<QualityOption>, String> {
    println!("Fetching video info for: {}", url);

    let bin_path = Path::new("bin/yt-dlp.exe");
    if !bin_path.exists() {
        return Err("yt-dlp.exe not found in bin directory".to_string());
    }

    // Use --dump-json to get video metadata
    let output = Command::new(bin_path)
        .args(&["--dump-json", "--no-playlist", url.as_str()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("Failed to execute yt-dlp: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to fetch video info: {}", stderr));
    }

    // Parse JSON response
    let json_str = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&json_str).map_err(|e| format!("Failed to parse JSON: {}", e))?;

    let mut quality_options = Vec::new();

    // Always include Best and Lowest as default options
    quality_options.push(QualityOption {
        id: "best".to_string(),
        label: "Best Available".to_string(),
        format_type: "default".to_string(),
    });

    quality_options.push(QualityOption {
        id: "worst".to_string(),
        label: "Lowest Available".to_string(),
        format_type: "default".to_string(),
    });

    // Extract formats array
    if let Some(formats) = parsed["formats"].as_array() {
        let mut seen_resolutions = std::collections::HashSet::new();

        for format in formats.iter().rev() {
            // Reverse to get best qualities first
            let vcodec = format["vcodec"].as_str().unwrap_or("none");
            let acodec = format["acodec"].as_str().unwrap_or("none");
            let height = format["height"].as_i64();
            let format_id = format["format_id"].as_str().unwrap_or("");
            let ext = format["ext"].as_str().unwrap_or("mp4");

            // Skip if no video or audio
            if vcodec == "none" && acodec == "none" {
                continue;
            }

            // Create quality label
            let has_video = vcodec != "none";
            let has_audio = acodec != "none";

            if has_video && has_audio {
                if let Some(h) = height {
                    let resolution = format!("{}p", h);
                    if !seen_resolutions.contains(&resolution) {
                        seen_resolutions.insert(resolution.clone());
                        quality_options.push(QualityOption {
                            id: format_id.to_string(),
                            label: format!("{} ({})", resolution, ext),
                            format_type: "video+audio".to_string(),
                        });
                    }
                }
            } else if has_video {
                if let Some(h) = height {
                    let resolution = format!("{}p", h);
                    let key = format!("{}-video", resolution);
                    if !seen_resolutions.contains(&key) {
                        seen_resolutions.insert(key);
                        quality_options.push(QualityOption {
                            id: format_id.to_string(),
                            label: format!("{} (video only)", resolution),
                            format_type: "video".to_string(),
                        });
                    }
                }
            } else if has_audio {
                let key = format!("audio-{}", format_id);
                if !seen_resolutions.contains(&key) {
                    seen_resolutions.insert(key);
                    quality_options.push(QualityOption {
                        id: format_id.to_string(),
                        label: format!("Audio only ({})", ext),
                        format_type: "audio".to_string(),
                    });
                    break; // Only add one audio option
                }
            }
        }
    }

    Ok(quality_options)
}

#[tauri::command]
async fn download_media(url: String, format: String, quality: String) -> Result<String, String> {
    println!(
        "Downloading: {} (Format: {}, Quality: {})",
        url, format, quality
    );

    let bin_path = Path::new("bin/yt-dlp.exe");
    if !bin_path.exists() {
        return Err("yt-dlp.exe not found in bin directory".to_string());
    }

    let mut args = Vec::new();
    args.push(url.as_str());

    // Output template to Downloads folder or current dir
    args.push("-o");
    args.push("%(title)s.%(ext)s");

    // Handle format and quality selection
    // If quality is a specific format ID (not "best" or "worst"), use it directly
    if quality != "best" && quality != "worst" {
        args.push("-f");
        args.push(quality.as_str());
    } else {
        // Use default best/worst logic
        match (format.as_str(), quality.as_str()) {
            ("video_audio", "best") | ("video+audio", "best") => {
                args.push("-f");
                args.push("bv+ba/b");
            }
            ("video_audio", "worst") | ("video+audio", "worst") => {
                args.push("-f");
                args.push("wv+wa/w");
            }
            ("video_only", "best") | ("video", "best") => {
                args.push("-f");
                args.push("bv");
            }
            ("video_only", "worst") | ("video", "worst") => {
                args.push("-f");
                args.push("wv");
            }
            ("audio_only", "best") | ("audio", "best") => {
                args.push("-x");
                args.push("--audio-quality");
                args.push("0");
            }
            ("audio_only", "worst") | ("audio", "worst") => {
                args.push("-x");
                args.push("--audio-quality");
                args.push("10");
            }
            _ => {
                // Default to best
                args.push("-f");
                args.push("bv+ba/b");
            }
        }
    }

    // Create a new command
    let output = Command::new(bin_path)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok("Download successful".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Download failed: {}", stderr))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            download_media,
            get_video_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
