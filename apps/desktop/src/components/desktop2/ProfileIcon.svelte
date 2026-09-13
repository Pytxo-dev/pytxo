<script lang="ts">
  let { profile }: { profile: string } = $props();
</script>

<!-- Flat vector implementation of the approved 2D study. These are identities,
     not evidence badges; the neighboring text owns the permission semantics. -->
<svg class="profile-emblem" viewBox="0 0 40 40" width="28" height="28" aria-hidden="true" focusable="false">
  {#if profile === "deep_space" || profile === "deepspace"}
    <path d="M24 4A16 16 0 1 0 33 31A16 16 0 0 1 24 4Z" fill="currentColor" />
    <circle cx="30" cy="13" r="3.5" class="violet" />
  {:else if profile === "orbit"}
    <circle cx="20" cy="20" r="13" fill="none" stroke="currentColor" stroke-width="2.5" />
    <ellipse cx="20" cy="20" rx="20" ry="5.5" transform="rotate(-28 20 20)" fill="none" class="teal-stroke" stroke-width="2.5" />
  {:else if profile === "galaxy"}
    <circle cx="20" cy="20" r="3.5" class="violet" />
    <g fill="currentColor">
      <path d="M20 2C8 2 3 13 10 22C7 12 14 6 20 2Z" />
      <path d="M20 2C8 2 3 13 10 22C7 12 14 6 20 2Z" transform="rotate(120 20 20)" />
      <path d="M20 2C8 2 3 13 10 22C7 12 14 6 20 2Z" transform="rotate(240 20 20)" />
    </g>
  {:else if profile === "supernova"}
    <path d="M20 2C22 15 25 18 38 20C25 22 22 25 20 38C18 25 15 22 2 20C15 18 18 15 20 2Z" class="teal" />
  {:else}
    <circle cx="20" cy="20" r="12" fill="none" stroke="currentColor" stroke-width="2" />
  {/if}
</svg>

<style>
  .profile-emblem { display: block; flex: none; width: 28px; height: 28px; color: var(--foreground); }
  .violet { fill: light-dark(#7860bb, #af96ed); }
  .teal { fill: light-dark(#427d75, #83bdb1); }
  .teal-stroke { stroke: light-dark(#427d75, #83bdb1); }
</style>
