# Canciones

## Purpose

Himnario permanente de reunión (`sjjm`, pistas **1–163** o las que existan en ese rango): vídeo con música. La guía solo enlaza el número. Ambiente va en **otra carpeta**, con su propia lista.

## Requirements

### Requirement: Solo cánticos de reunión
El catálogo SHALL listar `pub=sjjm` y quedarse con tracks **1–163** presentes en el API para ese `langwritten`. MUST NOT incluir pistas de audiodescripción (~600–663) ni huecos: si falta el 87 en un idioma, no se muestra fila vacía.

Título = campo `title` del API (`"151. Jehová los llamará"`). MUST NOT bajar RTF/`sjj`.

Lista: un GET `sjjm` sin `track`, filtrar rango 1–163.

#### Scenario: Español
- GIVEN `langwritten=S` y el API con 1–163 más audiodescripciones
- WHEN abre Canciones
- THEN la lista tiene hasta 163 ítems, sin 600–663
- AND cada fila muestra el título del API

#### Scenario: Idioma incompleto
- GIVEN un idioma con tracks 1–140 y 150–160
- WHEN lista
- THEN solo esas pistas, ordenadas, sin placeholders 141–149

### Requirement: Vídeo con música
La descarga de reunión SHALL ser **MP4**. La calidad es la **del perfil** (`video_quality`), la misma que vídeos de la guía. Ver `perfiles` / `proveedor-medios`.

Opcional, aparte: el operador MAY marcar “también audio MP3” del mismo himnario (sin vídeo) en `hymnal/{lang}/mp3/`. No sustituye el MP4 de reunión.

#### Scenario: Descargar himnario
- GIVEN 80 pistas pendientes y calidad por defecto
- WHEN pulsa Descargar himnario
- THEN pide MP4 720p (o el tope de ese idioma)
- AND progreso + cancelación; lo bajado permanece

### Requirement: Almacén permanente
`hymnal/{langwritten}/{track}_{label}.mp4`. Distinto de `week/` y de `ambient/`. Cambiar de semana no purga. Arrancar la app o abrir Multimedia MUST NOT disparar las 163 descargas.

### Requirement: Actualizar
“Actualizar himnario” vuelve a pedir la lista. Solo descarga tracks **nuevos** o con checksum distinto a la calidad elegida. No reescribe los que ya coinciden.

#### Scenario: Sale el 164
- GIVEN 1–163 en disco
- WHEN el API añade track 164
- THEN la lista muestra 164 pendiente
- AND Actualizar solo baja el 164

### Requirement: Tres ranuras
Inicio / Central / Final = números del `MeetingWeek`. Play = `HymnalLibrary.get(lang, n)` → MP4, o MP3 si no hay vídeo (ver `specs/audio`). Si no hay ninguno, baja el MP4 de la calidad del perfil.

### Requirement: Ambiente en carpeta propia
Ver `specs/audio`. Carpeta `ambient/{profile}/`. Loop o fin de lista. Play manual o reglas de cronómetro. Fade al cántico. MUST NOT mezclar con `hymnal/`.
