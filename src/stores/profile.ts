import { defineStore } from "pinia";
import { computed, ref } from "vue";

import { invokeCommand } from "@/composables/invoke";
import { setUiLocale } from "@/i18n";
import type {
  CreateProfileDto,
  ProfileDto,
  SelectProfileDto,
  UpdateProfileDto,
} from "@/types/dto";

/**
 * Active congregation profile. Source of truth is SQLite; this store is a cache of the last IPC result.
 */
export const useProfileStore = defineStore("profile", () => {
  const profiles = ref<ProfileDto[]>([]);
  const current = ref<ProfileDto | null>(null);

  const currentId = computed(() => current.value?.id ?? null);

  /** Loads the profile list and the last-used row. */
  async function hydrate(): Promise<void> {
    const list = await invokeCommand<ProfileDto[]>("profile_list");
    profiles.value = list;
    current.value = list[0] ?? null;
    const selected = list.reduce<ProfileDto | null>((best, item) => {
      if (!best) {
        return item;
      }
      return item.updated_at > best.updated_at ? item : best;
    }, null);
    if (selected) {
      current.value = selected;
      setUiLocale(selected.ui_locale);
    }
  }

  /** Creates a profile (UI locale `en`) and selects it. */
  async function create(name: string): Promise<ProfileDto> {
    const payload: CreateProfileDto = { name };
    const created = await invokeCommand<ProfileDto, CreateProfileDto>(
      "profile_create",
      payload,
    );
    await select(created.id);
    return created;
  }

  /** Marks a profile as last used and reloads the list. */
  async function select(id: string): Promise<void> {
    const payload: SelectProfileDto = { id };
    const selected = await invokeCommand<ProfileDto, SelectProfileDto>(
      "profile_select",
      payload,
    );
    current.value = selected;
    setUiLocale(selected.ui_locale);
    const list = await invokeCommand<ProfileDto[]>("profile_list");
    profiles.value = list;
  }

  /** Persists UI locale (and optional name) on the current profile. */
  async function update(payload: UpdateProfileDto): Promise<void> {
    const updated = await invokeCommand<ProfileDto, UpdateProfileDto>(
      "profile_update",
      payload,
    );
    current.value = updated;
    setUiLocale(updated.ui_locale);
    profiles.value = profiles.value.map((item) =>
      item.id === updated.id ? updated : item,
    );
  }

  return {
    profiles,
    current,
    currentId,
    hydrate,
    create,
    select,
    update,
  };
});
