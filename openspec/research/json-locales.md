# JSON locales

Hay dos familias. No se mezclan.

## 1. UI de JPresentation (nosotros + comunidad)

Carpeta de la app (cuando exista código): `locales/`.  
Contrato de muestra: `openspec/research/locales-ui/`.

```
locales/
  index.json     source=en fallback=en default=en
  en.json        fuente de claves
  es.json fr.json de.json it.json pt-BR.json
  ru.json uk.json pl.json nl.json id.json
  ja.json ko.json zh-Hans.json zh-Hant.json
```

### `index.json`

```json
{
  "source": "en",
  "fallback": "en",
  "default": "en",
  "locales": [
    { "id": "en", "name": "English", "dir": "ltr" }
  ]
}
```

- `id`: BCP-47 (`es`, `en`, `pt-BR`, `zh-Hans`).
- `dir`: `ltr` | `rtl`.
- Añadir idioma = nuevo `{id}.json` + una fila aquí.

### `{id}.json`

- UTF-8, JSON estricto (no comentarios).
- Árbol anidado por módulo (`nav`, `songs`, `timer`…), el mismo que los feature modules Vue.
- Claves en camelCase, estables; solo se traducen **valores**.
- Interpolación vue-i18n: `{name}`, `{n}`, `{time}` — no `{{name}}` (eso es i18next).
- Plurales vue-i18n: `"pendiente | {n} pendientes"` o mensaje ICU si el runtime lo activa. Un traductor no inventa claves `_one` / `_other` salvo que el manifiesto lo pida.
- Prohibido HTML en valores salvo claves `*Html` documentadas.
- Misma forma de árbol que `es.json`. Falta una clave → fallback a `es`.

Runtime: vue-i18n. El shell lee `index.json` y carga el JSON del `ui_locale` del perfil.

## 2. Catálogo JW de lenguas de contenido (no se edita a mano)

`GET https://www.jw.org/en/languages/` (Accept JSON):

```
{ "status": 200, "localizedCount": …,
  "languages": [
    {
      "symbol": "es",          // BCP-ish
      "langcode": "S",         // langwritten
      "name": "Spanish",
      "vernacularName": "español",
      "script": "ROMAN",
      "direction": "ltr",
      "isSignLanguage": false,
      "hasWebContent": true,
      "altSpellings": ["español", "espanol"]
    }
  ]
}
```

~1160 entradas. Cache en SQLite del perfil/app. **No** va en `locales/`.

`GETPUBMEDIALINKS?alllangs=1` trae un subconjunto (~503) con los mismos `langwritten` + `locale` + `direction`. Sirve para saber qué hay en publicaciones.

Selector de contenido: cache JW. Selector de UI: `index.json`.
