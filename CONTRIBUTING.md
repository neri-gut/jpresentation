# Contribuir a JPresentation

Specs primero: `openspec/INDEX.md` y el change abierto (`038-idioma-panel-crono`). No se implementa nada que no esté en un change.

## Entorno preferido

El [devcontainer](.devcontainer/devcontainer.json) instala Node 22, Rust stable y las librerías de WebView de Tauri en Linux. Funciona en VS Code / Cursor Dev Containers y en GitHub Codespaces para `npm run build` y `cargo test`. El `tauri dev` nativo necesita un display (`/dev/dri` o un escritorio local).

## Máquina local

- Node 20 o 22
- Rust stable
- WebView del SO (WebView2 / WKWebView / WebKitGTK 4.1)

Instalación:

```bash
npm ci
```

`package-lock.json` y `Cargo.lock` son canónicos. No uses `npm install` suelto en CI.

### Versiones Tauri (no son el mismo número)

| Pieza | Crate / paquete | Línea en 001 |
|-------|-----------------|--------------|
| Runtime (webview, comandos) | `tauri` | 2.11.x (`=2.11.1`) |
| API JS | `@tauri-apps/api` | 2.11.1 |
| CLI npm | `@tauri-apps/cli` | 2.11.x (puede ser 2.11.4) |
| Codegen de `build.rs` | `tauri-build` | **2.6.x** (`=2.6.3`), no 2.11 |

`tauri-build` es otro crate con semver propio. Subir `tauri` a 2.11 no implica `tauri-build = 2.11`. Nunca Tauri 3 alpha.

## Arranque

```bash
npm run tauri dev
```

El frontend Vite queda en `http://localhost:1420`. Tauri abre la ventana Operador y crea Auditorio (y Orador si el perfil lo tiene activo).

Pruebas de dominio (sin WebView) y empaquetado de frontend (CI de `029`):

```bash
cargo test --manifest-path src-tauri/Cargo.toml --locked
npm run build
```

CI: `.github/workflows/ci.yml` (`npm ci --ignore-scripts` + `npm run build`, `cargo test --locked`) y `supply-chain.yml` (`npm audit`, `cargo audit` con `continue-on-error`).

Si el clone vive en exFAT (sin exec/symlinks), apunta el target a un disco nativo:

```bash
CARGO_TARGET_DIR=/tmp/jpresentation-target cargo test --manifest-path src-tauri/Cargo.toml --locked
```

Frontend:

```bash
npm run build
```
