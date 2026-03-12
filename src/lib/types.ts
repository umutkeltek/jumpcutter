export interface ProcessOptions {
  inputPath: string;
  outputPath: string;
  noiseDb: number;
  minSilenceDuration: number;
  failureTolerance: number;
  edgePadding: number;
  mode: "remove" | "speed" | "voiced" | "both";
  silenceSpeed: number;
  minLoudDuration: number;
  codec: string | null;
  bitrate: string | null;
}

export type ProcessEvent =
  | { event: "log"; data: { message: string } }
  | { event: "silenceDetected"; data: { count: number; intervals: [number, number][] } }
  | { event: "progress"; data: { percent: number } }
  | { event: "complete"; data: { outputPath: string } }
  | { event: "error"; data: { message: string } };
