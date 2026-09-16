# Proposal: Guía de la semana y explorador Multimedia

## Intent

Con el reloj de `031` ya vivo, la consola sigue sin programa ni medios. La pestaña Multimedia es un placeholder. Este change conecta el idioma de contenido del perfil con el catálogo público (`GETPUBMEDIALINKS`), baja el `.jwpub` de **esta y la próxima semana** (`mwb` + `w`), lo abre como ZIP anidado + SQLite, y pinta un explorador: carpetas por semana/parte, imágenes extraídas, vídeos pendientes o en cache, previsualización en la consola.

No se proyecta al auditorio. No se descarga el himnario. No se descifra un paquete ilegible.

## Scope

In scope:

- Puertos `PublicationCatalog`, `ScheduleParser`, `MediaResolver` y un `MediaProvider` `jw-org` que los orquesta. HTTP **solo en Rust**. Vue no ve URLs del CDN.
- Consulta a demanda: «Esta semana» / «Próxima semana». Issue `YYYYMM` del `mwb` (bimensual: mes de la semana, un reintento al mes anterior si 404) y del `w` (mensual). Sin barrido histórico.
- Lector de paquete: ZIP externo (`manifest.json` + `contents`) → ZIP interno → SQLite si la cabecera es `SQLite format 3`. Extraer `FilePath` embebidos a `week/{langwritten}/{yyyy-mm-dd}/`.
- `MeetingWeek` persistido en `meeting_programs` (perfil + fecha + kind). Árbol en Multimedia agrupado por parte cuando el HTML de `Document.Content` es texto; si no, carpeta plana de medios de ese documento.
- Explorador: lista virtualizada, estado embebido / listo / pendiente, panel de previsualización de imagen (y ficha de vídeo/cántico). Miniaturas en Rust.
- «Descargar medios de la semana»: imágenes desde el paquete (sin red); vídeos de catálogo (`mwbv`, `jwb-*`, …) a `week/`. Cánticos `sjjm` = enlace al himnario, **no** se copian a `week/`.
- Fixture sintético en tests (ZIP+SQLite mínimo). **Cero** `.jwpub` oficiales en el git.

Out of scope:

- AES-128 / claves maestras / descifrado de `Document.Content` o del `.db` (`010`: ilegible → error + EPUB/local + cronómetro suelto).
- Himnario 1–163 (`008`), Biblia, raíces locales USB (`021`).
- Abrir imagen/vídeo en el auditorio o el orador (`023` + stage).
- Cadena de partes del cronómetro desde la guía (`004`).
- Barrer todos los meses, login, scrapers.
- Selector `video_quality` en Configuración (se usa `best` del API).
- EPUB como parser primario (sí como oferta si el JWPUB no abre).

## Approach

1. Un change de implementación. El adaptador de catálogo/paquete vive junto al dominio de medios, no en Vue.
2. La skill `jwpub-content-extractor` se usa para **contenedor ZIP, tablas y GETPUBMEDIALINKS**. No se usa su pipeline criptográfico: choca con `proveedor-medios` y `010`.
3. `meeting-schedules-parser` es la *forma* de las partes (títulos, minutos, canciones), no se copia el npm al repo.
4. Fallo parcial: programa visible, medio pendiente, toast en consola, auditorio intacto.
