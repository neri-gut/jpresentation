<script setup lang="ts">
import { computed, onMounted } from "vue";
import { useRoute } from "vue-router";

const route = useRoute();
const label = computed(() => String(route.query.label ?? "Display"));
const size = computed(() => String(route.query.size ?? ""));

onMounted(() => {
  document.documentElement.dataset.surface = "identify";
  document.documentElement.style.background = "#1d4ed8";
  document.body.style.background = "#1d4ed8";
});
</script>

<template>
  <div class="identify">
    <p class="name">{{ label }}</p>
    <p v-if="size" class="size">{{ size }}</p>
  </div>
</template>

<style scoped>
.identify {
  height: 100%;
  display: grid;
  place-content: center;
  text-align: center;
  background: #1d4ed8;
  color: #fff;
  cursor: none;
  user-select: none;
  padding: 1rem;
}

.name {
  margin: 0;
  font-size: 1.6rem;
  font-weight: 650;
}

.size {
  margin: 0.35rem 0 0;
  font-variant-numeric: tabular-nums;
  opacity: 0.85;
}
</style>
