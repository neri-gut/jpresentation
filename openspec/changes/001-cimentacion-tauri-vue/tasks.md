# Tasks — 001 Cimentación

## 1. Scaffold
- [x] 1.1 Crear proyecto con `npm create tauri-app@latest jpresentation -- --template vue-ts`
- [x] 1.2 Fijar `tauri` 2.11.x, `@tauri-apps/api` acorde, identifier `org.jpresentation.app`
- [x] 1.3 Configurar Vite + path aliases `@/` y TypeScript `strict`
- [x] 1.4 Añadir Pinia, Vue Router (operator / audience / speaker), vue-i18n
- [x] 1.5 Registry vacío de feature modules Vue

## 2. Backend mínimo
- [x] 2.1 Crates/módulos `domain` + `AppState` + `rusqlite` WAL en `app_data_dir/jpresentation.db`
- [x] 2.2 Migración v1: profiles, settings, meeting_programs
- [x] 2.3 Traits vacíos: `MediaProvider`, `OutputPort`, `MeetingClock`, `ProfileStore`, `PlatformSurface`
- [x] 2.4 Comandos: `profile_list|create|select`, `settings_get|set`, `monitors_list`
- [x] 2.5 Crear/posicionar superficies audience y speaker vía `PlatformSurface`
- [x] 2.6 Capabilities operator/audience/speaker + CSP (sin micrófono)

## 3. Shell de operador
- [x] 3.1 OperatorShell: menú Idiomas / Herramientas / Configuración
- [x] 3.2 Pestañas Canciones, Cronómetro, Multimedia, Biblia, Navegador Web, Texto (placeholders registrados)
- [x] 3.3 Panel derecho colapsable: Canciones, Multimedia, Biblia, Cronómetro — **sin Grabador**
- [x] 3.4 Tema denso, contraste, tipografía de sistema + fallback

## 4. Superficies de salida
- [x] 4.1 Audience: fondo negro, escucha `output://changed`, placeholder “JPresentation”
- [x] 4.2 Speaker: negro + HUD compacto placeholder; luego espejo del auditorio (`005`)
- [x] 4.3 Persistencia de monitor por perfil

## 5. Calidad
- [x] 5.1 README de desarrollo (Rust + Node + WebView)
- [x] 5.2 `tauri dev` documentado
- [x] 5.3 Checklist manual Windows/macOS/Linux de arranque a 2 monitores
