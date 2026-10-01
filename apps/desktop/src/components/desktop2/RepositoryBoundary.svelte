<script lang="ts">
  let { root, applied = false, uncertain = false, pending = false, paths = [] }: {
    root: string; applied?: boolean; uncertain?: boolean; pending?: boolean; paths?: string[];
  } = $props();
</script>

<aside class="repository-boundary" class:confirmed={applied && !uncertain && !pending} aria-label="Repository Apply boundary">
  <span class="boundary-label">Repository Apply boundary</span>
  <h3>Canonical repository</h3>
  <strong class="repository-name">{root.split(/[\\/]/).pop() || root}</strong>
  <p class="boundary-outcome">{uncertain ? "Repository outcome needs reconciliation" : pending ? "Apply in progress · outcome not confirmed" : applied ? "Apply recorded" : "No confirmed Apply"}</p>
  {#if paths.length}<small>{applied ? "Recorded candidate paths" : "Prepared destinations"}</small><ul aria-label="Prepared destination paths">{#each paths.slice(0, 4) as path}<li title={path}>{path}</li>{/each}</ul>{#if paths.length > 4}<small>+{paths.length - 4} more prepared paths</small>{/if}{/if}
  <p class="boundary-note">{applied ? "Recorded outcome, not a live filesystem check." : "Review does not change this repository. Apply is a separate decision."}</p>
  <details><summary>Destination path</summary><code>{root}</code></details>
</aside>

<style>

  .repository-boundary{position:relative;min-width:0;padding:24px 22px 24px 30px;border-left:1px solid var(--pytxo-line);display:flex;flex-direction:column;gap:14px;background:var(--pytxo-surface-shell)}
  .repository-boundary::before{content:"";position:absolute;left:-2px;top:44px;bottom:28px;width:3px;background:var(--pytxo-aperture);opacity:.8}
  .boundary-label{font-size:10px;letter-spacing:.08em;text-transform:uppercase;color:var(--pytxo-text-muted)}
  h3{margin:0;font-size:13px;font-weight:500}.repository-name{font-size:17px;overflow-wrap:anywhere}.boundary-outcome{margin:0;font-size:12px;color:var(--pytxo-text-soft)}
  .confirmed .boundary-outcome{color:var(--state-verified)}
  .confirmed::after{content:"";position:absolute;left:-12px;top:110px;width:24px;border-top:2px solid var(--state-verified);animation:recorded-apply 220ms ease-out}
  @keyframes recorded-apply{from{opacity:0}to{opacity:1}}
  @media(prefers-reduced-motion:reduce){.confirmed::after{animation:none}}
  ul{list-style:none;margin:6px 0;padding:0 0 0 14px;border-left:1px solid var(--pytxo-line);display:grid;gap:14px}li{position:relative;font:11px "IBM Plex Mono",monospace;overflow-wrap:anywhere}li::before{content:"";position:absolute;left:-14px;top:8px;width:8px;border-top:1px solid var(--pytxo-line)}
  small,.boundary-note,details{font-size:11px;color:var(--pytxo-text-muted);line-height:1.6}.boundary-note{margin:auto 0 0;padding-top:22px}summary{cursor:pointer}code{display:block;margin-top:8px;overflow-wrap:anywhere}summary:focus-visible{outline:2px solid var(--pytxo-accent);outline-offset:3px}
  @media(max-width:1100px){.repository-boundary{border-left:0;border-top:1px solid var(--pytxo-line);padding:18px 22px;gap:8px}.repository-boundary::before{left:0;right:0;top:-1px;bottom:auto;width:auto;height:2px;background:var(--pytxo-aperture-horizontal)}.boundary-note{padding-top:4px}.repository-boundary ul{display:none}}
  @container run-review (max-width:1000px){.repository-boundary{border-left:0;border-top:1px solid var(--pytxo-line);padding:16px}.repository-boundary::before{top:0;left:0;right:0;bottom:auto;height:2px;width:auto;background:var(--pytxo-aperture-horizontal)}.repository-boundary ul{display:none}.boundary-note{padding-top:0}}
</style>
