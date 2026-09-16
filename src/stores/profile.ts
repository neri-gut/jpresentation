import { computed, ref } from "vue";
import { defineStore } from "pinia";

import { invokeCommand } from "@/composables/invoke";
import { setUiLocale } from "@/i18n";
import type {
  CreateProfileDto,
  DeleteProfileDto,
  DuplicateProfileDto,
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

  /** Persists name, UI locale, and/or content locale on a profile. */
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

  /** Copies a profile and its settings. Does not select the copy. */
  async function duplicate(id: string): Promise<ProfileDto> {
    const payload: DuplicateProfileDto = { id };
    const copied = await invokeCommand<ProfileDto, DuplicateProfileDto>(
      "profile_duplicate",
      payload,
    );
    const list = await invokeCommand<ProfileDto[]>("profile_list");
    profiles.value = list;
    return copied;
  }

  /** Deletes a profile. Rust returns the profile that remains selected. */
  async function remove(id: string): Promise<ProfileDto> {
    const payload: DeleteProfileDto = { id };
    const remaining = await invokeCommand<ProfileDto, DeleteProfileDto>(
      "profile_delete",
      payload,
    );
    current.value = remaining;
    setUiLocale(remaining.ui_locale);
    const list = await invokeCommand<ProfileDto[]>("profile_list");
    profiles.value = list;
    return remaining;
  }

  return {
    profiles,
    current,
    currentId,
    hydrate,
    create,
    select,
    update,
    duplicate,
    remove,
  };
});
