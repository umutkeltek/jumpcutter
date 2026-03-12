mod ffmpeg;
mod types;

use std::path::PathBuf;
use tauri::ipc::Channel;
use tauri::AppHandle;
use types::{AnalyzeOptions, AnalysisResult, ProcessEvent, ProcessOptions};

// ── Phase 1: Analyze ─────────────────────────────────

#[tauri::command]
async fn analyze_video(
    app: AppHandle,
    options: AnalyzeOptions,
) -> Result<AnalysisResult, String> {
    let ffmpeg = ffmpeg::resolve_binary(&app, "ffmpeg");
    let ffprobe = ffmpeg::resolve_binary(&app, "ffprobe");

    ffmpeg::analyze(
        &ffmpeg,
        &ffprobe,
        &options.input_path,
        options.noise_db,
        options.min_silence_duration,
        options.failure_tolerance,
        options.edge_padding,
        options.min_loud_duration,
    )
    .await
}

// ── Phase 2: Process ─────────────────────────────────

#[tauri::command]
async fn process_video(
    app: AppHandle,
    options: ProcessOptions,
    on_event: Channel<ProcessEvent>,
) -> Result<(), String> {
    let ffmpeg_bin = ffmpeg::resolve_binary(&app, "ffmpeg");
    let silence_intervals = &options.silence_intervals;
    let total_duration = options.duration;

    if silence_intervals.is_empty() {
        on_event
            .send(ProcessEvent::Log {
                message: "No silence intervals — nothing to process.".to_string(),
            })
            .ok();
        return Ok(());
    }

    on_event
        .send(ProcessEvent::Log {
            message: format!(
                "Processing {} silent intervals from {:.1}s video...",
                silence_intervals.len(),
                total_duration
            ),
        })
        .ok();

    match options.mode.as_str() {
        "remove" => {
            let keep = ffmpeg::invert_intervals(silence_intervals, total_duration);
            ffmpeg::cut_video(
                &ffmpeg_bin,
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
                &ffmpeg_bin,
                &options.input_path,
                &options.output_path,
                silence_intervals,
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
                &ffmpeg_bin,
                &options.input_path,
                &options.output_path,
                silence_intervals,
                &options.codec,
                &options.bitrate,
                &on_event,
            )
            .await?;
        }
        "both" => {
            let keep = ffmpeg::invert_intervals(silence_intervals, total_duration);
            let path = PathBuf::from(&options.output_path);
            let stem = path.file_stem().unwrap().to_string_lossy();
            let ext = path
                .extension()
                .map(|e| e.to_string_lossy().to_string())
                .unwrap_or_else(|| "mp4".to_string());
            let parent = path.parent().unwrap().to_string_lossy();

            on_event.send(ProcessEvent::Log { message: "Creating voiced-only file...".into() }).ok();
            ffmpeg::cut_video(&ffmpeg_bin, &options.input_path, &format!("{}/{}_voiced.{}", parent, stem, ext), &keep, &options.codec, &options.bitrate, &on_event).await?;

            on_event.send(ProcessEvent::Log { message: "Creating silence-only file...".into() }).ok();
            ffmpeg::cut_video(&ffmpeg_bin, &options.input_path, &format!("{}/{}_silent.{}", parent, stem, ext), silence_intervals, &options.codec, &options.bitrate, &on_event).await?;
        }
        _ => return Err(format!("Unknown mode: {}", options.mode)),
    }

    on_event.send(ProcessEvent::Progress { percent: 100.0 }).ok();
    on_event.send(ProcessEvent::Complete { output_path: options.output_path.clone() }).ok();
    on_event.send(ProcessEvent::Log { message: "Processing complete!".into() }).ok();

    Ok(())
}

// ── Preview audio segment ───────────────────────────

#[tauri::command]
async fn preview_audio(
    app: AppHandle,
    input_path: String,
    start_time: f64,
    duration: f64,
) -> Result<String, String> {
    let ffmpeg = ffmpeg::resolve_binary(&app, "ffmpeg");
    ffmpeg::extract_audio_segment(&ffmpeg, &input_path, start_time, duration).await
}

// ── App setup ────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![analyze_video, process_video, preview_audio])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
