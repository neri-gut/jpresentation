# Seguridad y capacidades

## Purpose

Aplicar el modelo de permisos de Tauri 2: cada ventana tiene capabilities distintas, CSP estricto, sin Node en el frontend, y Rust como único puente a SO y SQLite.

## Requirements

### Requirement: Cadena de suministro
Ver `specs/cadena-suministro`. Lockfiles, `npm ci` / `cargo --locked`, audit en CI.

### Requirement: Least privilege
La ventana operador MAY usar fs (scope al data dir del perfil y cache), dialog, os, window, clipboard, global-shortcut e http solo hacia los orígenes que el MediaProvider declare (más el actualizador de la app). MUST NOT pedir permiso de micrófono. La superficie auditorio MUST limitarse a eventos de render y, si navega, a un webview aislado. La superficie orador MUST ser solo render (espejo de medio o texto paginado + HUD + mensaje). Sin input de paginación, sin fs, http de contenidos ni SQL.

### Requirement: CSP y IPC
El frontend SHALL usar CSP que impida scripts remotos en la consola. Todo IPC va por comandos nombrados. No se evalúa JavaScript arbitrario desde contenido de usuario.

### Requirement: Actualizaciones
Ver `specs/licencia-updater`: `tauri-plugin-updater` + GitHub Releases + firma. Opcional. Sin medios en el paquete.

### Requirement: Datos locales
SQLite y cache de medios en el directorio de datos del usuario. No hay cuenta en la nube en v1. Logs rotados, sin rutas de usuario enviadas a terceros. El sistema MUST NOT guardar contraseñas de servicios oficiales de contenido.
