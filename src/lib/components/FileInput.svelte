<script lang="ts">
  import { open, save } from "@tauri-apps/plugin-dialog";

  let {
    inputPath = $bindable(""),
    outputPath = $bindable(""),
  }: {
    inputPath: string;
    outputPath: string;
  } = $props();

  let dragOver = $state(false);
  let inputFilename = $derived(
    inputPath.split("/").pop()?.split("\\").pop() ?? ""
  );

  function autoOutputPath(input: string): string {
    const parts = input.split(".");
    const ext = parts.pop() ?? "mp4";
    return parts.join(".") + "_cut." + ext;
  }

  async function browseInput() {
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: "Video",
          extensions: [
            "mp4", "mkv", "avi", "mov", "webm", "flv", "wmv", "m4v",
          ],
        },
        { name: "All Files", extensions: ["*"] },
      ],
    });
    if (selected) {
      inputPath = selected as string;
      outputPath = autoOutputPath(inputPath);
    }
  }

  async function browseOutput() {
    const selected = await save({
      filters: [
        { name: "Video", extensions: ["mp4", "mkv", "avi", "mov", "webm"] },
      ],
    });
    if (selected) {
      outputPath = selected as string;
    }
  }

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    dragOver = true;
  }

  function handleDragLeave() {
    dragOver = false;
  }

  function handleDrop(e: DragEvent) {
    e.preventDefault();
    dragOver = false;
    const files = e.dataTransfer?.files;
    if (files && files.length > 0) {
      const file = files[0];
      if ("path" in file) {
        inputPath = (file as any).path;
        outputPath = autoOutputPath(inputPath);
      }
    }
  }
</script>

<section
  class="file-drop"
  class:drag-over={dragOver}
  ondragover={handleDragOver}
  ondragleave={handleDragLeave}
  ondrop={handleDrop}
  role="button"
  tabindex="0"
  onclick={browseInput}
  onkeydown={(e) => e.key === "Enter" && browseInput()}
>
  {#if inputPath}
    <div class="file-selected">
      <span class="file-icon">&#9658;</span>
      <span class="file-name">{inputFilename}</span>
    </div>
  {:else}
    <div class="file-placeholder">
      <span class="drop-icon">+</span>
      <span>Drop video here or click to browse</span>
    </div>
  {/if}
</section>

{#if inputPath}
  <div class="output-row">
    <label>Output:</label>
    <input type="text" bind:value={outputPath} class="output-input" />
    <button class="btn-small" onclick={browseOutput}>...</button>
  </div>
{/if}

<style>
  .file-drop {
    border: 2px dashed var(--border);
    border-radius: var(--radius-lg);
    padding: 24px;
    text-align: center;
    cursor: pointer;
    transition: all 0.2s;
  }
  .file-drop:hover,
  .file-drop.drag-over {
    border-color: var(--accent);
    background: color-mix(in srgb, var(--accent) 5%, transparent);
  }
  .file-placeholder {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    color: var(--text-muted);
  }
  .drop-icon {
    font-size: 28px;
    font-weight: 300;
    color: var(--text-dim);
  }
  .file-selected {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
  }
  .file-icon {
    color: var(--accent);
  }
  .file-name {
    font-weight: 500;
    word-break: break-all;
  }
  .output-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .output-row label {
    font-size: 12px;
    color: var(--text-muted);
    white-space: nowrap;
  }
  .output-input {
    flex: 1;
    font-size: 12px;
  }
  .btn-small {
    background: var(--bg-input);
    color: var(--text-muted);
    padding: 6px 10px;
    border: 1px solid var(--border);
  }
  .btn-small:hover {
    background: var(--bg-hover);
  }
</style>
