# Design: Perfil de congregación

## Technical Approach

Ampliar el puerto `ProfileStore` y las celdas de settings. Un comando = una acción. El auditorio no se recrea ni se cierra si falla un comando de consola.

Migración **v2** (idempotente, sobre la v1 de `001`):

- Recrear `settings` y `meeting_programs` con `ON DELETE CASCADE` hacia `profiles(id)`. SQLite no deja alterar una FK: `CREATE` + `INSERT SELECT` + `DROP` + `RENAME` en una transacción.
- No se añaden columnas a `profiles`. `content_locale` ya existe (defecto `'E'`).
- Tras migrar, `seed_default_profile` de v1 sigue siendo la garantía de ≥1 fila.

## DTOs

Todos camelCase en JSON de IPC (`serde rename_all` no: `001` serializa `snake_case` y Vue ya habla `ui_locale`. **Seguir snake_case** para no romper el store de `001`).

### Perfil

```
ProfileDto          { id, name, ui_locale, content_locale, media_provider, created_at, updated_at }
CreateProfileDto    { name }                          // ui_locale = en, content_locale = E
SelectProfileDto    { id }
UpdateProfileDto    { id, name?, ui_locale?, content_locale? }
DuplicateProfileDto { id }
DeleteProfileDto    { id }
```

`content_locale` es un `langwritten` JW (`E`, `S`, `CHS`, …), no un BCP-47.

Duplicar: nuevo UUID, `created_at`/`updated_at` ahora, `name` = `"{name} (copy)"` recortado a 80 caracteres. Copia **todas** las filas de `settings` del origen. No selecciona el duplicado (la UI MAY seleccionarlo después). Si el nombre resultante choca, se admite: no hay UNIQUE en `name`.

Borrar: si `COUNT(*) = 1` → `AppError::LastProfile`. Si el borrado es el perfil activo, el store selecciona el más reciente de los que quedan (`ORDER BY updated_at DESC`) y `profile_delete` devuelve ese `ProfileDto` para que Vue rehidrate. Confirmación **solo en Vue**; Rust no pregunta.

### Semilla de contenido

Fichero `src-tauri/src/domain/content_languages.json`, embebido con `include_str!`. Las 21 lenguas de `research/idiomas.md`:

`E S F I X T U J KO CHS CH A G O P K IN TG AM HI Z`

```
ContentLanguageDto { langwritten, name, locale, direction, script }
```

`direction`: `ltr` | `rtl`. `script` MAY ser `""` si la semilla no lo trae. El comando `content_languages_list` no pega a red. Un `content_locale` que no esté en la semilla se rechaza en `profile_update` / `profile_create` (create siempre `E`, que sí está).

El selector de Configuración es filtrable por `name` o `langwritten` (cliente). 21 ítems; el filtro queda listo para cuando el catálogo vivo traiga ~500.

### Horario de reunión

Nueva celda `settings.key = "meeting_schedule"`:

```
MeetingScheduleSetting {
  midweek_weekday: u8,   // ISO-8601 1=lunes … 7=domingo
  midweek_time: String,  // "HH:mm" 24h, hora local del equipo
  weekend_weekday: u8,
  weekend_time: String
}
```

Defecto: martes 19:00 / domingo 10:00 → `{ 2, "19:00", 7, "10:00" }`. Validación: weekday ∈ 1..=7; tiempo `^([01]\d|2[0-3]):[0-5]\d$`. Zona: la del SO; no se guarda IANA.

Este DTO es el que leerá el cronómetro (`014` / plantillas). Aquí solo se persiste y se edita.

### Apariencia

`AppearanceSetting` ya existe. Este change fija el contrato que `001` dejó hueco:

```
theme:   "system" | "light" | "dark"     // defecto system
accent:  "blue" | "teal" | "violet" | "amber"   // defecto blue
density: "compact" | "comfortable"       // defecto compact
```

Paleta (AA sobre el fondo claro y el oscuro; no hex libre):

| token  | light `--jp-accent` | dark `--jp-accent` |
|--------|---------------------|--------------------|
| blue   | `#1d4ed8`           | `#60a5fa`          |
| teal   | `#0f766e`           | `#2dd4bf`          |
| violet | `#6d28d9`           | `#c4b5fd`          |
| amber  | `#b45309`           | `#fbbf24`          |

`--jp-accent-fg` se elige por contraste sobre el acento (blanco sobre light; tinta oscura sobre dark). Vue aplica `document.documentElement.dataset.accent` (ya lo hace `applyAppearance`). Auditorio y orador **no** leen estos tokens.

### Superficies

```
SurfacesSetting { audience_monitor_id, speaker_monitor_id, use_speaker }
SpeakerPlacement { Hidden | Preview | Monitor(id) }
```

Regla de colocación (sustituye el `Option` que hoy oculta el orador):

| `use_speaker` | `speaker_monitor_id` | Resultado |
|---------------|----------------------|-----------|
| false         | *                    | `Hidden` — no hay ventana orador |
| true          | `None`               | `Preview` — flotante 960×540, decorada, no fullscreen sobre la consola |
| true          | id existente y ≠ auditorio | `Monitor(id)` fullscreen |
| true          | id ausente o igual al auditorio | `Preview` + toast `MonitorMissing` en consola; el auditorio no se mueve |

`place_audience(None)` sigue: ≥2 monitores → secundaria; 1 monitor → preview. Sin cambios de semántica.

Identificar: `PlatformSurface::identify_monitors`. Por cada `MonitorDto`, una ventana transitoria `identify-{n}` (n = 0..), siempre encima, sin IPC, 2,0 s, luego `destroy`. Muestra **nombre + `width×height`**. No mueve auditorio ni orador. Capability propia `identify-cap` (`windows: ["identify-*"]`, `core:default` only).

## Comandos (solo `operator-cap`)

| Comando | Payload | Retorno | Errores |
|---------|---------|---------|---------|
| `profile_list` | — | `ProfileDto[]` | Db |
| `profile_create` | `CreateProfileDto` | `ProfileDto` | `Invariant` (nombre) |
| `profile_select` | `SelectProfileDto` | `ProfileDto` | `NotFound` |
| `profile_update` | `UpdateProfileDto` | `ProfileDto` | `NotFound`, `Invariant`, `UnknownContentLanguage` |
| `profile_duplicate` | `DuplicateProfileDto` | `ProfileDto` (el nuevo) | `NotFound` |
| `profile_delete` | `DeleteProfileDto` | `ProfileDto` (el que queda activo) | `NotFound`, `LastProfile` |
| `settings_get` / `settings_set` | `SettingKeyDto` / `SettingDto` | `SettingDto` | `Invariant` (key/JSON) |
| `content_languages_list` | — | `ContentLanguageDto[]` | — |
| `monitors_list` | — | `MonitorDto[]` | Invariant |
| `monitors_identify` | — | `()` | Invariant (best-effort: 0 monitores = Ok) |
| `output_get` | — | `OutputBundleDto` | — |

`settings_set` con `key = "surfaces"` sigue llamando a `ensure_surfaces`. `key = "meeting_schedule"` solo persiste. `key = "appearance"` no toca el stage.

Permisos: añadir `profile_duplicate`, `profile_delete`, `content_languages_list`, `monitors_identify` a `operator-commands`. Audience/speaker: sin cambios.

## Errores

Ampliar `AppError` (código estable en `AppErrorDto.code`):

| Variante | code | Cuándo |
|----------|------|--------|
| `LastProfile` | `LastProfile` | borrar el único perfil |
| `UnknownContentLanguage` | `UnknownContentLanguage` | `content_locale` fuera de la semilla |
| `MonitorMissing` | `MonitorMissing` | id guardado no está; se degrada a preview |

Vue mapea `code` → `errors.*` en `en.json`. Si el código no tiene clave, usa `errors.invokeFailed`. El toast es solo consola.

## Capas

```
Vue SettingsDialog  --invoke-->  commands/profile.rs | settings.rs | monitors.rs
                                 |                      |
                                 v                      v
                            ProfileStore          SettingsStore
                                 |                      |
                                 v                      v
                              rusqlite               rusqlite
```

`content_languages_list` lee el JSON embebido; no hay repo. `DesktopSurface` implementa `SpeakerPlacement` + `identify_monitors`. El dominio no importa Tauri.

`ProfileStore` gana `duplicate` y `delete`. `validate_profile_name` se reutiliza. `validate_content_locale(code, seed)` y `validate_meeting_schedule` viven en `domain/profile.rs` / `domain/schedule.rs` con tests sin WebView.

## UI (solo consola)

Configuración, mismas secciones, más densas:

1. **Perfiles** — select, nombre (renombrar in-place o campo + Guardar), Crear, Duplicar, Borrar (diálogo `common.ok` / `common.cancel` + texto `settings.confirmDelete`).
2. **Idioma de contenido** — `<select>` filtrable. El menú **Idiomas** sigue siendo solo UI.
3. **Reuniones** — dos filas: entre semana / fin de semana; weekday (nombres i18n `weekday.1`…`weekday.7`) + `<input type="time">`.
4. **Tema** — theme (ya) + acento (4 swatches) + densidad (2 radios).
5. **Monitores** — audience, use speaker, speaker (ya, speaker ≠ audience) + botón Identificar.

Claves nuevas **solo** en `src/locales/en.json`. El resto de JSON cae al fallback `en`.

No hay wizard. El perfil `Default` de `001` recibe los defaults de schedule/appearance la primera vez que `settings_get` pide la clave.

## CI

Nuevo workflow `.github/workflows/ci.yml` (PR + `main`):

| Job | Qué |
|-----|-----|
| `frontend` | Node 22, `npm ci --ignore-scripts`, `npm run build` (`vue-tsc --noEmit` + Vite) |
| `rust` | `dtolnay/rust-toolchain`, deps WebKitGTK 4.1 de Tauri en Ubuntu, `cargo test --manifest-path src-tauri/Cargo.toml --locked` |
| `audit` | el job `npm-audit` de `supply-chain.yml` se queda; se añade `cargo audit` (`rustsec/audit-check` o `cargo install cargo-audit` + `cargo audit`) |

Sin `npm install` suelto. Sin `postinstall` nuevo. Clippy queda fuera (MAY en un change de DX).

## Tests de dominio (sin WebView)

- `validate_profile_name` (ya)
- `new_profile_defaults_ui_locale_to_en` (ya) — ampliar: `content_locale == "E"`
- duplicar copia `appearance` + `meeting_schedule` y genera otro id
- borrar el último → `LastProfile`
- borrar el activo → el store deja seleccionado otro
- `content_locale` desconocido → `UnknownContentLanguage`
- schedule: weekday 0 / 8 y `"9:00"` / `"24:00"` rechazados; `"19:00"` ok
- migración v2: FK cascade, el seed Default sigue ahí
- `SpeakerPlacement` se cubre con un fake `PlatformSurface` en test unitario del mapeo `SurfacesSetting → SpeakerPlacement`, no con ventanas reales

## Risks

- IDs de monitor `{name}@{x},{y}` siguen frágiles si el SO reordena. Fuera de alcance; Identificar mitiga el síntoma.
- Ventanas `identify-*` en Linux/Wayland pueden no ir “always on top”. 2 s + nombre en título sigue siendo suficiente.
- `cargo audit` en PRs de fork necesita token; el job MUST degradar a warning si el token no está, no bloquear el resto del CI.
- Recrear tablas en v2 sobre una DB de desarrollo de `001` es seguro (pocas filas). No hay datos de usuario en producción todavía.
