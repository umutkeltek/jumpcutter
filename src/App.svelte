<script lang="ts">
  import { invoke, Channel } from "@tauri-apps/api/core";
  import type {
    AnalyzeOptions,
    AnalysisResult,
    ProcessOptions,
    ProcessEvent,
  } from "./lib/types";
  import { PRESETS } from "./lib/types";
  import FileInput from "./lib/components/FileInput.svelte";
  import PresetSelector from "./lib/components/PresetSelector.svelte";
  import DetectionSettings from "./lib/components/DetectionSettings.svelte";
  import ProcessingSettings from "./lib/components/ProcessingSettings.svelte";
  import LogPanel from "./lib/components/LogPanel.svelte";
  import Waveform from "./lib/components/Waveform.svelte";
  import Stats from "./lib/components/Stats.svelte";
  import "./app.css";

  // ── File state ─────────────────────────────
  let inputPath = $state("");
  let outputPath = $state("");

  // ── Preset state ───────────────────────────
  let selectedPreset = $state("youtube");

  // ── Detection settings ─────────────────────
  let noiseDb = $state(-30);
  let minSilenceDuration = $state(0.5);
  let failureTolerance = $state(0.1);
  let edgePadding = $state(0.1);
  let minLoudDuration = $state(0.0);

  // ── Processing settings ────────────────────
  let mode = $state<"remove" | "speed" | "voiced" | "both">("remove");
  let silenceSpeed = $state(2.0);
  let codec = $state("");
  let bitrate = $state("");

  // ── Analysis state (Phase 1) ───────────────
  let analyzing = $state(false);
  let analysis = $state<AnalysisResult | null>(null);

  // ── Processing state (Phase 2) ─────────────
  let processing = $state(false);
  let progress = $state(0);
  let logs = $state<string[]>([]);

  // ── Waveform ref ───────────────────────────
  let waveformRef: Waveform | undefined = $state();

  let canAnalyze = $derived(inputPath.length > 0 && !analyzing && !processing);
  let canProcess = $derived(
    analysis !== null && outputPath.length > 0 && !processing && !analyzing
  );

  // ── Preset handling ───────────────────────
  let ignoreSettingChange = false;

  function applyPreset(presetName: string) {
    const preset = PRESETS.find((p) => p.name === presetName);
    if (!preset || presetName === "custom") return;

    ignoreSettingChange = true;
    noiseDb = preset.noiseDb;
    minSilenceDuration = preset.minSilenceDuration;
    failureTolerance = preset.failureTolerance;
    edgePadding = preset.edgePadding;
    minLoudDuration = preset.minLoudDuration;
    ignoreSettingChange = false;

    // Re-analyze with new settings
    if (inputPath) analyzeVideo();
  }

  // Watch preset changes
  $effect(() => {
    void selectedPreset;
    if (selectedPreset !== "custom") {
      applyPreset(selectedPreset);
    }
  });

  // Watch setting changes — switch to "Custom" if user manually adjusts
  $effect(() => {
    void noiseDb;
    void minSilenceDuration;
    void failureTolerance;
    void edgePadding;
    void minLoudDuration;
    if (!ignoreSettingChange && selectedPreset !== "custom") {
      const preset = PRESETS.find((p) => p.name === selectedPreset);
      if (
        preset &&
        (noiseDb !== preset.noiseDb ||
          minSilenceDuration !== preset.minSilenceDuration ||
          failureTolerance !== preset.failureTolerance ||
          edgePadding !== preset.edgePadding ||
          minLoudDuration !== preset.minLoudDuration)
      ) {
        selectedPreset = "custom";
      }
    }
  });

  // ── Phase 1: Analyze ──────────────────────
  async function analyzeVideo() {
    if (!canAnalyze) return;

    analyzing = true;
    analysis = null;

    try {
      const options: AnalyzeOptions = {
        inputPath,
        noiseDb,
        minSilenceDuration,
        failureTolerance,
        edgePadding,
        minLoudDuration,
      };
      analysis = await invoke<AnalysisResult>("analyze_video", { options });
    } catch (err: any) {
      logs = [...logs.slice(-499), `Analysis error: ${err}`];
    } finally {
      analyzing = false;
    }
  }

  // Auto-analyze when file is selected
  let prevInputPath = $state("");
  $effect(() => {
    if (inputPath && inputPath !== prevInputPath) {
      prevInputPath = inputPath;
      analyzeVideo();
    }
  });

  // ── Phase 2: Process ──────────────────────
  async function processVideo() {
    if (!canProcess || !analysis) return;

    processing = true;
    progress = 0;
    logs = [];

    const onEvent = new Channel<ProcessEvent>();
    onEvent.onmessage = (msg: ProcessEvent) => {
      switch (msg.event) {
        case "log":
          logs = [...logs.slice(-499), msg.data.message];
          break;
        case "progress":
          progress = Math.round(msg.data.percent);
          break;
        case "complete":
          logs = [...logs.slice(-499), `Output saved: ${msg.data.outputPath}`];
          break;
        case "error":
          logs = [...logs.slice(-499), `ERROR: ${msg.data.message}`];
          break;
      }
    };

    const options: ProcessOptions = {
      inputPath,
      outputPath,
      silenceIntervals: analysis.silenceIntervals,
      duration: analysis.duration,
      mode,
      silenceSpeed,
      codec: codec || null,
      bitrate: bitrate || null,
    };

    try {
      await invoke("process_video", { options, onEvent });
    } catch (err: any) {
      logs = [...logs.slice(-499), `ERROR: ${err}`];
    } finally {
      processing = false;
    }
  }

  // ── Keyboard shortcuts ────────────────────
  function handleKeydown(e: KeyboardEvent) {
    // Don't intercept when typing in inputs
    const tag = (e.target as HTMLElement)?.tagName;
    if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return;

    switch (e.key) {
      case " ":
        e.preventDefault();
        waveformRef?.togglePlayPause();
        break;
      case "ArrowLeft":
        e.preventDefault();
        waveformRef?.jumpToNextCut(-1);
        break;
      case "ArrowRight":
        e.preventDefault();
        waveformRef?.jumpToNextCut(1);
        break;
      case "Enter":
        e.preventDefault();
        processVideo();
        break;
      case "r":
      case "R":
        if (!e.metaKey && !e.ctrlKey) {
          e.preventDefault();
          analyzeVideo();
        }
        break;
      case "1":
      case "2":
      case "3":
      case "4":
      case "5": {
        const idx = parseInt(e.key) - 1;
        if (idx < PRESETS.length - 1) {
          // Skip "custom" which is last
          selectedPreset = PRESETS[idx].name;
        }
        break;
      }
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<main>
  <header>
    <h1>JumpCutter</h1>
    <span class="version">v2.0</span>
    <span class="shortcuts-hint">Space: play | Arrows: navigate cuts | R: re-analyze | 1-5: presets</span>
  </header>

  <div class="content">
    <!-- File input -->
    <FileInput bind:inputPath bind:outputPath />

    <!-- Preset selector (shown after file selected) -->
    {#if inputPath}
      <PresetSelector bind:selected={selectedPreset} />
    {/if}

    <!-- Analyzing spinner -->
    {#if analyzing}
      <div class="analyzing">
        <div class="spinner"></div>
        <span>Analyzing audio...</span>
      </div>
    {/if}

    <!-- Waveform + Stats (shown after analysis) -->
    {#if analysis}
      <Waveform
        bind:this={waveformRef}
        waveform={analysis.waveform}
        silenceIntervals={analysis.silenceIntervals}
        duration={analysis.duration}
        {inputPath}
      />

      <Stats
        duration={analysis.duration}
        silenceDuration={analysis.silenceDuration}
        estimatedOutput={analysis.estimatedOutput}
        cutCount={analysis.cutCount}
      />
    {/if}

    <!-- Settings row -->
    {#if inputPath}
      <div class="settings-row">
        <div class="settings-col">
          <DetectionSettings
            bind:noiseDb
            bind:minSilenceDuration
            bind:failureTolerance
            bind:edgePadding
          />
        </div>
        <div class="settings-col">
          <ProcessingSettings
            bind:mode
            bind:silenceSpeed
            bind:minLoudDuration={minLoudDuration}
            bind:codec
            bind:bitrate
          />
        </div>
      </div>

      <!-- Action buttons -->
      <div class="action-row">
        {#if analysis}
          <button
            class="btn-reanalyze"
            disabled={!canAnalyze}
            onclick={analyzeVideo}
          >
            Re-analyze
          </button>
        {/if}
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
      </div>

      <LogPanel {logs} {progress} {processing} />
    {/if}
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

  .shortcuts-hint {
    font-size: 10px;
    color: var(--text-dim);
    margin-left: auto;
    -webkit-app-region: no-drag;
  }

  .content {
    flex: 1;
    overflow-y: auto;
    padding: 8px 20px 20px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  /* Analyzing spinner */
  .analyzing {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 10px;
    padding: 24px;
    color: var(--text-muted);
    font-size: 13px;
  }

  .spinner {
    width: 18px;
    height: 18px;
    border: 2px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  /* Two-column settings layout */
  .settings-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }

  .settings-col {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  /* Shared setting styles */
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

  /* Action buttons */
  .action-row {
    display: flex;
    gap: 10px;
  }

  .btn-reanalyze {
    background: var(--bg-input);
    color: var(--text);
    font-size: 13px;
    font-weight: 500;
    padding: 12px 20px;
    border-radius: var(--radius-lg);
    border: 1px solid var(--border);
    transition: all 0.2s;
  }

  .btn-reanalyze:hover:not(:disabled) {
    background: var(--bg-hover);
    border-color: var(--accent);
  }

  .btn-reanalyze:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .btn-process {
    flex: 1;
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
