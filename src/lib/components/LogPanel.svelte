<script lang="ts">
  let { logs, progress, processing }: {
    logs: string[];
    progress: number;
    processing: boolean;
  } = $props();
</script>

{#if processing || progress > 0}
  <div class="progress-bar">
    <div class="progress-fill" style="width: {progress}%"></div>
  </div>
{/if}

{#if logs.length > 0}
  <section class="log-panel">
    {#each logs as line}
      <div class="log-line" class:error={line.startsWith("ERROR")}>{line}</div>
    {/each}
  </section>
{/if}

<style>
  .progress-bar {
    height: 6px;
    background: var(--bg-input);
    border-radius: 3px;
    overflow: hidden;
  }
  .progress-fill {
    height: 100%;
    background: var(--accent);
    border-radius: 3px;
    transition: width 0.3s ease;
  }
  .log-panel {
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 10px 12px;
    max-height: 140px;
    overflow-y: auto;
    font-family: "SF Mono", "Fira Code", "Cascadia Code", monospace;
    font-size: 11px;
  }
  .log-line {
    padding: 2px 0;
    color: var(--text-muted);
  }
  .log-line.error {
    color: var(--accent);
  }
</style>
