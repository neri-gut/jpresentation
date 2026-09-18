# Proposal: Himnario permanente (sjjm 1–163), vista de Canciones y audio/vídeo

## Intent

La pestaña **Canciones** es actualmente un placeholder. Los cánticos de reunión pertenecen al himnario permanente (`sjjm`, pistas **1–163** en MP4 con música), independientes del ciclo semanal de la guía (`week/`). 

El operador necesita:
1. Listar las canciones disponibles para el `content_locale` del perfil activo con títulos oficiales del API.
2. Descargar canciones individuales a demanda para pruebas y uso puntual, o descargar/actualizar el himnario completo en segundo plano con control de progreso y cancelación.
3. Asignar y reproducir rápidamente las tres ranuras de la reunión (Canción Inicio, Central y Final, inspiradas en `JMulti-01.png`), pre-cargadas desde la reunión de la semana o seleccionables mediante un selector numérico.
4. Enviar el cántico al Stage (auditorio con audio y vídeo; orador en espejo o HUD).
5. Controlar la reproducción del cántico activo desde el bloque Canciones del panel lateral derecho.

## Scope

In scope:

- **Catálogo y almacenamiento**:
  - Consulta al catálogo público `GETPUBMEDIALINKS` para `pub=sjjm` en el idioma de contenido (`langwritten`).
  - Filtrado estricto a pistas 1–163 (excluyendo audiodescripciones ~600–663 y huecos inexistentes).
  - Almacén persistente en `media/hymnal/{langwritten}/sjjm_{track}_{label}.mp4` y caché de catálogo `media/hymnal/{langwritten}/catalog.json`.
  - Descarga individual a demanda por pista y descarga por lotes en segundo plano con eventos de progreso (`hymnal://progress`) y soporte de cancelación atómica (`hymnal_cancel`).
- **Vista Canciones (`SongsView.vue`)**:
  - Diseño fiel al flujo de `JMulti-01.png` adaptado a los tokens de diseño densos del proyecto:
    - Lista izquierda con pistas (número, título, duración formateada `MM:SS`, estado descargado/pendiente y acción de descarga/play).
    - Buscador rápido por número o título.
    - Bloque derecho con las tres ranuras de reunión: **Canción Inicio**, **Canción Central** y **Canción Final** (spinner numérico + botón de título/play).
    - Botón inferior «⬇ Descargar / Actualizar Himnario» con indicador de progreso y botón Cancelar.
- **Stage y Panel Lateral**:
  - Comando para reproducir cántico en Stage (`output.open_media` con `StageKind::Video`).
  - Bloque Canciones en `RightPanel.vue` mostrando el cántico activo/cued y botón de Play / Detener.
- **Pruebas**:
  - Tests unitarios en Rust con catálogo simulado en memoria (`MemoryCatalog`), verificación de MD5, filtrado de pistas 1–163 y descarga individual / masiva.

Out of scope:

- Formato RTF/`sjj` (solo se usa MP4 de reunión).
- Grabador / micrófono (prohibido por arquitectura).
- Mute de emergencia o bus de precucha en consola (audio sale únicamente por el auditorio).
- Descarga obligatoria de los 163 vídeos al iniciar la aplicación.

## Approach

1. `domain::hymnal` define los DTOs (`HymnalSongDto`, `HymnalSlotsDto`, `HymnalProgressDto`, `SongStatus`) y lógica de filtrado/rutas.
2. `hymnal_service` orquesta la caché local, consulta al catálogo CDN, descargas individuales y por lotes con cálculo de checksums.
3. `commands::hymnal` expone los comandos Tauri seguros con DTOs de ≤4 parámetros y `Result<T, AppErrorDto>`.
4. En Vue, `stores/hymnal.ts` gestiona el estado reactivo, hidratación, progreso de descargas y comunicación con `weekStore` para las ranuras iniciales.
