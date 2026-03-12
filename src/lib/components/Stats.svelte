<script lang="ts">
  let {
    duration = 0,
    silenceDuration = 0,
    estimatedOutput = 0,
    cutCount = 0,
  }: {
    duration: number;
    silenceDuration: number;
    estimatedOutput: number;
    cutCount: number;
  } = $props();

  let percentRemoved = $derived(
    duration > 0 ? ((silenceDuration / duration) * 100).toFixed(1) : "0.0"
  );

  function fmt(seconds: number): string {
    if (seconds < 60) return `${seconds.toFixed(1)}s`;
    const m = Math.floor(seconds / 60);
    const s = (seconds % 60).toFixed(0);
    return `${m}m ${s}s`;
  }
</script>

<div class="stats-bar">
  <div class="stat">
    <span class="stat-value">{cutCount}</span>
    <span class="stat-label">Cuts</span>
  </div>
  <div class="stat-divider"></div>
  <div class="stat">
    <span class="stat-value">{fmt(silenceDuration)}</span>
    <span class="stat-label">Silence</span>
  </div>
  <div class="stat-divider"></div>
  <div class="stat">
    <span class="stat-value">{fmt(duration)}</span>
    <span class="stat-label">Original</span>
  </div>
  <div class="stat-divider"></div>
  <div class="stat">
    <span class="stat-value">{fmt(estimatedOutput)}</span>
    <span class="stat-label">Output</span>
  </div>
  <div class="stat-divider"></div>
  <div class="stat">
    <span class="stat-value accent">{percentRemoved}%</span>
    <span class="stat-label">Removed</span>
  </div>
</div>

<style>
  .stats-bar {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0;
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    padding: 10px 16px;
  }
  .stat {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    flex: 1;
    min-width: 0;
  }
  .stat-value {
    font-size: 15px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    color: var(--text);
  }
  .stat-value.accent {
    color: var(--accent);
  }
  .stat-label {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.8px;
    color: var(--text-dim);
    font-weight: 500;
  }
  .stat-divider {
    width: 1px;
    height: 28px;
    background: var(--border);
    flex-shrink: 0;
  }
</style>
