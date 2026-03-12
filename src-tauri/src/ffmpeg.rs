use regex::Regex;
use std::process::Stdio;
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::types::{AnalysisResult, ProcessEvent};

// ── Path resolution ──────────────────────────────────

pub fn resolve_binary(app: &AppHandle, name: &str) -> String {
    if let Ok(resource_dir) = app.path().resource_dir() {
        let binary_name = if cfg!(target_os = "windows") {
            format!("{}.exe", name)
        } else {
            name.to_string()
        };
        let bundled = resource_dir.join("binaries").join(&binary_name);
        if bundled.exists() {
            return bundled.to_string_lossy().to_string();
        }
    }
    name.to_string()
}

// ── Video duration ───────────────────────────────────

pub async fn get_duration(ffprobe_path: &str, input: &str) -> Result<f64, String> {
    let output = Command::new(ffprobe_path)
        .args([
            "-v", "error",
            "-show_entries", "format=duration",
            "-of", "default=noprint_wrappers=1:nokey=1",
            input,
        ])
        .output()
        .await
        .map_err(|e| format!("ffprobe failed: {}", e))?;

    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<f64>()
        .map_err(|_| "Could not parse video duration".to_string())
}

// ── Silence detection ────────────────────────────────

pub async fn detect_silence(
    ffmpeg_path: &str,
    input: &str,
    noise_db: f64,
    min_duration: f64,
    on_event: &Channel<ProcessEvent>,
) -> Result<Vec<[f64; 2]>, String> {
    on_event
        .send(ProcessEvent::Log {
            message: "Detecting silence...".to_string(),
        })
        .ok();

    let mut child = Command::new(ffmpeg_path)
        .args([
            "-i",
            input,
            "-af",
            &format!("silencedetect=noise={}dB:d={}", noise_db, min_duration),
            "-f",
            "null",
            "-",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start ffmpeg: {}", e))?;

    let stderr = child.stderr.take().unwrap();
    let reader = BufReader::new(stderr);
    let mut lines = reader.lines();

    let start_re = Regex::new(r"silence_start:\s*(\d+\.?\d*)").unwrap();
    let end_re = Regex::new(r"silence_end:\s*(\d+\.?\d*)").unwrap();

    let mut intervals: Vec<[f64; 2]> = Vec::new();
    let mut current_start: Option<f64> = None;

    while let Ok(Some(line)) = lines.next_line().await {
        if let Some(caps) = start_re.captures(&line) {
            current_start = caps[1].parse().ok();
        }
        if let Some(caps) = end_re.captures(&line) {
            if let (Some(start), Some(end)) = (current_start.take(), caps[1].parse::<f64>().ok()) {
                intervals.push([start, end]);
            }
        }
    }

    let status = child
        .wait()
        .await
        .map_err(|e| format!("ffmpeg failed: {}", e))?;
    if !status.success() {
        return Err("FFmpeg silence detection failed".to_string());
    }

    on_event
        .send(ProcessEvent::SilenceDetected {
            count: intervals.len(),
            intervals: intervals.clone(),
        })
        .ok();

    on_event
        .send(ProcessEvent::Log {
            message: format!("Found {} silent intervals", intervals.len()),
        })
        .ok();

    Ok(intervals)
}

// ── Interval math ────────────────────────────────────

pub fn post_process_intervals(
    intervals: Vec<[f64; 2]>,
    failure_tolerance: f64,
    edge_padding: f64,
    total_duration: f64,
) -> Vec<[f64; 2]> {
    if intervals.is_empty() {
        return intervals;
    }

    // Merge intervals closer than failure_tolerance
    let mut merged: Vec<[f64; 2]> = vec![intervals[0]];
    for interval in &intervals[1..] {
        let last = merged.last_mut().unwrap();
        if interval[0] - last[1] < failure_tolerance {
            last[1] = interval[1];
        } else {
            merged.push(*interval);
        }
    }

    // Apply edge padding
    merged
        .into_iter()
        .filter_map(|[start, end]| {
            let s = (start + edge_padding).max(0.0);
            let e = (end - edge_padding).min(total_duration);
            if s < e { Some([s, e]) } else { None }
        })
        .collect()
}

pub fn invert_intervals(silence: &[[f64; 2]], total_duration: f64) -> Vec<[f64; 2]> {
    let mut loud = Vec::new();
    let mut prev_end = 0.0;

    for &[start, end] in silence {
        if start > prev_end {
            loud.push([prev_end, start]);
        }
        prev_end = end;
    }

    if prev_end < total_duration {
        loud.push([prev_end, total_duration]);
    }

    loud
}

pub fn merge_short_loud_into_silence(
    silence: Vec<[f64; 2]>,
    min_loud_duration: f64,
    total_duration: f64,
) -> Vec<[f64; 2]> {
    let loud = invert_intervals(&silence, total_duration);
    let short_loud: Vec<[f64; 2]> = loud
        .into_iter()
        .filter(|[s, e]| (e - s) < min_loud_duration)
        .collect();

    let mut expanded = silence;
    expanded.extend(short_loud);
    expanded.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap());

    // Re-merge overlapping
    if expanded.is_empty() {
        return expanded;
    }
    let mut merged: Vec<[f64; 2]> = vec![expanded[0]];
    for seg in &expanded[1..] {
        let last = merged.last_mut().unwrap();
        if seg[0] <= last[1] {
            last[1] = last[1].max(seg[1]);
        } else {
            merged.push(*seg);
        }
    }
    merged
}

// ── FFmpeg select expression builder ─────────────────

fn build_select_expr(intervals: &[[f64; 2]]) -> String {
    if intervals.is_empty() {
        return "1".to_string();
    }
    intervals
        .iter()
        .map(|[s, e]| format!("between(t,{:.4},{:.4})", s, e))
        .collect::<Vec<_>>()
        .join("+")
}

// ── Video cutting (select/aselect approach) ──────────

pub async fn cut_video(
    ffmpeg_path: &str,
    input: &str,
    output: &str,
    keep_intervals: &[[f64; 2]],
    codec: &Option<String>,
    bitrate: &Option<String>,
    on_event: &Channel<ProcessEvent>,
) -> Result<(), String> {
    on_event
        .send(ProcessEvent::Log {
            message: "Cutting video segments...".to_string(),
        })
        .ok();

    let select_expr = build_select_expr(keep_intervals);
    let vf = format!("select='{}',setpts=N/FRAME_RATE/TB", select_expr);
    let af = format!("aselect='{}',asetpts=N/SR/TB", select_expr);

    let mut args: Vec<String> = vec![
        "-y".into(),
        "-i".into(),
        input.into(),
        "-vf".into(),
        vf,
        "-af".into(),
        af,
    ];

    if let Some(c) = codec {
        args.extend(["-c:v".into(), c.clone()]);
    }
    if let Some(b) = bitrate {
        args.extend(["-b:v".into(), b.clone()]);
    }

    args.extend(["-progress".into(), "pipe:1".into()]);
    args.push(output.into());

    let mut child = Command::new(ffmpeg_path)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start ffmpeg: {}", e))?;

    let stdout = child.stdout.take().unwrap();
    let reader = BufReader::new(stdout);
    let mut lines = reader.lines();
    let time_re = Regex::new(r"out_time_ms=(\d+)").unwrap();
    let kept_duration: f64 = keep_intervals.iter().map(|[s, e]| e - s).sum();

    while let Ok(Some(line)) = lines.next_line().await {
        if let Some(caps) = time_re.captures(&line) {
            if let Ok(time_ms) = caps[1].parse::<f64>() {
                let time_s = time_ms / 1_000_000.0;
                let percent = if kept_duration > 0.0 {
                    (time_s / kept_duration * 100.0).min(99.0)
                } else {
                    0.0
                };
                on_event.send(ProcessEvent::Progress { percent }).ok();
            }
        }
    }

    let status = child.wait().await.map_err(|e| format!("ffmpeg failed: {}", e))?;
    if !status.success() {
        return Err("FFmpeg encoding failed".to_string());
    }

    Ok(())
}

// ── Speed up silence (concat approach) ───────────────

pub async fn speed_silence(
    ffmpeg_path: &str,
    input: &str,
    output: &str,
    silence_intervals: &[[f64; 2]],
    speed: f64,
    total_duration: f64,
    codec: &Option<String>,
    bitrate: &Option<String>,
    on_event: &Channel<ProcessEvent>,
) -> Result<(), String> {
    on_event
        .send(ProcessEvent::Log {
            message: format!(
                "Speeding up {} silent segments at {:.1}x...",
                silence_intervals.len(),
                speed
            ),
        })
        .ok();

    let loud = invert_intervals(silence_intervals, total_duration);

    // Build time-ordered segments: (interval, is_silent)
    let mut all_segments: Vec<([f64; 2], bool)> = Vec::new();
    for &interval in &loud {
        all_segments.push((interval, false));
    }
    for &interval in silence_intervals {
        all_segments.push((interval, true));
    }
    all_segments.sort_by(|a, b| a.0[0].partial_cmp(&b.0[0]).unwrap());

    let mut filter_parts = Vec::new();
    let mut concat_inputs = Vec::new();
    let mut idx = 0;

    for (segment, is_silent) in &all_segments {
        let [start, end] = segment;
        if end - start < 0.01 {
            continue;
        }

        let v_label = format!("v{}", idx);
        let a_label = format!("a{}", idx);

        if *is_silent {
            let atempo = speed.clamp(0.5, 100.0);
            let setpts = 1.0 / speed;
            filter_parts.push(format!(
                "[0:v]trim=start={:.4}:end={:.4},setpts=(PTS-STARTPTS)*{:.4}[{}]",
                start, end, setpts, v_label
            ));
            filter_parts.push(format!(
                "[0:a]atrim=start={:.4}:end={:.4},asetpts=PTS-STARTPTS,atempo={:.4}[{}]",
                start, end, atempo, a_label
            ));
        } else {
            filter_parts.push(format!(
                "[0:v]trim=start={:.4}:end={:.4},setpts=PTS-STARTPTS[{}]",
                start, end, v_label
            ));
            filter_parts.push(format!(
                "[0:a]atrim=start={:.4}:end={:.4},asetpts=PTS-STARTPTS[{}]",
                start, end, a_label
            ));
        }

        concat_inputs.push(format!("[{}][{}]", v_label, a_label));
        idx += 1;
    }

    if idx == 0 {
        Command::new(ffmpeg_path)
            .args(["-y", "-i", input, "-c", "copy", output])
            .status()
            .await
            .map_err(|e| format!("FFmpeg copy failed: {}", e))?;
        return Ok(());
    }

    let concat = format!(
        "{}concat=n={}:v=1:a=1[outv][outa]",
        concat_inputs.join(""),
        idx
    );
    filter_parts.push(concat);
    let filter_complex = filter_parts.join(";");

    let mut args: Vec<String> = vec![
        "-y".into(),
        "-i".into(),
        input.into(),
        "-filter_complex".into(),
        filter_complex,
        "-map".into(),
        "[outv]".into(),
        "-map".into(),
        "[outa]".into(),
    ];

    if let Some(c) = codec {
        args.extend(["-c:v".into(), c.clone()]);
    }
    if let Some(b) = bitrate {
        args.extend(["-b:v".into(), b.clone()]);
    }

    args.extend(["-progress".into(), "pipe:1".into()]);
    args.push(output.into());

    let mut child = Command::new(ffmpeg_path)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start ffmpeg: {}", e))?;

    let stdout = child.stdout.take().unwrap();
    let reader = BufReader::new(stdout);
    let mut lines = reader.lines();
    let time_re = Regex::new(r"out_time_ms=(\d+)").unwrap();

    while let Ok(Some(line)) = lines.next_line().await {
        if let Some(caps) = time_re.captures(&line) {
            if let Ok(time_ms) = caps[1].parse::<f64>() {
                let time_s = time_ms / 1_000_000.0;
                let percent = (time_s / total_duration * 100.0).min(99.0);
                on_event.send(ProcessEvent::Progress { percent }).ok();
            }
        }
    }

    let status = child.wait().await.map_err(|e| format!("ffmpeg failed: {}", e))?;
    if !status.success() {
        return Err("FFmpeg speed processing failed".to_string());
    }

    Ok(())
}

// ── Waveform extraction ──────────────────────────────

pub async fn extract_waveform(
    ffmpeg_path: &str,
    input: &str,
    target_peaks: usize,
) -> Result<Vec<f32>, String> {
    // Extract raw mono audio at 8kHz as f32le
    let output = Command::new(ffmpeg_path)
        .args([
            "-i", input,
            "-ac", "1",
            "-ar", "8000",
            "-f", "f32le",
            "-acodec", "pcm_f32le",
            "pipe:1",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .await
        .map_err(|e| format!("Waveform extraction failed: {}", e))?;

    let raw = output.stdout;
    if raw.len() < 4 {
        return Ok(vec![0.0; target_peaks]);
    }

    // Convert bytes to f32 samples
    let sample_count = raw.len() / 4;
    let samples: Vec<f32> = (0..sample_count)
        .map(|i| {
            let bytes = [raw[i * 4], raw[i * 4 + 1], raw[i * 4 + 2], raw[i * 4 + 3]];
            f32::from_le_bytes(bytes)
        })
        .collect();

    // Downsample to target_peaks by taking max absolute value per window
    let window_size = (samples.len() / target_peaks).max(1);
    let peaks: Vec<f32> = samples
        .chunks(window_size)
        .map(|chunk| {
            chunk
                .iter()
                .map(|s| s.abs())
                .fold(0.0f32, f32::max)
        })
        .collect();

    Ok(peaks)
}

// ── Full analysis (waveform + silence detection) ─────

pub async fn analyze(
    ffmpeg_path: &str,
    ffprobe_path: &str,
    input: &str,
    noise_db: f64,
    min_silence_duration: f64,
    failure_tolerance: f64,
    edge_padding: f64,
    min_loud_duration: f64,
) -> Result<AnalysisResult, String> {
    // Get duration
    let duration = get_duration(ffprobe_path, input).await?;

    // Run waveform extraction and silence detection concurrently
    let waveform_fut = extract_waveform(ffmpeg_path, input, 2000);
    let silence_fut = detect_silence_raw(ffmpeg_path, input, noise_db, min_silence_duration);

    let (waveform_result, silence_result) = tokio::join!(waveform_fut, silence_fut);
    let waveform = waveform_result?;
    let raw_intervals = silence_result?;

    // Post-process
    let mut silence_intervals = post_process_intervals(
        raw_intervals,
        failure_tolerance,
        edge_padding,
        duration,
    );

    if min_loud_duration > 0.0 && !silence_intervals.is_empty() {
        silence_intervals = merge_short_loud_into_silence(
            silence_intervals,
            min_loud_duration,
            duration,
        );
    }

    let silence_duration: f64 = silence_intervals.iter().map(|[s, e]| e - s).sum();
    let cut_count = silence_intervals.len();
    let estimated_output = duration - silence_duration;

    Ok(AnalysisResult {
        duration,
        waveform,
        silence_intervals,
        silence_duration,
        estimated_output,
        cut_count,
    })
}

// ── Raw silence detection (no event channel) ─────────

async fn detect_silence_raw(
    ffmpeg_path: &str,
    input: &str,
    noise_db: f64,
    min_duration: f64,
) -> Result<Vec<[f64; 2]>, String> {
    let mut child = Command::new(ffmpeg_path)
        .args([
            "-i", input,
            "-af", &format!("silencedetect=noise={}dB:d={}", noise_db, min_duration),
            "-f", "null", "-",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start ffmpeg: {}", e))?;

    let stderr = child.stderr.take().unwrap();
    let reader = BufReader::new(stderr);
    let mut lines = reader.lines();

    let start_re = Regex::new(r"silence_start:\s*(\d+\.?\d*)").unwrap();
    let end_re = Regex::new(r"silence_end:\s*(\d+\.?\d*)").unwrap();

    let mut intervals: Vec<[f64; 2]> = Vec::new();
    let mut current_start: Option<f64> = None;

    while let Ok(Some(line)) = lines.next_line().await {
        if let Some(caps) = start_re.captures(&line) {
            current_start = caps[1].parse().ok();
        }
        if let Some(caps) = end_re.captures(&line) {
            if let (Some(start), Some(end)) = (current_start.take(), caps[1].parse::<f64>().ok()) {
                intervals.push([start, end]);
            }
        }
    }

    let status = child.wait().await.map_err(|e| format!("ffmpeg failed: {}", e))?;
    if !status.success() {
        return Err("FFmpeg silence detection failed".to_string());
    }

    Ok(intervals)
}

// ── Audio preview extraction ─────────────────────────

pub async fn extract_audio_segment(
    ffmpeg_path: &str,
    input: &str,
    start_time: f64,
    duration: f64,
) -> Result<String, String> {
    use base64::Engine;

    let output = Command::new(ffmpeg_path)
        .args([
            "-ss", &format!("{:.3}", start_time),
            "-i", input,
            "-t", &format!("{:.3}", duration),
            "-ac", "1",
            "-ar", "44100",
            "-f", "wav",
            "-acodec", "pcm_s16le",
            "pipe:1",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .await
        .map_err(|e| format!("Audio preview failed: {}", e))?;

    if output.stdout.is_empty() {
        return Err("No audio data extracted".to_string());
    }

    Ok(base64::engine::general_purpose::STANDARD.encode(&output.stdout))
}

// ── Video info (ffprobe JSON) ────────────────────────

pub async fn get_info(ffprobe_path: &str, path: &str) -> Result<serde_json::Value, String> {
    let output = Command::new(ffprobe_path)
        .args([
            "-v", "quiet",
            "-print_format", "json",
            "-show_format",
            "-show_streams",
            path,
        ])
        .output()
        .await
        .map_err(|e| format!("ffprobe failed: {}", e))?;

    serde_json::from_str(&String::from_utf8_lossy(&output.stdout))
        .map_err(|e| format!("Parse error: {}", e))
}
