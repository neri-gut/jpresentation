<script setup lang="ts">
import { storeToRefs } from "pinia";
import { onMounted, onUnmounted } from "vue";
import { useI18n } from "vue-i18n";
import { RouterLink, RouterView } from "vue-router";

import AboutDialog from "@/components/AboutDialog.vue";
import RightPanel from "@/components/RightPanel.vue";
import SettingsDialog from "@/components/SettingsDialog.vue";
import { registeredFeatures } from "@/features/registry";
import { uiLocales } from "@/i18n";
import { useProfileStore } from "@/stores/profile";
import { useUiStore } from "@/stores/ui";
import type { AppErrorDto } from "@/types/dto";

const { t } = useI18n();
const features = registeredFeatures();
const profiles = useProfileStore();
const ui = useUiStore();
const { toast, aboutOpen, settingsOpen, openMenu } = storeToRefs(ui);

function closeMenus(): void {
  openMenu.value = null;
}

function toggleMenu(id: "languages" | "tools" | "settings"): void {
  openMenu.value = openMenu.value === id ? null : id;
}

async function chooseLocale(locale: string): Promise<void> {
  closeMenus();
  if (!profiles.current) {
    return;
  }
  try {
    await profiles.update({ id: profiles.current.id, ui_locale: locale });
  } catch (err) {
    ui.showToast(errorMessage(err));
  }
}

function openAbout(): void {
  closeMenus();
  aboutOpen.value = true;
}

function openSettings(): void {
  closeMenus();
  settingsOpen.value = true;
}

function errorMessage(err: unknown): string {
  if (err && typeof err === "object" && "message" in err) {
    return (err as AppErrorDto).message;
  }
  return t("errors.invokeFailed");
}

function onDocumentClick(): void {
  closeMenus();
}

onMounted(() => {
  document.addEventListener("click", onDocumentClick);
});

onUnmounted(() => {
  document.removeEventListener("click", onDocumentClick);
});
</script>

<template>
  <div class="shell">
    <header class="menubar" @click.stop>
      <div class="brand">{{ t("app.name") }}</div>
      <div class="menus">
        <div class="menu">
          <button type="button" class="menu-btn" @click="toggleMenu('languages')">
            {{ t("menu.languages") }}
          </button>
          <ul v-if="openMenu === 'languages'" class="dropdown" role="menu">
            <li v-for="locale in uiLocales" :key="locale.id">
              <button
                type="button"
                role="menuitem"
                :aria-current="profiles.current?.ui_locale === locale.id"
                @click="chooseLocale(locale.id)"
              >
                {{ locale.name }}
              </button>
            </li>
          </ul>
        </div>
        <div class="menu">
          <button type="button" class="menu-btn" @click="toggleMenu('tools')">
            {{ t("menu.tools") }}
          </button>
          <ul v-if="openMenu === 'tools'" class="dropdown" role="menu">
            <li>
              <button type="button" role="menuitem" @click="openAbout">
                {{ t("menu.about") }}
              </button>
            </li>
          </ul>
        </div>
        <div class="menu">
          <button type="button" class="menu-btn" @click="openSettings">
            {{ t("menu.settings") }}
          </button>
        </div>
      </div>
    </header>

    <nav class="tabs" :aria-label="t('app.name')">
      <RouterLink
        v-for="feature in features"
        :key="feature.id"
        :to="feature.path === '' ? '/' : `/${feature.path}`"
        class="tab"
      >
        {{ t(feature.navKey) }}
      </RouterLink>
    </nav>

    <div class="body">
      <main class="main">
        <RouterView />
      </main>
      <RightPanel />
    </div>

    <p v-if="toast" class="toast" role="status">{{ toast }}</p>
    <AboutDialog v-if="aboutOpen" @close="aboutOpen = false" />
    <SettingsDialog v-if="settingsOpen" @close="settingsOpen = false" />
  </div>
</template>

<style scoped>
.shell {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--jp-bg);
  color: var(--jp-fg);
}

.menubar {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  height: var(--jp-menubar);
  padding: 0 0.6rem;
  border-bottom: 1px solid var(--jp-border);
  background: var(--jp-bg-elev);
}

.brand {
  font-weight: 650;
  font-size: var(--jp-title);
}

.menus {
  display: flex;
  gap: 0.15rem;
}

.menu {
  position: relative;
}

.menu-btn,
.tab {
  border: 0;
  background: transparent;
  padding: var(--jp-pad);
  border-radius: 4px;
  color: var(--jp-fg);
  text-decoration: none;
}

.menu-btn:hover,
.tab:hover {
  background: color-mix(in srgb, var(--jp-accent) 12%, transparent);
}

.dropdown {
  position: absolute;
  z-index: 20;
  top: 100%;
  left: 0;
  min-width: 12rem;
  margin: 0;
  padding: 0.25rem;
  list-style: none;
  background: var(--jp-bg-elev);
  border: 1px solid var(--jp-border);
  border-radius: var(--jp-radius);
  box-shadow: var(--jp-shadow);
}

.dropdown button {
  display: block;
  width: 100%;
  text-align: start;
  border: 0;
  background: transparent;
  padding: var(--jp-pad);
  border-radius: 4px;
}

.dropdown button:hover,
.dropdown button[aria-current="true"] {
  background: color-mix(in srgb, var(--jp-accent) 16%, transparent);
}

.tabs {
  display: flex;
  gap: 0.1rem;
  height: var(--jp-tabbar);
  padding: 0 0.4rem;
  border-bottom: 1px solid var(--jp-border);
  background: var(--jp-bg-elev);
  overflow-x: auto;
}

.tab.router-link-active,
.tab.router-link-exact-active {
  color: var(--jp-accent);
  box-shadow: inset 0 -2px 0 var(--jp-accent);
  font-weight: 650;
}

.body {
  display: flex;
  flex: 1;
  min-height: 0;
}

.main {
  flex: 1;
  min-width: 0;
  overflow: auto;
  padding: 0.75rem;
}

.toast {
  position: fixed;
  bottom: 0.75rem;
  left: 0.75rem;
  margin: 0;
  padding: 0.45rem 0.7rem;
  background: var(--jp-bg-elev);
  border: 1px solid var(--jp-border);
  border-radius: var(--jp-radius);
  box-shadow: var(--jp-shadow);
  color: var(--jp-fg);
}
</style>
