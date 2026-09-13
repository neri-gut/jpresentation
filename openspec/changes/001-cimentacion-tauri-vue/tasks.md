# Tasks — 001 Cimentación

## 1. Scaffold
- [ ] 1.1 Crear proyecto con `npm create tauri-app@latest jpresentation -- --template vue-ts`
- [ ] 1.2 Fijar `tauri` 2.11.x, `@tauri-apps/api` acorde, identifier `org.jpresentation.app`
- [ ] 1.3 Configurar Vite + path aliases `@/` y TypeScript `strict`
- [ ] 1.4 Añadir Pinia, Vue Router (operator / audience / speaker), vue-i18n
- [ ] 1.5 Registry vacío de feature modules Vue

## 2. Backend mínimo
- [ ] 2.1 Crates/módulos `domain` + `AppState` + `rusqlite` WAL en `app_data_dir/jpresentation.db`
- [ ] 2.2 Migración v1: profiles, settings, meeting_programs
- [ ] 2.3 Traits vacíos: `MediaProvider`, `OutputPort`, `MeetingClock`, `ProfileStore`, `PlatformSurface`
- [ ] 2.4 Comandos: `profile_list|create|select`, `settings_get|set`, `monitors_list`
- [ ] 2.5 Crear/posicionar superficies audience y speaker vía `PlatformSurface`
- [ ] 2.6 Capabilities operator/audience/speaker + CSP (sin micrófono)

## 3. Shell de operador
- [ ] 3.1 OperatorShell: menú Idiomas / Herramientas / Configuración
- [ ] 3.2 Pestañas Canciones, Cronómetro, Multimedia, Biblia, Navegador Web, Texto (placeholders registrados)
- [ ] 3.3 Panel derecho colapsable: Canciones, Multimedia, Biblia, Cronómetro — **sin Grabador**
- [ ] 3.4 Tema denso, contraste, tipografía de sistema + fallback

## 4. Superficies de salida
- [ ] 4.1 Audience: fondo negro, escucha `output://changed`, placeholder “JPresentation”
- [ ] 4.2 Speaker: negro + HUD compacto placeholder; luego espejo del auditorio (`005`)
- [ ] 4.3 Persistencia de monitor por perfil

## 5. Calidad
- [ ] 5.1 README de desarrollo (Rust + Node + WebView)
- [ ] 5.2 `tauri dev` documentado
- [ ] 5.3 Checklist manual Windows/macOS/Linux de arranque a 2 monitores
