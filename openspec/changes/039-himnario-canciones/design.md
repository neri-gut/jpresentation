# Design: Himnario permanente y Canciones

## Catálogo y Adaptador CDN

`pub=sjjm` sin parámetro `track` devuelve la lista completa de archivos en formato `MP4` para `langwritten`.
El adapter agrupa por `track`, filtra `1 <= track <= 163`, descarta audiodescripciones (`track >= 600`) y selecciona la mejor calidad (`720p` > `480p` > `360p` > `240p`).
La respuesta se persiste como índice local en `media/hymnal/{langwritten}/catalog.json` para permitir carga instantánea sin red en posteriores arranques.

## Almacenamiento Permanente

Los vídeos MP4 se almacenan en:
`media/hymnal/{langwritten}/sjjm_{track}_{label}.mp4`

Al listar las canciones, se verifica la existencia física del archivo para determinar el estado:
- `ready`: archivo presente en disco con tamaño válido.
- `pending`: disponible en catálogo pero no descargado.
- `failed`: error de red o fallo de checksum MD5.

## Descarga a Demanda vs Lotes

1. **Descarga individual (`hymnal_download_song`)**: permite descargar una sola pista (útil para pruebas o reuniones específicas sin necesidad de descargar las 163 canciones).
2. **Descarga por lotes (`hymnal_download_all`)**: itera las canciones pendientes, emitiendo `hymnal://progress` (`phase: "download"`, `done: u32`, `total: u32`, `label: String`).
3. **Cancelación (`hymnal_cancel`)**: activa `AppState.hymnal_cancel = true` para interrumpir el ciclo inmediatamente. Lo ya descargado permanece intacto.

## Ranuras de Reunión (Slots)

Las ranuras se modelan con `HymnalSlotsDto`:
- `start: Option<u32>`
- `middle: Option<u32>`
- `end: Option<u32>`

Al cargar la vista, si las ranuras están vacías, se extraen automáticamente de los números de cántico presentes en `MeetingWeek.media` de la reunión seleccionada (`midweek` o `weekend`). El operador puede cambiar el número en cualquier momento con el spinner numérico.

## Reproducción en Stage

`hymnal_play(track)`:
1. Comprueba si el archivo está en disco; si no, lo descarga bajo demanda.
2. Invoca `output.open_media(StageKind::Video, song.title, "video/mp4", path)`.
3. Notifica a las superficies mediante `output://changed`.
4. El auditorio reproduce el vídeo con audio.
