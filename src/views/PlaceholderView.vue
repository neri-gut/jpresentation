<script setup lang="ts">
import { useI18n } from "vue-i18n";

defineProps<{
  titleKey: string;
  bodyKey: string;
}>();

const { t } = useI18n();
</script>

<template>
  <section class="placeholder">
    <h1>{{ t(titleKey) }}</h1>
    <p>{{ t(bodyKey) }}</p>
    <p class="hint">{{ t("views.placeholder") }}</p>
  </section>
</template>

<style scoped>
.placeholder {
  max-width: 40rem;
}

h1 {
  margin: 0 0 0.4rem;
  font-size: 1.15rem;
}

p {
  margin: 0 0 0.5rem;
}

.hint {
  color: var(--jp-muted);
}
</style>
