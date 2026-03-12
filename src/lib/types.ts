// ── Presets ──────────────────────────────────────────

export interface Preset {
  name: string;
  label: string;
  noiseDb: number;
  minSilenceDuration: number;
  failureTolerance: number;
  edgePadding: number;
  minLoudDuration: number;
}

export const PRESETS: Preset[] = [
  { name: "podcast",   label: "Podcast",   noiseDb: -35, minSilenceDuration: 0.8, failureTolerance: 0.2,  edgePadding: 0.15, minLoudDuration: 0.0 },
  { name: "youtube",   label: "YouTube",   noiseDb: -30, minSilenceDuration: 0.4, failureTolerance: 0.1,  edgePadding: 0.08, minLoudDuration: 0.0 },
  { name: "lecture",   label: "Lecture",    noiseDb: -28, minSilenceDuration: 1.0, failureTolerance: 0.3,  edgePadding: 0.2,  minLoudDuration: 0.0 },
  { name: "interview", label: "Interview",  noiseDb: -32, minSilenceDuration: 0.6, failureTolerance: 0.15, edgePadding: 0.12, minLoudDuration: 0.0 },
  { name: "aggressive",label: "Aggressive", noiseDb: -25, minSilenceDuration: 0.3, failureTolerance: 0.05, edgePadding: 0.05, minLoudDuration: 0.0 },
  { name: "custom",    label: "Custom",     noiseDb: -30, minSilenceDuration: 0.5, failureTolerance: 0.1,  edgePadding: 0.1,  minLoudDuration: 0.0 },
];

// ── Analysis (Phase 1) ──────────────────────────────

export interface AnalyzeOptions {
  inputPath: string;
  noiseDb: number;
  minSilenceDuration: number;
  failureTolerance: number;
  edgePadding: number;
  minLoudDuration: number;
}

export interface AnalysisResult {
  duration: number;
  waveform: number[];
  silenceIntervals: [number, number][];
  silenceDuration: number;
  estimatedOutput: number;
  cutCount: number;
}

// ── Processing (Phase 2) ────────────────────────────

export interface ProcessOptions {
  inputPath: string;
  outputPath: string;
  silenceIntervals: [number, number][];
  duration: number;
  mode: "remove" | "speed" | "voiced" | "both";
  silenceSpeed: number;
  codec: string | null;
  bitrate: string | null;
}

export type ProcessEvent =
  | { event: "log"; data: { message: string } }
  | { event: "silenceDetected"; data: { count: number; intervals: [number, number][] } }
  | { event: "progress"; data: { percent: number } }
  | { event: "complete"; data: { outputPath: string } }
  | { event: "error"; data: { message: string } };
