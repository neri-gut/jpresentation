# Proposal: Reabrir stage y árbol de carpetas locales

## Intent

Tras `033`, Mostrar la primera imagen funciona; al Cerrar y elegir otra, el stage no cambia hasta desactivar el orador. Causa: se pisa `media/stage/current.jpg` (misma URL, webview cacheado y fichero retenido).

El árbol de Multimedia solo lista Esta/Próxima semana. Sin Escritorio/Home ni subcarpetas, no se puede buscar un jpg local.

El degradado del cronómetro **no** entra en este change.

## Scope

In scope:

- Cada `stage_open` escribe un fichero **nuevo** (`{rev}.ext`). Cerrar emite `none` y borra el anterior. Vue remonta img/vídeo con `:key=rev`.
- Árbol: Home, Desktop (si existe), Pictures/Videos si existen, raíces guardadas. Subcarpetas expandibles. Diálogo nativo para añadir carpeta.
- Jail: `media_root` + raíces por defecto + raíces del perfil.

Out of scope: cronómetro/degradado, PDF, himnario, seek, Record.
