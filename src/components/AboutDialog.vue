<script setup lang="ts">
import { useI18n } from "vue-i18n";

const emit = defineEmits<{ close: [] }>();
const { t } = useI18n();
</script>

<template>
  <div class="overlay" role="presentation" @click.self="emit('close')">
    <div class="dialog" role="dialog" :aria-label="t('about.title')">
      <h1>{{ t("app.name") }}</h1>
      <p>{{ t("about.stack") }}</p>
      <p>{{ t("app.aboutUnofficial") }}</p>
      <button type="button" class="primary" @click="emit('close')">
        {{ t("about.close") }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  background: rgb(10 12 16 / 45%);
  display: grid;
  place-items: center;
  z-index: 40;
}

.dialog {
  width: min(28rem, calc(100% - 2rem));
  background: var(--jp-bg-elev);
  color: var(--jp-fg);
  border: 1px solid var(--jp-border);
  border-radius: var(--jp-radius);
  box-shadow: var(--jp-shadow);
  padding: 1rem;
}

h1 {
  margin: 0 0 0.5rem;
  font-size: 1.1rem;
}

p {
  margin: 0 0 0.6rem;
  color: var(--jp-muted);
}

.primary {
  border: 0;
  background: var(--jp-accent);
  color: var(--jp-accent-fg);
  border-radius: 4px;
  padding: var(--jp-pad);
}
</style>
