# Design

## Idioma

`week_fetch` ya pasa `profile.content_locale`. El hueco es la plantilla hardcodeada en inglés y el menú Idiomas (`ui_locale`) que no mueve el catálogo.

Configuración gana el selector de UI (clave `settings.uiLanguage` ya existe). El menú de la barra desaparece.

`skeleton_parts(kind, prefix, langwritten)`: `S` usa los nombres de la spec (Tesoros, Perlas, …). Otros códigos: inglés.

## Sin cánticos

`parts_from_html` ignora Song/Canción. `ensure_outline` y las plantillas de sistema no insertan `tone=song`. Los tracks `sjjm` permanecen en `media`.

## Panel

`TimerView` deja de montar `TimerControls`. El panel derecho sigue siendo el HUD operativo (lista sin cánticos, Iniciar/Pausar/Terminar).
