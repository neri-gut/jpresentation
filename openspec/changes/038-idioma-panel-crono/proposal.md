# Proposal: Idioma del perfil, cronómetro sin cánticos, lista solo a la derecha

## Intent

El menú **Idiomas** de la barra y Configuración se pisan: el operador cambia la UI y la guía sigue bajando en `content_locale` (a menudo `E`), con títulos de plantilla en inglés.

Única fuente: **Configuración** (idioma de la app + idioma de contenido). El cronómetro MUST pedir el `mwb`/`w` con el `content_locale` del perfil y mostrar los nombres de sección en esa lengua.

Los cánticos no se usan en el cronómetro (van a Canciones). La pestaña Cronómetro no MUST duplicar reloj ni lista: eso vive en el panel derecho.

## Scope

In scope:

- Quitar el menú Idiomas. `ui_locale` y `content_locale` solo en Configuración.
- Plantilla de sistema / fallback: títulos según `content_locale` (`S` → español, resto inglés de momento). Fetch ya usa `langwritten`.
- Sin filas `song` en el cronómetro. `sjjm` queda en `MeetingWeek.media`.
- Pestaña Cronómetro: guía, plantillas, tabla editable. Reloj + lista operativa solo en el panel.

Out of scope: AES, himnario MP4, unificar `ui_locale` con `content_locale`, traducir plantillas a las 21 lenguas.
