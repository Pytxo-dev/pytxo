<script lang="ts">
  type ImageAsset = { mode: "image"; src: string };
  type MaskAsset = { mode: "mask"; src: string; color?: string };
  type ThemedAsset = { mode: "themed"; onDark: string; onLight: string };
  type AdeAsset = ImageAsset | MaskAsset | ThemedAsset;

  let { id }: { id: string } = $props();

  const assets: Record<string, AdeAsset> = {
    claude: { mode: "mask", src: "/ade/anthropic.svg" },
    agy: { mode: "image", src: "/ade/antigravity.png" },
    codex: {
      mode: "themed",
      onDark: "/ade/openai-on-dark.svg",
      onLight: "/ade/openai-on-light.svg",
    },
    cursor: {
      mode: "themed",
      onDark: "/ade/cursor-on-dark.svg",
      onLight: "/ade/cursor-on-light.svg",
    },
    opencode: { mode: "image", src: "/ade/opencode.svg" },
    gemini: { mode: "mask", src: "/ade/gemini.svg", color: "#8e75b2" },
    copilot: { mode: "mask", src: "/ade/copilot.svg" },
    aider: { mode: "image", src: "/ade/aider.png" },
    grok: {
      mode: "themed",
      onDark: "/ade/grok-on-dark.svg",
      onLight: "/ade/grok-on-light.svg",
    },
    droid: { mode: "mask", src: "/ade/factory-droid.svg" },
    cline: { mode: "mask", src: "/ade/cline.svg" },
    goose: { mode: "image", src: "/ade/goose.svg" },
    qwen: { mode: "image", src: "/ade/qwen.svg" },
    kimi: { mode: "mask", src: "/ade/kimi.svg" },
  };

  let failedId = $state<string | null>(null);
  const asset = $derived(assets[id]);
  const maskStyle = $derived(
    asset?.mode === "mask"
      ? `--ade-mask: url("${asset.src}"); --ade-mark-color: ${asset.color ?? "currentColor"}`
      : "",
  );
</script>

{#if asset && failedId !== id}
  <span class="ade-identity" data-ade-id={id} aria-hidden="true">
    {#if asset.mode === "mask"}
      <span class="ade-mark" style={maskStyle}></span>
    {:else if asset.mode === "themed"}
      <img class="ade-logo ade-logo-on-dark" src={asset.onDark} alt="" width="20" height="20" onerror={() => { failedId = id; }} />
      <img class="ade-logo ade-logo-on-light" src={asset.onLight} alt="" width="20" height="20" onerror={() => { failedId = id; }} />
    {:else}
      <img class="ade-logo" src={asset.src} alt="" width="20" height="20" onerror={() => { failedId = id; }} />
    {/if}
  </span>
{/if}

<style>
  .ade-identity {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    flex: none;
    color: inherit;
    background: transparent;
  }

  .ade-logo,
  .ade-mark {
    display: block;
    width: 20px;
    height: 20px;
  }

  .ade-logo {
    object-fit: contain;
  }

  .ade-identity[data-ade-id="codex"] .ade-logo {
    width: 24px;
    height: 24px;
  }

  .ade-identity[data-ade-id="agy"] .ade-logo {
    width: 24px;
    height: 24px;
  }

  .ade-mark {
    background: var(--ade-mark-color, currentColor);
    -webkit-mask: var(--ade-mask) center / contain no-repeat;
    mask: var(--ade-mask) center / contain no-repeat;
  }

  .ade-logo-on-light {
    display: none;
  }

  :global(html[data-chroma-theme="light"]) .ade-logo-on-dark {
    display: none;
  }

  :global(html[data-chroma-theme="light"]) .ade-logo-on-light {
    display: block;
  }
</style>
