<script lang="ts">
  import { ICONS, type IconName } from "./icons";

  interface Props {
    name: IconName;
    size?: number;
    color?: string;
  }

  let { name, size = 18, color = "currentColor" }: Props = $props();

  let def = $derived(ICONS[name]);
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill={def.fill === "currentColor" ? color : "none"}
  stroke={def.fill === "currentColor" ? "none" : color}
  stroke-width={def.strokeWidth}
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
  focusable="false"
>
  <!-- Trazos propios, definidos en icons.ts — nunca contenido de terceros. -->
  <!-- eslint-disable-next-line svelte/no-at-html-tags -->
  {@html def.paths}
</svg>
