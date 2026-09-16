# Design

## Imagen

WebKitGTK ignora `object-fit` si `width` y `height` son `100%`. `min-width/min-height: 100%` es `cover` y recorta.

Medir `naturalWidth/Height` vs caja y aplicar `scale = min(cw/nw, ch/nh)` en píxeles. Proporción igual → llena. Cuadrada en 16:9 → franjas laterales. ResizeObserver al contenedor.

## Guía sin HTML

`Document.Content` cifrado → `parts_from_html` vacío. MUST NOT AES.

Tras parsear SQLite, si `parts` está vacío: plantilla sistema (`midweek` / `weekend`) + overlay de tracks `sjjm` en orden de párrafo. Medios sobrantes siguen en `MeetingWeek.media`.

## Plantillas

Sistema (código, id `sys:*`): entre semana, fin de semana, circuito entre semana, circuito fin de semana. No se borran.

Usuario: tabla `event_templates` (v3), JSON de filas `{title, minutes, tone}`. Duplicar/guardar desde la tabla actual.

Aplicar circuito entre semana: conservar hasta vida cristiana, quitar CBS, añadir recapitulación + presentación + discurso 30 min + cántico final.

Restaurar: reparsear `week/.../pub/*.jwpub` en disco (sin catálogo si el archivo está).
