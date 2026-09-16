# Design: Guía de la semana y Multimedia

## Cause (031)

El reloj cuenta una parte suelta. No hay `MeetingWeek`, ni cache `week/`, ni pestaña Multimedia. El `MediaProvider` está vacío. El operador no puede ver lo que la guía de esa semana proyectaría.

## Catálogo (HTTP, solo Rust)

Host y path viven en el adaptador (`research/pub-media-getpubmedialinks.md`):

```
GET b.jw-cdn.org/apis/pub-media/GETPUBMEDIALINKS
  ?langwritten={content_locale}
  &pub=mwb|w|mwbv|…
  &issue=YYYYMM
  &track=N          # solo vídeos/cánticos
  &output=json
```

Sin autenticación. `files[lang][JWPUB][0].file.url` + `checksum`. El dominio solo ve `CatalogHit { key, checksum, size }`; el repo baja bytes.

«Esta semana» = lunes–domingo que contiene `LocalDate::today()` (o la fecha que mande el operador). `mwb` cubre dos meses: si `issue=YYYYMM` del lunes 404, un segundo intento al mes anterior. `w` es mensual. 404 = no publicado; no reintentar en bucle.

## Paquete JWPUB

```
.jwpub ZIP
  manifest.json
  contents ZIP
    *.db          SQLite
    *.jpg         embebidos
```

Apertura:

1. ZIP externo → `manifest` + `contents`.
2. ZIP interno → `.db` a un fichero temporal o `sqlite` en memoria + copia de imágenes a `week/`.
3. Si el `.db` no empieza por `SQLite format 3` → `UnreadablePub`. MUST NOT descifrar. UI: error + «usar EPUB» (catálogo) + «continuar sin guía».
4. Tablas: `Publication`, `Document`, `DatedText`, `Multimedia`, `DocumentMultimedia`.

`MediaRef`:

- `embedded` si `FilePath` no vacío (típico: jpeg en `contents`).
- `catalog { key_symbol, track, lang_meps, issue_tag, mime }` si `FilePath` vacío.
- `sjjm` + track → no se descarga en este change (himnario). Queda `pending_hymnal`.

`Document.Content`: si es UTF-8 y parece HTML (`<` …), `ScheduleParser` extrae partes (h1 fecha, h3 cánticos, bloques tesoro/AYF/LC, minutos entre paréntesis) y mapea medios por `BeginParagraphOrdinal`. Si es binario/cifrado, no hay clave: la semana existe igual, con una sola carpeta de medios ordenados por párrafo.

## Disco

```
{app_data}/media/
  week/{langwritten}/{yyyy-mm-dd}/     # lunes de esa semana
    pub/mwb_*.jwpub
    pub/w_*.jwpub
    img/…
    vid/…                              # tras «Descargar medios»
    thumb/{hash}.jpg
```

Checksum antes de volver a GET. Esta + próxima semana se conservan; no se purga himnario (aún no existe). Jobs de descarga fuera del hilo UI (`JobQueue`). Progreso por evento `week://progress`. Cancelable. Fallo = diálogo Reintentar / Continuar; lo bajado se queda.

## Vue (solo operador)

Pestaña Multimedia:

- Toolbar: Esta semana | Próxima | Actualizar | Descargar medios pendientes.
- Árbol: semana → entre semana / fin de semana → partes (o «Media») → ítems.
- Panel derecho de la pestaña (no el panel de la consola): preview de imagen desde cache; vídeo/cántico muestran título + estado, no un decoder.
- Doble clic **no** manda al stage en este change.

Comandos (operador): `week_fetch`, `week_get`, `week_download_media`, `week_cancel`. `output_get` no cambia.

## Tests (sin red, sin WebView)

- Fixture sintético: ZIP anidado + SQLite con `DatedText` de una semana, 1 jpeg embebido, 1 fila `mwbv` sin `FilePath`, 1 `sjjm`.
- Parser: semana, `MediaRef` embebido vs catálogo, `sjjm` no se copia a `week/`.
- Cabecera no-SQLite → `UnreadablePub`.
- Issue: lunes 7 sep 2026 → `202609`; 404 de `mwb` en mes impar → reintento mes anterior.
- Catalog mock: no HTTP real en `cargo test`.

## Skill vs spec

| Skill | Este change |
|-------|-------------|
| ZIP anidado, `manifest`, tablas Multimedia/DatedText | Sí |
| GETPUBMEDIALINKS | Sí, en el adaptador |
| AES-128-CBC + clave maestra de `Document.Content` | **No** (`010` / `proveedor-medios`) |
| `jwpub-media://` en HTML | Normalizar a clave `Multimedia` si aparece; no es HTTP |
| Protocolo custom Tauri para el auditorio | Después (`023`) |
