# Dev container

## Purpose

Un entorno reproducible para quien clone el repo: Node, Rust, dependencias de WebView/Tauri en Linux, extensiones del editor. Sin “funciona en mi máquina”.

## Requirements

### Requirement: Contenedor oficial
El repo SHALL incluir `.devcontainer/devcontainer.json` (+ Dockerfile o image documentada) con:

- Node LTS alineado al scaffold (20 o 22)
- Rust stable + `rust-analyzer`
- paquetes de build Tauri en Debian/Ubuntu (`libwebkit2gtk`, `libayatana-appindicator`, `librsvg`, `patchelf`, clang, pkg-config)
- `npm ci` opcional al create
- extensiones: Vue (Volar), rust-analyzer, ESLint, CodeLLDB
- `forwardPorts` del Vite/Tauri de desarrollo
- usuario no root

MUST funcionar en VS Code / Cursor Dev Containers y en GitHub Codespaces a nivel de **compilar el frontend y el crate** (el WebView de GUI puede no existir en Codespaces: ahí `cargo test` + `npm run build` bastan; el run nativo se documenta como “local o contenedor con /dev/dri”).

### Requirement: Sin secretos en la imagen
No API keys. El contenedor no descarga himnario ni Biblias. Volumen del workspace = el clone.

### Requirement: Documentación
`CONTRIBUTING` (cuando exista código) apunta al devcontainer como camino preferido. El change `001` de scaffold MUST crear este directorio, no dejarlo para “luego”.
