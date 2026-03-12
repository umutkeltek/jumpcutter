<script lang="ts">
  let {
    noiseDb = $bindable(-30),
    minSilenceDuration = $bindable(0.5),
    failureTolerance = $bindable(0.1),
    edgePadding = $bindable(0.1),
  }: {
    noiseDb: number;
    minSilenceDuration: number;
    failureTolerance: number;
    edgePadding: number;
  } = $props();
</script>

<section class="settings-group">
  <h2>Detection</h2>

  <div class="setting">
    <div class="setting-header">
      <label>Silence Threshold</label>
      <span class="value">{noiseDb} dB</span>
    </div>
    <input type="range" min="-60" max="-10" step="1" bind:value={noiseDb} />
  </div>

  <div class="setting">
    <div class="setting-header">
      <label>Min Silence Duration</label>
      <span class="value">{minSilenceDuration.toFixed(1)}s</span>
    </div>
    <input
      type="range"
      min="0.1"
      max="5.0"
      step="0.1"
      bind:value={minSilenceDuration}
    />
  </div>

  <div class="setting">
    <div class="setting-header">
      <label>Failure Tolerance</label>
      <span class="value">{failureTolerance.toFixed(2)}</span>
    </div>
    <input
      type="range"
      min="0"
      max="1.0"
      step="0.01"
      bind:value={failureTolerance}
    />
  </div>

  <div class="setting">
    <div class="setting-header">
      <label>Edge Padding</label>
      <span class="value">{edgePadding.toFixed(1)}s</span>
    </div>
    <input
      type="range"
      min="0"
      max="2.0"
      step="0.1"
      bind:value={edgePadding}
    />
  </div>
</section>
