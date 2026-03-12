<script lang="ts">
  import { invoke, Channel } from "@tauri-apps/api/core";
  import type { ProcessOptions, ProcessEvent } from "./lib/types";
  import FileInput from "./lib/components/FileInput.svelte";
  import DetectionSettings from "./lib/components/DetectionSettings.svelte";
  import ProcessingSettings from "./lib/components/ProcessingSettings.svelte";
  import LogPanel from "./lib/components/LogPanel.svelte";
  import "./app.css";

  // ── File state ─────────────────────────────
  let inputPath = $state("");
  let outputPath = $state("");

  // ── Detection settings ─────────────────────
  let noiseDb = $state(-30);
  let minSilenceDuration = $state(0.5);
  let failureTolerance = $state(0.1);
  let edgePadding = $state(0.1);

  // ── Processing settings ────────────────────
  let mode = $state<"remove" | "speed" | "voiced" | "both">("remove");
  let silenceSpeed = $state(2.0);
  let minLoudDuration = $state(0.0);
  let codec = $state("");
  let bitrate = $state("");

  // ── Processing state ───────────────────────
  let processing = $state(false);
  let progress = $state(0);
  let logs = $state<string[]>([]);

  let canProcess = $derived(
    inputPath.length > 0 && outputPath.length > 0 && !processing
  );

  // ── Process video ──────────────────────────
  async function processVideo() {
    if (!canProcess) return;

    processing = true;
    progress = 0;
    logs = [];

    const onEvent = new Channel<ProcessEvent>();
    onEvent.onmessage = (msg: ProcessEvent) => {
      switch (msg.event) {
        case "log":
          logs = [...logs, msg.data.message];
          break;
        case "silenceDetected":
          break;
        case "progress":
          progress = Math.round(msg.data.percent);
          break;
        case "complete":
          logs = [...logs, `Output saved: ${msg.data.outputPath}`];
          break;
        case "error":
          logs = [...logs, `ERROR: ${msg.data.message}`];
          break;
      }
    };

    const options: ProcessOptions = {
      inputPath,
      outputPath,
      noiseDb,
      minSilenceDuration,
      failureTolerance,
      edgePadding,
      mode,
      silenceSpeed,
      minLoudDuration,
      codec: codec || null,
      bitrate: bitrate || null,
    };

    try {
      await invoke("process_video", { options, onEvent });
    } catch (err: any) {
      logs = [...logs, `ERROR: ${err}`];
    } finally {
      processing = false;
    }
  }
</script>

<main>
  <header>
    <h1>JumpCutter</h1>
    <span class="version">v2.0</span>
  </header>

  <div class="content">
    <FileInput bind:inputPath bind:outputPath />

    <DetectionSettings
      bind:noiseDb
      bind:minSilenceDuration
      bind:failureTolerance
      bind:edgePadding
    />

    <ProcessingSettings
      bind:mode
      bind:silenceSpeed
      bind:minLoudDuration
      bind:codec
      bind:bitrate
    />

    <button
      class="btn-process"
      class:processing
      disabled={!canProcess}
      onclick={processVideo}
    >
      {#if processing}
        Processing... {progress}%
      {:else}
        Process Video
      {/if}
    </button>

    <LogPanel {logs} {progress} {processing} />
  </div>
</main>

<style>
  main {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }

  header {
    display: flex;
    align-items: baseline;
    gap: 8px;
    padding: 16px 20px 8px;
    -webkit-app-region: drag;
  }

  header h1 {
    font-size: 20px;
    font-weight: 700;
    letter-spacing: -0.3px;
  }

  .version {
    font-size: 11px;
    color: var(--text-dim);
  }

  .content {
    flex: 1;
    overflow-y: auto;
    padding: 8px 20px 20px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  /* Shared setting styles (inherited by child components via :global) */
  :global(.settings-group) {
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    padding: 14px 16px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  :global(.settings-group h2) {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: var(--text-dim);
    font-weight: 600;
  }

  :global(.setting) {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  :global(.setting-header) {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  :global(.setting-header label) {
    font-size: 13px;
    color: var(--text);
  }

  :global(.value) {
    font-size: 12px;
    color: var(--accent);
    font-weight: 500;
    font-variant-numeric: tabular-nums;
    min-width: 48px;
    text-align: right;
  }

  .btn-process {
    background: var(--accent);
    color: white;
    font-size: 15px;
    font-weight: 600;
    padding: 14px;
    border-radius: var(--radius-lg);
    letter-spacing: 0.3px;
    transition: all 0.2s;
  }

  .btn-process:hover:not(:disabled) {
    background: var(--accent-hover);
    transform: translateY(-1px);
    box-shadow: 0 4px 12px color-mix(in srgb, var(--accent) 40%, transparent);
  }

  .btn-process:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .btn-process.processing {
    background: var(--bg-input);
    color: var(--text-muted);
  }
</style>
