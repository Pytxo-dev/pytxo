<script lang="ts">
  import { onMount } from "svelte";
  import type { DesktopBackend } from "../../lib/desktop-backend";
  import type { PreparedRunFile } from "../../lib/types";
  let { backend, domainId, runId, packageDigest, file }: { backend: DesktopBackend; domainId: string; runId: string; packageDigest: string; file: PreparedRunFile } = $props();
  type Side = "before" | "after";
  type Content = { bytes: Uint8Array; complete: boolean; loading: boolean; error: string };
  const empty = (): Content => ({ bytes: new Uint8Array(), complete: false, loading: false, error: "" });
  let content = $state<Record<Side, Content>>({ before: empty(), after: empty() });
  let disposed = false;
  onMount(() => { disposed = false; void load("before"); void load("after"); return () => { disposed = true; }; });
  async function load(side: Side) {
    const current = content[side];
    const digest = side === "before" ? file.before_sha256 : file.after_sha256;
    if (!digest || current.complete || current.loading) return;
    current.loading = true; current.error = "";
    try {
      const offset = current.bytes.length;
      const part = await backend.runReviewContent(runId, file.path, side, offset, 65536, domainId);
      if (disposed) return;
      const bytes = Uint8Array.from(atob(part.data_base64), c => c.charCodeAt(0));
      const length = side === "before" ? file.before_byte_count : file.after_byte_count;
      const binary = side === "before" ? file.before_is_binary : file.after_is_binary;
      if (part.run_id !== runId || part.path !== file.path || part.package_digest !== packageDigest || part.side !== side || part.digest !== digest || part.binary !== binary || part.byte_count !== length || part.offset !== offset || part.length !== bytes.length || part.next_offset !== offset + bytes.length || part.next_offset > length || part.complete !== (part.next_offset === length) || (!bytes.length && !part.complete)) throw new Error("Prepared content identity changed. Reopen the current package in Review.");
      const joined = new Uint8Array(offset + bytes.length); joined.set(current.bytes); joined.set(bytes, offset);
      content[side] = { bytes: joined, complete: part.complete, loading: false, error: "" };
    } catch (e) { if (!disposed) current.error = String(e); }
    finally { if (!disposed) content[side].loading = false; }
  }
  const decode = (bytes: Uint8Array, binary: boolean) => binary ? Array.from(bytes.slice(0, 256), b => b.toString(16).padStart(2, "0")).join(" ") : new TextDecoder().decode(bytes);
</script>
<section class="prepared-file" aria-label={`Frozen contents of ${file.path}`}>
  <h3>{file.path}</h3><p>Frozen package · {packageDigest}</p>
  {#each ["before", "after"] as value}
    {@const side = value as Side}
    {@const digest = side === "before" ? file.before_sha256 : file.after_sha256}
    {@const binary = (side === "before" ? file.before_is_binary : file.after_is_binary) ?? true}
    <section aria-label={`Prepared ${side}`}><h4>{side === "before" ? "Before" : "After"}</h4>
      {#if !digest}<p>No file on this side.</p>{:else}
        {#if content[side].error}<p class="error" role="alert">{content[side].error}</p>{/if}
        <small>{content[side].bytes.length.toLocaleString()} bytes loaded{binary ? " · binary, first 256 bytes shown as hex" : ""}</small>
        <!-- Keyboard-readable frozen contents; never editable or executable. -->
        <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
        <pre tabindex="0" role="region" aria-label={`${side} content`}>{decode(content[side].bytes, binary)}</pre>
        {#if !content[side].complete}<button disabled={content[side].loading || content[side].bytes.length >= 262144} onclick={() => load(side)}>{content[side].loading ? "Loading…" : "Load next 64 KiB"}</button>{/if}
        {#if !content[side].complete && content[side].bytes.length >= 262144}<p>Dock limit reached. Open Review to inspect the complete file.</p>{/if}
      {/if}
    </section>
  {/each}
</section>
<style>
  .prepared-file{padding:12px;min-width:0}h3{font-size:13px;overflow-wrap:anywhere}h4{font-size:12px;margin:12px 0 6px}p,small{font-size:11px;line-height:1.5;overflow-wrap:anywhere;color:var(--pytxo-text-muted)}pre{max-height:360px;overflow:auto;white-space:pre-wrap;overflow-wrap:anywhere;padding:10px;border:1px solid var(--pytxo-line-soft);font:12px/1.6 "IBM Plex Mono",monospace;background:var(--pytxo-surface-raised);color:var(--pytxo-text-soft)}button{padding:6px 10px;border:1px solid var(--pytxo-line);border-radius:4px;background:transparent;color:var(--pytxo-text-strong);font:inherit;cursor:pointer}.error{color:var(--state-refuted)}button:focus-visible,pre:focus-visible{outline:2px solid var(--pytxo-accent);outline-offset:2px}
</style>
