<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let {
    waveform = [],
    silenceIntervals = [],
    duration = 0,
    inputPath = "",
  }: {
    waveform: number[];
    silenceIntervals: [number, number][];
    duration: number;
    inputPath: string;
  } = $props();

  let canvas: HTMLCanvasElement | undefined = $state();
  let container: HTMLDivElement | undefined = $state();
  let width = $state(800);
  const height = 120;

  // ── Playback state ────────────────────────
  let playing = $state(false);
  let playbackTime = $state(-1);
  let audio: HTMLAudioElement | null = $state(null);

  export function togglePlayPause() {
    if (playing && audio) {
      audio.pause();
      playing = false;
    } else if (playbackTime >= 0) {
      playAt(playbackTime);
    }
  }

  export function jumpToNextCut(direction: number) {
    if (silenceIntervals.length === 0 || duration <= 0) return;
    const boundaries = silenceIntervals.flatMap(([s, e]) => [s, e]).sort((a, b) => a - b);
    const current = playbackTime >= 0 ? playbackTime : 0;
    if (direction > 0) {
      const next = boundaries.find((b) => b > current + 0.1);
      if (next !== undefined) playAt(next);
    } else {
      const prev = boundaries.reverse().find((b) => b < current - 0.1);
      if (prev !== undefined) playAt(prev);
    }
  }

  async function playAt(time: number) {
    if (!inputPath || duration <= 0) return;

    // Stop existing playback and revoke ObjectURL to prevent memory leak
    if (audio) {
      const oldSrc = audio.src;
      audio.pause();
      if (oldSrc.startsWith("blob:")) URL.revokeObjectURL(oldSrc);
      audio = null;
    }

    playbackTime = time;
    playing = true;

    try {
      const base64Wav: string = await invoke("preview_audio", {
        inputPath,
        startTime: time,
        duration: 5.0,
      });

      const blob = base64ToBlob(base64Wav, "audio/wav");
      const url = URL.createObjectURL(blob);
      const a = new Audio(url);

      a.onended = () => {
        playing = false;
        URL.revokeObjectURL(url);
      };

      a.ontimeupdate = () => {
        playbackTime = time + a.currentTime;
        draw(); // Redraw to move cursor
      };

      audio = a;
      a.play();
    } catch (err) {
      playing = false;
      console.error("Preview failed:", err);
    }
  }

  function base64ToBlob(base64: string, mime: string): Blob {
    const binary = atob(base64);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) {
      bytes[i] = binary.charCodeAt(i);
    }
    return new Blob([bytes], { type: mime });
  }

  function handleCanvasClick(e: MouseEvent) {
    if (!canvas || duration <= 0) return;
    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const time = (x / rect.width) * duration;
    playAt(Math.max(0, Math.min(time, duration - 0.1)));
  }

  // ── Drawing ───────────────────────────────

  // Cached values recomputed when waveform/silenceIntervals change (not per draw)
  let cachedMaxVal = $derived(waveform.length > 0 ? waveform.reduce((a, b) => Math.max(a, b), 0.001) : 0.001);

  // Pre-build silence bitmap: O(n) build, O(1) per-bar lookup
  let silenceBitmap = $derived.by(() => {
    const barCount = waveform.length;
    if (barCount === 0 || duration <= 0) return new Uint8Array(0);
    const bitmap = new Uint8Array(barCount);
    for (let i = 0; i < barCount; i++) {
      const time = (i / barCount) * duration;
      for (const [s, e] of silenceIntervals) {
        if (time >= s && time <= e) {
          bitmap[i] = 1;
          break;
        }
      }
    }
    return bitmap;
  });

  function formatTime(seconds: number): string {
    const m = Math.floor(seconds / 60);
    const s = Math.floor(seconds % 60);
    return `${m}:${s.toString().padStart(2, "0")}`;
  }

  function draw() {
    if (!canvas || waveform.length === 0) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const dpr = window.devicePixelRatio || 1;
    canvas.width = width * dpr;
    canvas.height = height * dpr;
    ctx.scale(dpr, dpr);

    ctx.clearRect(0, 0, width, height);

    const barCount = waveform.length;
    const barWidth = width / barCount;
    const maxVal = cachedMaxVal;
    const waveHeight = height - 20;

    // Cache CSS colors outside loop
    const style = getComputedStyle(canvas);
    const silentColor = style.getPropertyValue("--waveform-silent").trim() || "rgba(233, 69, 96, 0.3)";
    const loudColor = style.getPropertyValue("--waveform-loud").trim() || "rgba(46, 213, 115, 0.8)";
    const bitmap = silenceBitmap;

    // Draw bars
    for (let i = 0; i < barCount; i++) {
      const normalized = waveform[i] / maxVal;
      const barH = Math.max(1, normalized * waveHeight);
      const x = i * barWidth;
      const y = waveHeight - barH;

      ctx.fillStyle = bitmap[i] ? silentColor : loudColor;
      ctx.fillRect(x, y, Math.max(barWidth - 0.5, 0.5), barH);
    }

    // Draw playback cursor
    if (playbackTime >= 0 && duration > 0) {
      const cursorX = (playbackTime / duration) * width;
      ctx.strokeStyle = "#ffffff";
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.moveTo(cursorX, 0);
      ctx.lineTo(cursorX, waveHeight);
      ctx.stroke();

      // Cursor time label
      ctx.fillStyle = "#ffffff";
      ctx.font = "bold 10px -apple-system, sans-serif";
      ctx.textAlign = "center";
      ctx.fillText(formatTime(playbackTime), cursorX, 10);
    }

    // Draw time axis
    ctx.fillStyle = style.getPropertyValue("--text-dim").trim() || "#5a6478";
    ctx.font = "10px -apple-system, sans-serif";
    ctx.textAlign = "center";

    const labelCount = Math.min(Math.floor(width / 80), 10);
    for (let i = 0; i <= labelCount; i++) {
      const t = (i / labelCount) * duration;
      const x = (i / labelCount) * width;
      ctx.fillText(formatTime(t), x, height - 4);
    }
  }

  $effect(() => {
    if (container) {
      const ro = new ResizeObserver((entries) => {
        width = entries[0].contentRect.width;
      });
      ro.observe(container);
      return () => ro.disconnect();
    }
  });

  $effect(() => {
    void waveform;
    void silenceIntervals;
    void duration;
    void width;
    draw();
  });
</script>

<div class="waveform-container" bind:this={container}>
  <div class="waveform-header">
    <span class="waveform-label">Waveform</span>
    {#if playing}
      <span class="playback-indicator">Playing {formatTime(playbackTime)}</span>
    {:else if playbackTime >= 0}
      <span class="playback-hint">Click waveform to preview</span>
    {:else}
      <span class="playback-hint">Click to preview audio</span>
    {/if}
  </div>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <canvas
    bind:this={canvas}
    style="width: {width}px; height: {height}px; cursor: pointer;"
    onclick={handleCanvasClick}
  ></canvas>
</div>

<style>
  .waveform-container {
    width: 100%;
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    padding: 12px;
    overflow: hidden;
  }
  .waveform-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
  }
  .waveform-label {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: var(--text-dim);
    font-weight: 600;
  }
  .playback-indicator {
    font-size: 11px;
    color: var(--accent);
    font-weight: 500;
    font-variant-numeric: tabular-nums;
  }
  .playback-hint {
    font-size: 11px;
    color: var(--text-dim);
  }
  canvas {
    display: block;
    width: 100%;
  }
</style>
