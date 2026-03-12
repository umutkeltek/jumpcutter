<script lang="ts">
  let {
    mode = $bindable("remove" as "remove" | "speed" | "voiced" | "both"),
    silenceSpeed = $bindable(2.0),
    minLoudDuration = $bindable(0.0),
    codec = $bindable(""),
    bitrate = $bindable(""),
  }: {
    mode: "remove" | "speed" | "voiced" | "both";
    silenceSpeed: number;
    minLoudDuration: number;
    codec: string;
    bitrate: string;
  } = $props();
</script>

<section class="settings-group">
  <h2>Processing</h2>

  <div class="mode-selector">
    <label class="mode-option" class:active={mode === "remove"}>
      <input type="radio" bind:group={mode} value="remove" />
      Remove silence
    </label>
    <label class="mode-option" class:active={mode === "speed"}>
      <input type="radio" bind:group={mode} value="speed" />
      Speed up silence
    </label>
    <label class="mode-option" class:active={mode === "voiced"}>
      <input type="radio" bind:group={mode} value="voiced" />
      Keep only silence
    </label>
    <label class="mode-option" class:active={mode === "both"}>
      <input type="radio" bind:group={mode} value="both" />
      Both (two files)
    </label>
  </div>

  {#if mode === "speed"}
    <div class="setting">
      <div class="setting-header">
        <label>Speed Multiplier</label>
        <span class="value">{silenceSpeed.toFixed(1)}x</span>
      </div>
      <input
        type="range"
        min="1.5"
        max="10.0"
        step="0.5"
        bind:value={silenceSpeed}
      />
    </div>
  {/if}

  <div class="setting">
    <div class="setting-header">
      <label>Min Loud Duration</label>
      <span class="value">{minLoudDuration.toFixed(1)}s</span>
    </div>
    <input
      type="range"
      min="0"
      max="5.0"
      step="0.1"
      bind:value={minLoudDuration}
    />
  </div>
</section>

<section class="settings-group">
  <h2>Output</h2>
  <div class="inline-settings">
    <div class="inline-field">
      <label>Codec</label>
      <input type="text" bind:value={codec} placeholder="auto" />
    </div>
    <div class="inline-field">
      <label>Bitrate</label>
      <input type="text" bind:value={bitrate} placeholder="auto" />
    </div>
  </div>
</section>

<style>
  .mode-selector {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 6px;
  }
  .mode-option {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 10px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    cursor: pointer;
    font-size: 12px;
    transition: all 0.15s;
  }
  .mode-option:hover {
    background: var(--bg-hover);
  }
  .mode-option.active {
    border-color: var(--accent);
    background: color-mix(in srgb, var(--accent) 8%, transparent);
  }
  .mode-option input {
    accent-color: var(--accent);
  }
  .inline-settings {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
  }
  .inline-field {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .inline-field label {
    font-size: 12px;
    color: var(--text-muted);
  }
  .inline-field input {
    width: 100%;
  }
</style>
