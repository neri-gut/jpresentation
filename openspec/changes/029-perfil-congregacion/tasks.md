# Tasks — 029 Perfil de congregación

## 1. Datos
- [x] 1.1 Migración v2: recrear `settings` y `meeting_programs` con `ON DELETE CASCADE`; test de cascade + seed `Default`
- [x] 1.2 Semilla `domain/content_languages.json` (21 códigos de `research/idiomas.md`) + `ContentLanguageDto`
- [x] 1.3 `MeetingScheduleSetting` (defecto mar 19:00 / dom 10:00) y clave `meeting_schedule` en `SettingsStore`
- [x] 1.4 Paleta cerrada de `accent` (`blue|teal|violet|amber`) validada al escribir `appearance`; tokens CSS por `data-accent`

## 2. Puerto de perfil
- [x] 2.1 `UpdateProfileDto.content_locale`; validar contra la semilla
- [x] 2.2 `ProfileStore::duplicate` — nuevo id, sufijo ` (copy)`, copia settings
- [x] 2.3 `ProfileStore::delete` — `LastProfile` si es el único; si era el activo, seleccionar el más reciente que queda
- [x] 2.4 Tests de dominio: defaults `en`/`E`, duplicate, last profile, locale desconocido, schedule inválido

## 3. Comandos y permisos
- [x] 3.1 Comandos `profile_duplicate`, `profile_delete`, `content_languages_list`, `monitors_identify`
- [x] 3.2 Extender `profile_update` y registrar los cuatro en `operator-commands`
- [x] 3.3 `AppError::{LastProfile, UnknownContentLanguage, MonitorMissing}` → `AppErrorDto.code`; Vue mapea a `errors.*`

## 4. Superficies
- [x] 4.1 `SpeakerPlacement { Hidden, Preview, Monitor(id) }`; `use_speaker=true` + id vacío/ausente = Preview, no hide
- [x] 4.2 Si el id del orador coincide con el del auditorio o no existe: Preview + toast `MonitorMissing`; no mover el auditorio
- [x] 4.3 `identify_monitors`: ventana `identify-{n}` por pantalla, nombre + resolución, 2 s, destroy; capability `identify-cap`
- [x] 4.4 Test unitario del mapeo `SurfacesSetting → SpeakerPlacement` con fake `PlatformSurface`

## 5. Consola
- [x] 5.1 Configuración / Perfiles: renombrar, duplicar, borrar con diálogo de confirmación
- [x] 5.2 Selector de idioma de contenido filtrable (`name` / `langwritten`); menú Idiomas intacto (solo UI)
- [x] 5.3 Sección Reuniones: weekday ISO + `input type="time"` para entre semana y fin de semana
- [x] 5.4 Tema: swatches de acento + radios de densidad; cambio en caliente, auditorio intacto
- [x] 5.5 Botón Identificar en Monitores
- [x] 5.6 Claves nuevas solo en `en.json` (`settings.*`, `weekday.1`–`7`, `errors.lastProfile|unknownContentLanguage|monitorMissing|confirmDelete`)

## 6. CI
- [x] 6.1 `.github/workflows/ci.yml`: job `frontend` (`npm ci --ignore-scripts`, `npm run build`) y job `rust` (`cargo test --locked` + deps WebKitGTK 4.1)
- [x] 6.2 `cargo audit` en `supply-chain.yml` (warning si no hay token; no tumba el resto)
- [x] 6.3 README / CONTRIBUTING: el change de código actual es `029`; el checklist manual incluye contenido, horario, acento, identificar y borrar-el-último

## 7. Documentación SDD
- [x] 7.1 `openspec/INDEX.md` y `AGENTS.md` apuntan a `029` como change de código actual
- [x] 7.2 Deltas aplicados a specs vivas: `perfiles`, `ventanas`, `tema`, `cadena-suministro`
