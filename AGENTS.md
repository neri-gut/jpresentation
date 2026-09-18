# JPresentation — instrucciones para el agente

Eres un experto en Tauri 2, Vue 3, TypeScript, Rust y SQLite. UX densa y segura. SDD/OpenSpec. No improvises features.

## Qué es esto

Reescritura multiplataforma de JMultimedia 2.8.01 (Windows/.NET).  
Nombre: **JPresentation**. Id: `org.jpresentation.app`.  
No oficial. No afiliado a Watch Tower. Complemento de JW Library.

Stack: **Tauri 2.11.x estable** (no 3 alpha) + Vue 3 + Vite + Pinia + TS + Rust + SQLite WAL.  
`tauri` 2.11.x ≠ `tauri-build` (2.6.x en 001). No igualar el parche.

## Fuente de verdad

1. `openspec/INDEX.md` — mapa  
2. `openspec/specs/**/spec.md` — comportamiento  
3. `openspec/changes/<id>/` — un change a la vez  
4. Capturas `JMulti-01.png` … `JMulti-06.png` — flujo visual (sin Grabador)

No implementes nada que no esté en un change abierto. El change actual de código es `039-himnario-canciones`. `001` y `029`–`038` están cerrados.

## Change 039 (hacer ahora)

Himnario permanente (`sjjm` 1–163), vista de Canciones (estilo JMulti-01), descargas individuales y por lotes, ranuras de reunión, proyección en Stage y bloque en panel lateral.

## Prohibido

- Grabador, micrófono, carpeta Record, CCTV, cámara virtual  
- Scrapers / login automático a sitios oficiales  
- Himnario, Biblia o vídeos en el git  
- `unwrap` en caminos de reunión  
- SQL o fs desde Vue  
- Tauri 3 alpha  

## Capas

Controller = comando Tauri + composable `invoke`.  
Service = crate de dominio.  
Repo = SQLite/fs/HTTP solo Rust.  
DTOs en cada frontera. ≤4 parámetros. rustdoc/TSDoc en API pública.  
`Result<T, AppErrorDto>`. El auditorio no se cae si falla la consola.

## Superficies

| Rol | Privilegio |
|-----|------------|
| Operador | fs acotado, dialog, http de catálogo, ventanas |
| Auditorio | render (+ webview kiosco si hay página) |
| Orador | solo render; espejo o HUD; sin input |

## Recordatorios de producto

- UI default/fallback: `en`. Contenido: `langwritten` JW.  
- Himnario `sjjm` 1–163 permanente. Guía `mwb`/`w` semanal.  
- Cronómetro arma la siguiente parte; Iniciar la cuenta; no lanza medios.  
- Texto diario = reposo del auditorio; el orador no lo ve.  
- Un decoder de stage. Tema solo en consola.  
- Updater: `tauri-plugin-updater` + GitHub Releases (después de 001).

## Cómo trabajar

1. Lee `openspec/INDEX.md` y el `proposal.md` del change.  
2. Implementa solo ese alcance.  
3. Pruebas de dominio sin WebView.  
4. `npm ci` / `cargo --locked`. No commitees `node_modules`, `target`, `.jwpub` de usuario.
