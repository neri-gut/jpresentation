import { createI18n } from "vue-i18n";

import de from "@/locales/de.json";
import en from "@/locales/en.json";
import es from "@/locales/es.json";
import fr from "@/locales/fr.json";
import id from "@/locales/id.json";
import index from "@/locales/index.json";
import it from "@/locales/it.json";
import ja from "@/locales/ja.json";
import ko from "@/locales/ko.json";
import nl from "@/locales/nl.json";
import pl from "@/locales/pl.json";
import ptBR from "@/locales/pt-BR.json";
import ru from "@/locales/ru.json";
import uk from "@/locales/uk.json";
import zhHans from "@/locales/zh-Hans.json";
import zhHant from "@/locales/zh-Hant.json";

/** Catalog of UI locales shipped with the app. Source and fallback are English. */
export interface UiLocaleInfo {
  id: string;
  name: string;
  dir: "ltr" | "rtl";
}

/** Locale index loaded from `locales/index.json`. */
export const uiLocaleIndex = index;

/** UI locales the operator can pick without a Rust change. */
export const uiLocales = index.locales as UiLocaleInfo[];

/**
 * vue-i18n instance. Default and fallback are `en`. Missing keys in other files resolve to English.
 */
export const i18n = createI18n({
  legacy: false,
  globalInjection: true,
  locale: index.default,
  fallbackLocale: index.fallback,
  missingWarn: false,
  fallbackWarn: false,
  messages: {
    de,
    en,
    es,
    fr,
    id,
    it,
    ja,
    ko,
    nl,
    pl,
    "pt-BR": ptBR,
    ru,
    uk,
    "zh-Hans": zhHans,
    "zh-Hant": zhHant,
  },
});

/**
 * Applies a BCP-47 UI locale. Unknown ids fall back to English at render time.
 */
export function setUiLocale(locale: string): void {
  i18n.global.locale.value = locale as typeof i18n.global.locale.value;
}
