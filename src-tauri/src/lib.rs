use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Stdio;
use tauri::{Emitter, Window};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

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
    // Use --dump-json to get video metadata
    let output = Command::new(bin_path)
        .args(&["--dump-json", "--no-playlist", url.as_str()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
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

#[derive(Clone, Serialize)]
struct LogMessage {
    message_type: String,
    message: String,
}

#[derive(Clone, Serialize)]
struct DownloadProgress {
    progress: f64,
    status: String,
}

#[tauri::command]
async fn download_media(
    window: Window,
    url: String,
    format: String,
    quality: String,
    download_path: Option<String>,
) -> Result<String, String> {
    println!(
        "Downloading: {} (Format: {}, Quality: {}, Path: {:?})",
        url, format, quality, download_path
    );

    // Emit initial log to frontend
    let _ = window.emit(
        "download-log",
        LogMessage {
            message_type: "stdout".to_string(),
            message: format!(
                "Starting download... URL: {}, Path: {:?}",
                url, download_path
            ),
        },
    );

    let bin_path = Path::new("bin/yt-dlp.exe");
    println!("Checking for yt-dlp.exe at: {:?}", bin_path);
    println!("Current dir: {:?}", std::env::current_dir());
    if !bin_path.exists() {
        let err_msg = format!(
            "yt-dlp.exe not found in bin directory. Current dir: {:?}, Checked path: {:?}",
            std::env::current_dir(),
            bin_path
        );
        println!("{}", err_msg);
        return Err(err_msg);
    }
    println!("yt-dlp.exe found, building args...");

    let mut args = Vec::new();
    args.push(url.clone());
    args.push("--newline".to_string()); // Ensure line-buffered output
    args.push("--progress".to_string()); // Force progress output

    // Output template to Downloads folder or current dir
    // Set download path if provided
    if let Some(path) = download_path {
        args.push("-P".to_string());
        args.push(path);
    }

    // Output template for filename only
    args.push("-o".to_string());
    args.push("%(title)s.%(ext)s".to_string());

    // Handle format and quality selection
    if quality != "best" && quality != "worst" {
        args.push("-f".to_string());
        args.push(quality.clone());
    } else {
        match (format.as_str(), quality.as_str()) {
            ("video_audio", "best") | ("video+audio", "best") => {
                args.push("-f".to_string());
                args.push("bv+ba/b".to_string());
            }
            ("video_audio", "worst") | ("video+audio", "worst") => {
                args.push("-f".to_string());
                args.push("wv+wa/w".to_string());
            }
            ("video_only", "best") | ("video", "best") => {
                args.push("-f".to_string());
                args.push("bv".to_string());
            }
            ("video_only", "worst") | ("video", "worst") => {
                args.push("-f".to_string());
                args.push("wv".to_string());
            }
            ("audio_only", "best") | ("audio", "best") => {
                args.push("-x".to_string());
                args.push("--audio-quality".to_string());
                args.push("0".to_string());
            }
            ("audio_only", "worst") | ("audio", "worst") => {
                args.push("-x".to_string());
                args.push("--audio-quality".to_string());
                args.push("10".to_string());
            }
            _ => {
                args.push("-f".to_string());
                args.push("bv+ba/b".to_string());
            }
        }
    }

    println!("Spawning yt-dlp with args: {:?}", args);

    // Create a new command
    let mut child = Command::new(bin_path)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped()) // Capture stderr
        .spawn()
        .map_err(|e| {
            let err_msg = format!("Failed to spawn yt-dlp: {}", e);
            println!("{}", err_msg);
            err_msg
        })?;

    println!("yt-dlp spawned successfully, reading output...");

    let stdout = child.stdout.take().ok_or("Failed to open stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to open stderr")?;

    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    // Regex to capture progress percentage
    let progress_regex = Regex::new(r"(\d+\.?\d*)%").map_err(|e| e.to_string())?;

    // Spawn a task to read stderr concurrently so it doesn't block
    let window_clone = window.clone();
    tokio::spawn(async move {
        while let Ok(Some(line)) = stderr_reader.next_line().await {
            println!("yt-dlp stderr: {}", line);
            let _ = window_clone.emit(
                "download-log",
                LogMessage {
                    message_type: "stderr".to_string(),
                    message: line,
                },
            );
        }
    });

    println!("Starting to read stdout...");
    while let Ok(Some(line)) = stdout_reader.next_line().await {
        println!("yt-dlp stdout: {}", line); // Log output for debugging

        let _ = window.emit(
            "download-log",
            LogMessage {
                message_type: "stdout".to_string(),
                message: line.clone(),
            },
        );

        if let Some(caps) = progress_regex.captures(&line) {
            if let Some(match_) = caps.get(1) {
                if let Ok(progress) = match_.as_str().parse::<f64>() {
                    let _ = window.emit(
                        "download-progress",
                        DownloadProgress {
                            progress,
                            status: "downloading".to_string(),
                        },
                    );
                }
            }
        }
    }
    println!("Finished reading stdout.");

    let status = child
        .wait()
        .await
        .map_err(|e| format!("Failed to wait on child: {}", e))?;
    println!("yt-dlp exit status: {}", status);

    if status.success() {
        Ok("Download successful".to_string())
    } else {
        Err(format!("Download failed with status: {}", status))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            download_media,
            get_video_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
