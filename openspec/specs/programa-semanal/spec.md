# Programa semanal

## Purpose

Construir el programa de la reunión (entre semana y fin de semana) a partir de las publicaciones oficiales de la semana: Vida y Ministerio (símbolo `mwb`) y artículo de estudio de La Atalaya (símbolo `w`). Extraer nombre de cada sección, tiempo, canciones e imágenes/vídeos relacionados. Los binarios que no viajen dentro de la publicación se resuelven y descargan aparte.

Referencia de forma del programa: el modelo público de `meeting-schedules-parser` (sws2apps), no su código copiado.

## Requirements

### Requirement: Semana como unidad
El operador SHALL poder pedir “esta semana” y “próxima semana” en el idioma de contenido del perfil. El sistema SHALL identificar las publicaciones de esa semana (`mwb` del mes, `w` del mes) y producir un `MeetingWeek` con partes ordenadas.

#### Scenario: Entre semana
- GIVEN idioma ES y una fecha de lunes
- WHEN el operador abre Esta Semana
- THEN ve partes equivalentes a: canción inicial, tesoro, gemas, lectura, asignaciones AYF (1–n), canción central, vida cristiana (1–n), estudio bíblico de congregación, canción final
- AND cada parte tiene título, minutos cuando el origen los trae, y una lista de medios (puede estar vacía)

#### Scenario: Fin de semana
- GIVEN el artículo de estudio de esa semana
- WHEN se carga el programa de Atalaya
- THEN aparecen canción de apertura, título del artículo, canción de cierre
- AND los medios del artículo quedan asociados a esa parte

### Requirement: Parser enchufable
La extracción de texto/tiempos/canciones SHALL pasar por `ScheduleParser`. La primera implementación MAY apoyarse en el contrato de `loadPub` de meeting-schedules-parser (entrada: ruta, bytes o URL de `.jwpub`/`.epub`; salida: lista de semanas). El dominio JPresentation MUST mapear esa salida a su propio `MeetingPart`, no persistir los nombres `mwb_*` en la UI.

#### Scenario: Cambio de parser
- GIVEN un `ScheduleParser` distinto (Rust nativo u otro)
- WHEN se registra en el contenedor
- THEN el cronómetro y multimedia siguen recibiendo `MeetingWeek`
- AND no hay que tocar Vue

### Requirement: Canciones como enlace al himnario
Las canciones extraídas SHALL ser números (`sjjm` track). El `MeetingPart` de tipo song guarda ese número y un enlace a `HymnalLibrary`, no una copia semanal del MP4. Si el número no está en el himnario remoto, se guarda el texto de la guía.

#### Scenario: Mismo cántico dos semanas
- GIVEN el himnario ya tiene el track 1
- WHEN otra semana vuelve a programar el 1
- THEN no hay descarga nueva
- AND Play usa el mismo archivo permanente

### Requirement: Medios por parte
Cada `MeetingPart` SHALL listar `MediaRef`:

- `embedded` — archivo que ya viene en el paquete de la publicación (típico: ilustraciones)
- `jwpub-media` — URI de esquema `jwpub-media://` u otro identificador de medio de la publicación que no incluye el binario
- `catalog` — clave de catálogo (símbolo, idioma, pista/issue) cuando el parser o la tabla Multimedia solo deja metadatos
- `local` — archivo añadido por el operador

#### Scenario: Ilustración en el JWPUB
- GIVEN una imagen empaquetada en la publicación de la semana
- WHEN se construye el programa
- THEN la parte correspondiente tiene un `MediaRef.embedded`
- AND no se pide red para proyectarla

#### Scenario: Vídeo referenciado y ausente
- GIVEN una parte con URI `jwpub-media://…` o clave de catálogo y sin archivo en el paquete
- WHEN el operador pide descargar medios de la semana
- THEN `MediaResolver` obtiene el binario y lo deja en cache
- AND la parte pasa a estado listo

### Requirement: Fallos parciales
Si el parser obtiene el programa pero falla un medio, el sistema SHALL mostrar el programa igual y marcar ese medio como pendiente. MUST NOT tirar abajo la reunión por un vídeo que no resolvió.

### Requirement: Persistencia
El `MeetingWeek` normalizado SHALL guardarse en SQLite por perfil + fecha + idioma. Una nueva consulta MAY refrescar metadatos; los binarios en cache no se borran salvo purga explícita.
