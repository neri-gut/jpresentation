<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  progressPct: number;
  remainingMs: number;
  compact?: boolean;
}>();

const overtime = computed(() => props.remainingMs < 0);
const width = computed(() => {
  if (overtime.value) {
    return 100;
  }
  return Math.min(100, Math.max(0, props.progressPct));
});
/** Stretch the gradient so the clip reveals green→red along the full bar. */
const innerWidth = computed(() => {
  const w = width.value;
  if (w <= 0) {
    return "10000%";
  }
  return `${(100 / w) * 100}%`;
});
</script>

<template>
  <div class="clock-bar" :class="{ overtime, compact: props.compact }">
    <div class="fill" :style="{ width: `${width}%` }">
      <i :style="{ width: innerWidth }" />
    </div>
  </div>
</template>

<style scoped>
.clock-bar {
  height: 12px;
  background: rgb(255 255 255 / 18%);
  overflow: hidden;
}

.clock-bar.compact {
  height: 6px;
  border-radius: 99px;
  background: color-mix(in srgb, var(--jp-fg) 12%, transparent);
}

.fill {
  height: 100%;
  overflow: hidden;
}

.fill i {
  display: block;
  height: 100%;
  background: linear-gradient(to right, #12b76a 0%, #eab308 55%, #f04438 100%);
}

.overtime .fill i {
  background: #f04438;
}
</style>
