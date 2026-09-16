# JPresentation

Aplicación de escritorio para operar reuniones (canciones, multimedia, cronómetro, Biblia, textos) con salida a auditorio y monitor de orador.

No oficial. No afiliada a Watch Tower. Complemento de JW Library.

Stack: **Tauri 2.11** + Vue 3 + TypeScript + Rust + SQLite WAL.

Las cajas `tauri` y `tauri-build` **no comparten número de parche**. Runtime: `tauri` 2.11.x (y `@tauri-apps/api` acorde). Generador de build: `tauri-build` 2.6.x. El CLI npm (`@tauri-apps/cli`) puede ir un parche por delante del crate. No forzar `tauri-build = 2.11`. Prohibido Tauri 3 alpha.

Desarrollo dirigido por specs: [`openspec/INDEX.md`](openspec/INDEX.md). Change de código actual: [`034-stage-reopen-arbol-local`](openspec/changes/034-stage-reopen-arbol-local/). `001` y `029`–`033` cerrados.

## Requisitos

| Herramienta | Notas |
|-------------|--------|
| Node 20 o 22 | LTS. `engines.node >= 20` |
| Rust stable | `rustc` 1.77.2+ (Tauri 2.11) |
| WebView del SO | Windows: WebView2. macOS: WKWebView. Linux: WebKitGTK 4.1 (`libwebkit2gtk-4.1-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `patchelf`, `clang`, `pkg-config`) |

Entorno reproducible: [devcontainer](.devcontainer/devcontainer.json). Ver [CONTRIBUTING.md](CONTRIBUTING.md).

## Arranque

```bash
npm ci
npm run tauri dev
```

`npm run tauri dev` levanta Vite en `http://localhost:1420` y abre la ventana **Operador**. Rust crea la superficie **Auditorio** (fullscreen en el segundo monitor, o ventana de previsualización si solo hay uno). El **Orador** se crea cuando el perfil tiene «Usar monitor de orador».

Pruebas de dominio (sin WebView):

```bash
cargo test --manifest-path src-tauri/Cargo.toml --locked
```

Empaquetado de frontend:

```bash
npm run build
```

Lockfiles canónicos: `package-lock.json` y `src-tauri/Cargo.lock`. CI usa `npm ci --ignore-scripts` y `cargo --locked`.

## Checklist manual (2 monitores)

Tras `npm run tauri dev`:

1. **Windows / macOS / Linux** — la consola se titula JPresentation y muestra menú Idiomas / Herramientas / Configuración.
2. Pestañas: Canciones, Cronómetro, Multimedia, Biblia, Navegador Web, Texto (placeholders). El panel derecho tiene esos cuatro bloques y **no** hay Grabador.
3. **Herramientas → Acerca de**: nombre JPresentation, stack Tauri/Vue/Rust y aviso de no afiliación.
4. Con escritorio extendido, asignar Auditorio a la pantalla secundaria en Configuración: esa pantalla pasa a negro con el texto «JPresentation»; la consola permanece en la primaria. Si Auditorio queda en «ventana de previsualización», **no** se cubre sola la secundaria.
5. Con un solo monitor (o preview explícito), auditorio y orador abren como ventanas flotantes 960×540 junto a la consola, no a pantalla completa encima de ella. El HUD del orador, sin medio, ocupa toda su ventana.
6. Cerrar la consola cierra auditorio y orador. Cerrar y reabrir conserva el perfil (UI `en` en perfiles nuevos) y la asignación de monitores.
7. Un fallo en un comando de la consola muestra un aviso en el operador; el auditorio no se cierra.
8. **Configuración → idioma de contenido**: el selector lista la semilla (p. ej. `S`, `TG`); no hay petición de red. El menú Idiomas sigue cambiando solo la UI.
9. Horario de las dos reuniones (weekday + hora) se guarda por perfil y sobrevive un reinicio.
10. Acento (blue/teal/violet/amber) y densidad cambian la consola en caliente; el auditorio permanece negro.
11. **Identificar** muestra nombre y resolución ~2 s en cada pantalla y no mueve auditorio ni orador.
12. Borrar el único perfil se rechaza (`LastProfile`). Duplicar copia settings y no selecciona la copia.
13. Panel Cronómetro: armar una parte (título + minutos), Iniciar / Pausar / Terminar. El orador muestra el restante a pantalla completa y la barra; a 0:00 el desfase sale en rojo. El auditorio no pinta el reloj.
14. Configuración → Orador: modo espejo+HUD o solo HUD. Los desplegables marcan consola y primaria; preview no implica un solo monitor.
15. **Cronómetro → Obtener guía**: baja `mwb`/`w` y lista partes con minutos; pulsar una fila arma el reloj. **Multimedia** es un explorador de archivos (cache de la semana, carpetas locales, `.jwpub`); Abrir envía imagen o vídeo al auditorio y al orador.

## Fuera de alcance (v1 / change 034)

Grabador de audio, CCTV, precucha, himnario/Biblia en el git, parser JWPUB, catálogo HTTP, asistente de 9 pasos, Tauri 3 alpha.
