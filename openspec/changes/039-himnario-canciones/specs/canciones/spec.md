# Delta — Canciones (039)

## ADDED Requirements

### Requirement: Lista desde el catálogo del perfil
Al abrir Canciones, el sistema SHALL consultar `GETPUBMEDIALINKS` `pub=sjjm` con el `content_locale` del perfil (o `catalog.json` ya cacheado). Filtra pistas 1–163, elige el `label` MP4 más alto y muestra título y duración del API. MUST NOT usar el idioma de UI.

#### Scenario: Español
- GIVEN `content_locale=S` y red
- WHEN abre Canciones
- THEN hay hasta 163 filas con títulos del API (p. ej. «Las cualidades principales de Jehová»)
- AND no aparecen pistas 600–663

### Requirement: Descarga local a demanda
Cada fila pendiente SHALL poder descargarse sola. El operador MAY marcar varias pendientes y bajar solo esas, o pulsar descarga completa de las pendientes. Los MP4 viven en `media/hymnal/{langwritten}/sjjm_{track}_{label}.mp4`. Lo ya bajado no se vuelve a pedir. Cancelar el lote deja lo escrito.

#### Scenario: Tres cánticos de la semana
- GIVEN 163 pendientes
- WHEN marca 38, 99 y 112 y pulsa descargar seleccionadas
- THEN solo esos tres quedan `ready` en disco
- AND el resto sigue `pending`

#### Scenario: Himnario completo
- GIVEN 80 pendientes
- WHEN pulsa descargar todas
- THEN pide esas 80 en segundo plano con progreso
- AND Cancelar detiene el ciclo; las ya escritas permanecen
