# Tasks — 032 Guía de la semana y Multimedia

## 1. Dominio
- [x] 1.1 `MeetingWeek` / `MeetingPart` / `MediaRef` + estados `embedded` | `ready` | `pending` | `pending_hymnal` | `failed`
- [x] 1.2 Puertos `PublicationCatalog`, `ScheduleParser`, `MediaResolver`; `MediaProvider` `jw-org`
- [x] 1.3 Cálculo esta/próxima semana + issue `mwb`/`w` (reintento un mes atrás en `mwb`)
- [x] 1.4 Tests puros con catalog/parser fake (sin HTTP)

## 2. Paquete y disco
- [x] 2.1 Lector ZIP anidado + `manifest` + SQLite (`SQLite format 3` o `UnreadablePub`)
- [x] 2.2 Extraer embebidos a `week/{lang}/{yyyy-mm-dd}/`; thumbs en job
- [x] 2.3 Fixture sintético en tests; nada de pubs oficiales en git
- [x] 2.4 Persistencia `meeting_programs` (perfil + lunes + kind)

## 3. Catálogo HTTP
- [x] 3.1 Adaptador GETPUBMEDIALINKS (`reqwest`) + checksum; host no sale a Vue
- [x] 3.2 `week_fetch` / `week_download_media` / cancel + evento de progreso
- [x] 3.3 `sjjm` no se descarga a `week/`

## 4. Vue
- [x] 4.1 Pestaña Multimedia: árbol esta/próxima, estados, preview de imagen
- [x] 4.2 Toolbar Actualizar / Descargar pendientes; toasts `UnreadablePub` / red / 404
- [x] 4.3 Claves nuevas solo en `en.json`

## 5. Docs
- [x] 5.1 INDEX / AGENTS / README / CONTRIBUTING apuntan a `032`; `031` cerrado
- [x] 5.2 Deltas `programa-semanal`, `multimedia`, `proveedor-medios`, `cache`
