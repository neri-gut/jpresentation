# Perfiles, idiomas y configuración

## Purpose

Varias congregaciones en el mismo equipo. Cada perfil separa **idioma de la UI** (traducción de JPresentation) e **idioma de contenido** (`langwritten` del catálogo JW: guía, himnario, Biblia). La comunidad puede añadir traducciones de UI sin recompilar Rust.

## Requirements

### Requirement: Multi-perfil
Crear, duplicar, renombrar y borrar. Arranque: último usado. Cada perfil guarda: `ui_locale`, `content_langwritten`, `video_quality`, provider, cache, carpetas locales, símbolo de Biblia, monitores, horarios de las dos reuniones, estilo del countdown de pre-reunión, texto del año, texto diario, plantillas. Sin carpeta Record.

#### Scenario: Dos congregaciones
- GIVEN “Cong A” UI `es` + contenido `S`, y “Cong B” UI `en` + contenido `E`
- WHEN cambia a Cong B
- THEN menús pasan a inglés y las rutas de himnario/guía/Biblia a `E`
- AND no se mezclan caches

### Requirement: No borrar el último perfil
El sistema MUST NOT borrar el único perfil que quede. La consola pide confirmación antes de borrar cualquiera. Si se borra el perfil activo y hay otros, el sistema SHALL seleccionar el más reciente de los restantes y rehidratar la consola. Las celdas de `settings` (y `meeting_programs`) del perfil borrado SHALL eliminarse en la misma transacción (`ON DELETE CASCADE`).

#### Scenario: Único perfil
- GIVEN solo existe `Default`
- WHEN el operador confirma Borrar
- THEN el perfil sigue ahí
- AND la consola muestra `LastProfile`
- AND el auditorio no se cierra

#### Scenario: Borrar el activo habiendo otro
- GIVEN “Cong A” (activo) y “Cong B”
- WHEN borra “Cong A”
- THEN queda “Cong B” seleccionado
- AND los settings de A ya no están

### Requirement: Duplicar copia settings
Duplicar SHALL crear un perfil nuevo (otro id, `ui_locale` y `content_locale` iguales, nombre `"{name} (copy)"` recortado a 80) y copiar **todas** las celdas de `settings` del origen. MUST NOT seleccionar el duplicado por sí solo.

#### Scenario: Duplicar congregación
- GIVEN “Cong A” con UI `es`, contenido `S`, tema `dark` y auditorio en el monitor 2
- WHEN duplica
- THEN existe “Cong A (copy)” con los mismos locale, tema y monitores
- AND “Cong A” sigue activo

### Requirement: Dos ejes de idioma
`ui_locale` (BCP-47: `es`, `en`, `pt-BR`…) y `content_langwritten` (código JW: `S`, `E`, `CHS`, `A`…) son independientes. Un operador MAY usar UI en español y contenido en `TG` (tagalo).

El código JW, nombre, `locale`, `direction` y `script` salen del catálogo (`GETPUBMEDIALINKS?alllangs=1`). No se mantiene a mano una tabla de 500 lenguas como fuente de verdad. Un **semilla** de idiomas frecuentes MAY ir embebida para pintar el selector sin red.

Hasta que un change posterior pegue al catálogo vivo, el selector SHALL listar esa semilla (`research/idiomas.md`: `E S F I X T U J KO CHS CH A G O P K IN TG AM HI Z`), filtrable por nombre o código. Un `content_langwritten` fuera de la semilla MUST rechazarse al persistir (`UnknownContentLanguage`). Perfil nuevo: `content_locale = E`.

#### Scenario: UI en español, contenido tagalo
- GIVEN un perfil con `ui_locale = es`
- WHEN elige contenido `TG`
- THEN los menús siguen en español
- AND `content_locale` queda `TG`
- AND no hay petición de red

#### Scenario: Código inventado
- GIVEN un `profile_update` con `content_locale = "ZZZ"`
- WHEN Rust valida
- THEN responde `UnknownContentLanguage`
- AND la fila no cambia

### Requirement: Horario de las dos reuniones
Cada perfil SHALL guardar, en hora **local del equipo**:

- entre semana: `midweek_weekday` (ISO 1=lunes … 7=domingo) + `midweek_time` (`HH:mm` 24h)
- fin de semana: `weekend_weekday` + `weekend_time`

Defecto: martes 19:00 / domingo 10:00. Weekday fuera de 1–7 o tiempo que no cumpla `HH:mm` MUST rechazarse. Vive en settings (`meeting_schedule`); el asistente de `027` lo reutiliza, no lo redefine.

#### Scenario: Martes por la noche
- GIVEN un perfil nuevo
- WHEN el operador pone entre semana martes 19:30 y guarda
- THEN `meeting_schedule.midweek_weekday = 2` y `midweek_time = "19:30"`
- AND el auditorio no cambia

### Requirement: Catálogo de contenido
Al abrir el selector de idioma de contenido, el sistema SHALL mostrar las lenguas del catálogo (filtrable por nombre o código). Si una publicación no existe en esa lengua (p. ej. `nwtsty` 404 en árabe), el módulo afectado queda pendiente y ofrece otra edición o archivo local. El resto (guía, himnario) sigue si el catálogo las tiene.

#### Scenario: Comunes verificados (2026-09)
- GIVEN `mwb` issue `202609`
- WHEN se pide `E S F I X T U J KO CHS CH A G O P K IN TG AM HI Z`
- THEN todas responden JWPUB
- AND `nwtsty` falta en `A`, `G`, `AM` (404) — la Biblia de esos perfiles MUST elegir otra edición o JWPUB local

### Requirement: UI traducible por la comunidad
Fuente de claves, idioma por defecto y fallback: **inglés** (`en`). v1 SHALL empaquetar los JSON de: `en`, `es`, `fr`, `de`, `it`, `pt-BR`, `ru`, `uk`, `pl`, `nl`, `id`, `ja`, `ko`, `zh-Hans`, `zh-Hant` (muestras en `research/locales-ui/`).

Formato: JSON UTF-8 anidado vue-i18n. Añadir otro idioma: copiar `en.json` → `{bcp47}.json` + fila en `index.json`. Claves nuevas solo en `en.json`. Placeholders `{name}` / `{n}`. Fallback: locale pedido → `en` → clave cruda. Perfil nuevo: `ui_locale = en`.

#### Scenario: Perfil nuevo
- GIVEN instalación limpia
- WHEN se crea un perfil
- THEN `ui_locale` es `en`

#### Scenario: Clave ausente
- GIVEN UI en `es` y una clave nueva solo en `en.json`
- WHEN se pinta esa cadena
- THEN se muestra el valor inglés

#### Scenario: Aportar un idioma extra
- GIVEN `locales/sv.json` copiado de `en.json` y fila en `index.json`
- WHEN se construye la app
- THEN el menú ofrece ese idioma
- AND no hay cambios en crates Rust

### Requirement: Calidad de vídeo única
`video_quality` del perfil aplica a **todo** MP4: himnario, `mwbv`, `jwb-*` y cualquier otro del catálogo. Valores: `best` | `1080p` | `720p` | `480p` | `360p` | `240p`. Defecto: `best` (el `label` más alto que ofrezca ese ítem, incluido 1080p si existe). Si pide 1080p y el ítem solo llega a 720p, se usa 720p. Un cambio MAY re-descargar solo lo que no tenga ya ese label.

### Requirement: Configuración
Monitores, dispositivo y máster de audio, calidad de vídeo, atajos, fundidos, provider, idioma de contenido, UI, carpetas, cache, actualización, cronómetro, alertas visuales, monitor orador, modo ambiente, plantillas de evento. Sin micrófono. Borrar perfil o cache pide confirmación.
