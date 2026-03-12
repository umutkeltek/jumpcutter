# JumpCutter

A fast, native desktop app that automatically removes silent parts from videos. Built with Tauri v2, Svelte 5, and FFmpeg.

![macOS](https://img.shields.io/badge/macOS-supported-green) ![Windows](https://img.shields.io/badge/Windows-supported-green) ![License](https://img.shields.io/badge/license-MIT-blue)

## Features

- **Silence detection** powered by FFmpeg's native `silencedetect` filter
- **Multiple modes**: remove silence, speed it up, keep only silence, or export both
- **Real-time progress** streamed from FFmpeg processing
- **Drag & drop** video files or use the native file picker
- **Dark / light theme** follows your system preference
- **Tiny footprint**: ~13MB app bundle (+ FFmpeg)

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (1.70+)
- [Node.js](https://nodejs.org/) (18+)
- [FFmpeg](https://ffmpeg.org/) installed on your system

### Development

```bash
# Install frontend dependencies
npm install

# Link system FFmpeg for development
bash scripts/setup-ffmpeg.sh

# Run in development mode
npx tauri dev
```

### Build

```bash
# Release build
npx tauri build
```

Output: `src-tauri/target/release/bundle/` contains `.app` (macOS) or `.exe` (Windows).

## Project Structure

```
src/                        # Svelte frontend
  lib/
    components/
      FileInput.svelte      # Drop zone + file picker
      DetectionSettings.svelte
      ProcessingSettings.svelte
      LogPanel.svelte        # Progress bar + log output
    types.ts                # TypeScript types
  App.svelte                # Main app shell
  app.css                   # Global styles + theme

src-tauri/                  # Rust backend
  src/
    main.rs                 # Entry point
    lib.rs                  # Tauri commands (orchestrator)
    ffmpeg.rs               # FFmpeg operations (silence detect, cut, speed)
    types.rs                # Shared types
  tauri.conf.json           # Tauri configuration
  capabilities/             # Permission definitions
```

## Parameters

| Parameter | Description | Default |
|-----------|-------------|---------|
| Silence Threshold | dB level below which audio is considered silent | -30 dB |
| Min Silence Duration | Minimum length of silence to detect | 0.5s |
| Failure Tolerance | Gap tolerance when merging nearby silent intervals | 0.1s |
| Edge Padding | Padding to leave around cut boundaries | 0.1s |
| Mode | Remove / Speed up / Keep only silence / Both | Remove |
| Speed Multiplier | Playback speed for silent parts (speed mode) | 2.0x |
| Min Loud Duration | Loud segments shorter than this are also cut | 0.0s |
| Codec | Output video codec (auto-detected if empty) | auto |
| Bitrate | Output video bitrate (auto if empty) | auto |

## Acknowledgements

Based on the original [jumpcutter](https://github.com/kivancyuksel/jumpcutter) concept by Kivanc Yuksel.

## License

MIT
