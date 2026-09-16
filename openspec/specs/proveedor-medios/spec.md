# Proveedor de medios

## Purpose

Obtener publicaciones de la semana, parsear el programa, extraer medios embebidos y resolver los que solo vienen como referencia (`jwpub-media://` o clave de catálogo). El frontend no llama URLs de contenido.

## Requirements

### Requirement: Tres puertos, no un monolito
El origen JW SHALL descomponerse en:

- `PublicationCatalog` — qué archivo de publicación corresponde a idioma + símbolo + periodo (`mwb`, `w`, y otros que el programa cite)
- `ScheduleParser` — texto, tiempos y canciones a partir de `.jwpub` / `.epub`
- `MediaResolver` — de un `MediaRef` a bytes en cache

`MediaProvider` orquesta esos tres. Sustituir el catálogo o el parser MUST NOT reescribir la proyección. El frontend MUST NOT llamar URLs de contenido. Host y path de GETPUBMEDIALINKS MUST vivir solo en el adaptador. `mwb` MAY reintentar una vez el mes anterior si el issue del lunes 404 (bimensual); `w` no. Checksum en disco = no repetir el GET del binario.

#### Scenario: Listar la semana
- GIVEN un perfil con idioma de contenido y provider activo
- WHEN el operador abre “Esta Semana”
- THEN obtiene un `MeetingWeek` tipado (partes, tiempos, canciones, medios con estado de cache)
- AND la UI no conoce HTTP ni el esquema `jwpub-media://`

#### Scenario: Solo carpeta local
- GIVEN un provider local sin red
- WHEN el operador abre un `.jwpub` ya copiado a disco
- THEN el parser corre sobre ese archivo
- AND los `embedded` se extraen del paquete
- AND los `jwpub-media` quedan pendientes si no hay resolver de red

### Requirement: Consulta semanal a demanda
El operador SHALL disparar “actualizar esta semana” / “próxima”. El sistema MUST NOT descargar todos los meses al arrancar. Publicaciones mínimas: `mwb` y `w` del periodo que cubre esa semana.

#### Scenario: Primera vez en el mes
- GIVEN cache vacía y red disponible
- WHEN pide Esta Semana
- THEN se localiza y descarga la publicación del mes (si falta)
- AND se parsea
- AND se listan partes + estado de cada medio

### Requirement: Paquete ilegible (JWPUB cifrado o dañado)
Si el `.jwpub` no es un ZIP legible, el SQLite interno no abre (cabecera distinta de `SQLite format 3`, cifrado, o corrupto) o el parser falla, el sistema SHALL:

1. Mostrar un error claro: no se puede leer esta publicación  
2. Ofrecer EPUB del mismo `pub`+`issue` si el catálogo lo trae  
3. Ofrecer abrir un archivo local  
4. Dejar usar cronómetro en plantilla y el himnario ya descargado  

MUST NOT intentar descifrar, guardar claves ni bloquear la consola. El auditorio no cambia. El log interno MAY decir `unreadable_package`; la UI no explica el cifrado.

La muestra `mwb_S_202609` venía en claro; este requisito cubre el caso contrario.

#### Scenario: JWPUB que no abre
- GIVEN el `mwb` del mes descargado pero el parser no lee la base
- WHEN el operador pide Esta Semana
- THEN ve el error y las acciones EPUB / archivo local / continuar sin guía
- AND no se cierra la app

### Requirement: Contrato del parser
`ScheduleParser` SHALL aceptar bytes o ruta de `.jwpub` / `.epub` y devolver semanas estructuradas. La primera implementación MAY alinearse al contrato público de `meeting-schedules-parser` (`loadPub`) y mapear a `MeetingWeek`. No se copia el paquete npm al repo; se usa como dependencia del adaptador o se reimplementa en Rust.

### Requirement: Clave de catálogo (y URI `jwpub-media://`)
La fuente canónica en el `.jwpub` SHALL ser la fila `Multimedia`: si `FilePath` tiene valor, el medio es embebido; si está vacío, el medio es `MediaRef.catalog` con `(key_symbol, track, lang_meps, issue_tag, mime)`.

En la muestra `mwb_S_202609`: cánticos = `sjjm` + número; vídeos de la guía = `mwbv` + track + issue del mes. Una URI `jwpub-media://`, si aparece, se normaliza a esa misma clave. El adaptador consulta GETPUBMEDIALINKS (`langwritten`, `pub`, `issue` YYYYMM, `track`, `output=json`) y toma `files[lang][JWPUB|MP4|MP3].file.url` + checksum. El label MP4 lo fija `video_quality` del perfil (himnario y guía iguales). `20260900` se envía como `202609`. Cánticos usan `sjjm`, no `sjj`. Host y path MUST NOT estar en Vue.

#### Scenario: Cántico 1
- GIVEN `Multimedia.KeySymbol = sjjm` y `Track = 1`
- WHEN el programa enlaza esa ranura
- THEN no se copia a la cache semanal
- AND Play usa `HymnalLibrary` (`hymnal/{lang}/1`)
- AND si falta el archivo, se descarga al himnario permanente, no a `week/`

#### Scenario: Vídeo de la guía
- GIVEN `KeySymbol = mwbv`, `Track = 1`, `IssueTagNumber = 20260900`
- WHEN se descarga
- THEN la clave incluye el issue
- AND no se confunde con el cántico 1


### Requirement: Lo que no viene en la publicación
Imágenes de lección suelen viajar dentro del `.jwpub`. Vídeos y canciones oficiales suelen no viajar. El sistema SHALL:

1. Extraer y cachear todo archivo de medio presente en el paquete
2. Recolectar referencias sin archivo
3. Resolverlas por catálogo de medios
4. Descargar solo esas piezas, con progreso y cancelación

#### Scenario: Lote de la semana
- GIVEN el programa parseado con 3 imágenes embebidas y 2 vídeos externos
- WHEN pulsa descargar medios de la semana
- THEN las 3 imágenes se copian del paquete sin HTTP
- AND los 2 vídeos se descargan
- AND el progreso distingue ambos orígenes

### Requirement: Catálogo y protocolos fuera de la spec
Endpoints, tokens, cabeceras y detalles del contenedor JWPUB MUST vivir solo en el crate adaptador. Si el origen cambia, se actualiza el adaptador. La app MUST degradar a cache + archivo local.

### Requirement: Uso lícito
Descargas para proyección local de la reunión. MUST NOT haber reupload, torrent ni “publicar paquete”. MUST NOT guardar cuentas personales ni automatizar login. Contenido con copyright no se commitea al repo.

### Requirement: Concurrencia adaptable
Paralelismo de descargas según CPU y red del salón. Equipo modesto: una a la vez. Potente: techo bajo (2–3) para no saturar el Wi‑Fi.
