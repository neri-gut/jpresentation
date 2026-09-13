# Design: Cimentación Tauri 2 + Vue 3

## Technical Approach

Aplicación Tauri 2 con tres `WebviewWindow`:

| Label        | Decorada | Capability           | Frontend route   |
|--------------|----------|----------------------|------------------|
| operator     | sí       | operator-cap         | /                |
| audience     | no       | audience-cap         | /audience        |
| speaker      | no       | speaker-cap          | /speaker         |

El backend mantiene `AppState { db, output: OutputState, providers: ProviderRegistry }`. Sin estado de grabador. Vue-operator llama `invoke`; Rust emite `output://changed` a audience y `timer://changed` a speaker. Los comandos delegan a puertos de dominio, no a un `lib.rs` monolítico.

## Architecture Decisions

### Decision: Tauri 2.11 estable, no 3.0 alpha
v3 está en alpha el 2026-09-12. Una app de salón necesita WebView y plugins maduros (fs, dialog, window-state, sql/rusqlite). Se revisará v3 cuando haya estable.

### Decision: rusqlite en Rust, no plugin-sql desde el frontend
El frontend no escribe SQL. Evita inyección y mantiene migraciones en un solo lugar (`src-tauri/src/db/migrations`).

### Decision: Pinia + eventos, no estado Vue compartido entre ventanas
Cada webview es un proceso de render distinto. El source of truth del auditorio es Rust.

### Decision: Reproducción futura por capa
v1 de medios usará `<video>`/`<audio>` del WebView del auditorio con URLs `asset:` o `convertFileSrc` para archivos locales. Si un SO no decodifica bien, un change posterior podrá añadir sidecar mpv. No se ata VLC en la cimentación.

### Decision: UI moderna pero densa
Se abandona el look WinForms, no el flujo. Densidad alta, tipografía clara, contraste AA, atajos. Tema claro/oscuro. Panel derecho fijo. Pestañas = Vue components lazy.

### Decision: Pool de trabajo
`tokio` + `num_cpus` con techo `max(2, cores-1)` para jobs de índice/thumbnails/descargas (changes posteriores). La cimentación solo deja el `JobQueue` vacío.

### Decision: Sin grabador
No hay crate, comando, capability ni widget de captura de audio.

### Decision: PlatformSurface desde el día uno
Crear/mover ventanas pasa por un trait `PlatformSurface`. En escritorio usa `WebviewWindow`. Móvil se añadirá como otro adaptador.

## Data model (mínimo v0)

```
profiles(id, name, ui_locale, content_locale, media_provider, created_at, updated_at)
settings(profile_id, key, value_json)
meeting_programs(id, profile_id, date, kind, payload_json)
schema_migrations(version, applied_at)
```

## Capabilities (borrador)

`operator-cap`: core:event, core:window, dialog, fs scope `$APPLOCALDATA/**` + carpetas que el usuario autorice, opener.

`audience-cap` / `speaker-cap`: core:event default, sin fs write.

## File map previsto

```
jpresentation/
  src/                 # Vue
    views/{Songs,Timer,Media,Bible,Browser,Text}.vue
    layout/OperatorShell.vue
    windows/{AudienceApp,SpeakerApp}.vue
    stores/{profile,output,timer}.ts
  src-tauri/
    src/{lib.rs,commands/,db/,platform/,domain/{media,output,clock,profile}/,jobs.rs}
    capabilities/{operator,audience,speaker}.json
    tauri.conf.json
```

## Risks

- WebView de Linux (WebKitGTK) varía por distro: probar Ubuntu LTS y Fedora.
- macOS retiene fullscreen por espacio: usar fullscreen borderless sobre el monitor elegido.
- El catálogo JW puede cambiar de forma: el adaptador debe fallar con mensaje claro y no romper la reunión si hay cache.
