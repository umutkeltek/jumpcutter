mod ffmpeg;
mod types;

use std::path::PathBuf;
use tauri::ipc::Channel;
use tauri::AppHandle;
use types::{ProcessEvent, ProcessOptions};

// ── Tauri Commands ───────────────────────────────────

#[tauri::command]
async fn process_video(
    app: AppHandle,
    options: ProcessOptions,
    on_event: Channel<ProcessEvent>,
) -> Result<(), String> {
    let ffmpeg = ffmpeg::resolve_binary(&app, "ffmpeg");
    let ffprobe = ffmpeg::resolve_binary(&app, "ffprobe");

    // Get video duration
    let total_duration = ffmpeg::get_duration(&ffprobe, &options.input_path).await?;
    on_event
        .send(ProcessEvent::Log {
            message: format!("Video duration: {:.1}s", total_duration),
        })
        .ok();

    // Detect silence
    let raw_intervals = ffmpeg::detect_silence(
        &ffmpeg,
        &options.input_path,
        options.noise_db,
        options.min_silence_duration,
        &on_event,
    )
    .await?;

    // Post-process intervals (merge close gaps, add edge padding)
    let silence_intervals = ffmpeg::post_process_intervals(
        raw_intervals,
        options.failure_tolerance,
        options.edge_padding,
        total_duration,
    );

    if silence_intervals.is_empty() {
        on_event
            .send(ProcessEvent::Log {
                message: "No silence detected — nothing to cut.".to_string(),
            })
            .ok();
        on_event
            .send(ProcessEvent::Complete {
                output_path: options.input_path.clone(),
            })
            .ok();
        return Ok(());
    }

    // Merge short loud parts into silence if threshold is set
    let silence_intervals = if options.min_loud_duration > 0.0 {
        ffmpeg::merge_short_loud_into_silence(
            silence_intervals,
            options.min_loud_duration,
            total_duration,
        )
    } else {
        silence_intervals
    };

    // Execute based on mode
    match options.mode.as_str() {
        "remove" => {
            let keep = ffmpeg::invert_intervals(&silence_intervals, total_duration);
            ffmpeg::cut_video(
                &ffmpeg,
                &options.input_path,
                &options.output_path,
                &keep,
                &options.codec,
                &options.bitrate,
                &on_event,
            )
            .await?;
        }
        "speed" => {
            ffmpeg::speed_silence(
                &ffmpeg,
                &options.input_path,
                &options.output_path,
                &silence_intervals,
                options.silence_speed,
                total_duration,
                &options.codec,
                &options.bitrate,
                &on_event,
            )
            .await?;
        }
        "voiced" => {
            ffmpeg::cut_video(
                &ffmpeg,
                &options.input_path,
                &options.output_path,
                &silence_intervals,
                &options.codec,
                &options.bitrate,
                &on_event,
            )
            .await?;
        }
        "both" => {
            let keep = ffmpeg::invert_intervals(&silence_intervals, total_duration);
            let path = PathBuf::from(&options.output_path);
            let stem = path.file_stem().unwrap().to_string_lossy();
            let ext = path
                .extension()
                .map(|e| e.to_string_lossy().to_string())
                .unwrap_or_else(|| "mp4".to_string());
            let parent = path.parent().unwrap().to_string_lossy();

            let voiced_path = format!("{}/{}_voiced.{}", parent, stem, ext);
            let silent_path = format!("{}/{}_silent.{}", parent, stem, ext);

            on_event
                .send(ProcessEvent::Log {
                    message: "Creating voiced-only file...".to_string(),
                })
                .ok();
            ffmpeg::cut_video(
                &ffmpeg,
                &options.input_path,
                &voiced_path,
                &keep,
                &options.codec,
                &options.bitrate,
                &on_event,
            )
            .await?;

            on_event
                .send(ProcessEvent::Log {
                    message: "Creating silence-only file...".to_string(),
                })
                .ok();
            ffmpeg::cut_video(
                &ffmpeg,
                &options.input_path,
                &silent_path,
                &silence_intervals,
                &options.codec,
                &options.bitrate,
                &on_event,
            )
            .await?;
        }
        _ => return Err(format!("Unknown mode: {}", options.mode)),
    }

    on_event.send(ProcessEvent::Progress { percent: 100.0 }).ok();
    on_event
        .send(ProcessEvent::Complete {
            output_path: options.output_path.clone(),
        })
        .ok();
    on_event
        .send(ProcessEvent::Log {
            message: "Processing complete!".to_string(),
        })
        .ok();

    Ok(())
}

#[tauri::command]
async fn get_video_info(app: AppHandle, path: String) -> Result<serde_json::Value, String> {
    let ffprobe = ffmpeg::resolve_binary(&app, "ffprobe");
    ffmpeg::get_info(&ffprobe, &path).await
}

// ── App setup ────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![process_video, get_video_info])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
