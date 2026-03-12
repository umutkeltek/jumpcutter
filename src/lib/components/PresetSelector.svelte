<script lang="ts">
  import { PRESETS } from "../types";

  let {
    selected = $bindable("youtube"),
  }: {
    selected: string;
  } = $props();
</script>

<div class="preset-bar">
  {#each PRESETS as preset}
    <button
      class="preset-pill"
      class:active={selected === preset.name}
      onclick={() => (selected = preset.name)}
      title={preset.name === "custom" ? "Manual settings" : `Optimized for ${preset.label.toLowerCase()} content`}
    >
      {preset.label}
    </button>
  {/each}
</div>

<style>
  .preset-bar {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }
  .preset-pill {
    padding: 6px 14px;
    border-radius: 20px;
    font-size: 12px;
    font-weight: 500;
    background: var(--bg-card);
    color: var(--text-muted);
    border: 1px solid var(--border);
    cursor: pointer;
    transition: all 0.15s;
    white-space: nowrap;
  }
  .preset-pill:hover {
    background: var(--bg-hover);
    color: var(--text);
  }
  .preset-pill.active {
    background: var(--accent);
    color: white;
    border-color: var(--accent);
  }
</style>
