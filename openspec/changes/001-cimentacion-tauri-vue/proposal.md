# Proposal: Cimentación Tauri 2 + Vue 3 + TypeScript

## Intent

Preparar el esqueleto de JPresentation para migrar JMultimedia 2.8.01 desde Windows/.NET a un binario Rust + Vue multiplataforma. Este cambio no implementa aún canciones ni multimedia reales: deja listo el contenedor, las tres superficies, SQLite, i18n, el registry de features y el cascarón de UI fiel al flujo operativo (sin grabador).

## Scope

In scope:
- Proyecto `create-tauri-app` plantilla `vue-ts` sobre Tauri 2.11.x estable
- Identificador `org.jpresentation.app`, nombre visible JPresentation
- Ventanas Operador / Auditorio / Orador
- Layout de consola: menú, pestañas, panel derecho colapsable
- SQLite WAL + tabla de perfiles y settings
- Pinia stores + eventos Tauri para estado de salida
- i18n es/en
- Capabilities mínimas y CSP
- Documentación SDD ya existente como fuente de verdad

Out of scope (cambios siguientes):
- Reproducción real de audio/vídeo
- Indexado de carpetas
- Grabación (descartada de forma permanente; ver 002)
- Módulo bíblico
- MediaProvider JW
- Navegador embebido con envío al auditorio
- Importadores desde la instalación vieja de JMultimedia

## Approach

1. Congelar stack estable (no Tauri 3 alpha).
2. Rust posee monitores, ventanas, DB y un `OutputState`.
3. Vue solo pinta y despacha comandos.
4. Cada pestaña nace como vista vacía con el contrato de la spec de dominio, lista para el siguiente change.
