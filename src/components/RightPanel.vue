<script setup lang="ts">
import { storeToRefs } from "pinia";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import { errorMessage } from "@/composables/errors";
import { useProfileStore } from "@/stores/profile";
import { useUiStore } from "@/stores/ui";

const { t } = useI18n();
const ui = useUiStore();
const profiles = useProfileStore();
const { panel } = storeToRefs(ui);

const blocks = [
  { id: "songs", labelKey: "panel.songs" },
  { id: "media", labelKey: "panel.media" },
  { id: "bible", labelKey: "panel.bible" },
  { id: "timer", labelKey: "panel.timer" },
] as const;

const width = computed(() => (panel.value.collapsed ? 44 : panel.value.width));

async function toggle(): Promise<void> {
  if (!profiles.current) {
    return;
  }
  try {
    await ui.setPanel(profiles.current.id, {
      ...panel.value,
      collapsed: !panel.value.collapsed,
    });
  } catch (err) {
    ui.showToast(errorMessage(err, t));
  }
}
</script>

<template>
  <aside class="panel" :class="{ collapsed: panel.collapsed }" :style="{ width: `${width}px` }">
    <button
      type="button"
      class="collapse"
      :title="panel.collapsed ? t('panel.expand') : t('panel.collapse')"
      @click="toggle"
    >
      {{ panel.collapsed ? "«" : "»" }}
    </button>
    <section v-for="block in blocks" :key="block.id" class="block">
      <h2>{{ t(block.labelKey) }}</h2>
      <p v-if="!panel.collapsed">{{ t("panel.placeholder") }}</p>
    </section>
  </aside>
</template>

<style scoped>
.panel {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  border-left: 1px solid var(--jp-border);
  background: var(--jp-bg-elev);
  padding: 0.45rem;
  overflow: auto;
}

.collapsed .block h2 {
  writing-mode: vertical-rl;
  transform: rotate(180deg);
  font-size: 11px;
}

.collapsed .block p {
  display: none;
}

.collapse {
  align-self: flex-end;
  border: 1px solid var(--jp-border);
  background: var(--jp-bg);
  border-radius: 4px;
  padding: 0.1rem 0.35rem;
}

.block {
  border: 1px solid var(--jp-border);
  border-radius: var(--jp-radius);
  padding: 0.45rem;
}

.block h2 {
  margin: 0 0 0.25rem;
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--jp-muted);
}

.block p {
  margin: 0;
  color: var(--jp-muted);
}
</style>
