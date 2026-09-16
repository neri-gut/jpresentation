# Design: Explorador y stage

## Cause (032)

Multimedia pintó `MeetingWeek.parts` (tesoro, canciones, minutos). El operador necesita un árbol de **ficheros** y mandarlos al auditorio. El cronómetro seguía con una parte suelta desconectada de la guía.

## Explorer

Raíces virtuales:

| Id | Disco |
|----|--------|
| `week:this` / `week:next` | `media/week/{lang}/{monday}/` (`img` + `vid` + `pub`) |
| `root:{path}` | carpetas del perfil |
| `pub:{hash}` | `media/local-pub/{hash}/` tras inspeccionar un `.jwpub` |

`explorer_list(path)`: un directorio, perezoso, filtrado. `.jwpub` es entrada de fichero; al abrirlo se extraen jpg/png/webp/gif/mp4/webm del ZIP interno (no el `.db`) y se lista esa carpeta.

Jail: path canónico MUST ser prefijo de `media_root` o de una raíz guardada.

## Stage

`StageKind = none | image | video`. `stage_open(path)` copia a `media/stage/current.{ext}` (así el asset protocol solo cubre app data) y publica `StageSnapshot { kind, name, mime, path }`. `stage_close` borra el current y vuelve a `none`.

Auditorio: `<img>` / `<video autoplay>` con `convertFileSrc(path)`. Orador: igual si `mirror`; HUD a pantalla si `hud_only` o `none`. El reloj no se toca.

## Cronómetro

`week_get` / `week_fetch` ya existen. La pestaña lista partes midweek/weekend. Clic → `clock_arm(title, minutes)`. «Obtener guía» = `week_fetch`. Sin iconos de medio en esa lista.

## Tests

- `list_dir` ignora `.db` y `.txt`; incluye jpg/mp4/jwpub.
- Jail: path fuera de raíces → error.
- Extraer jwpub sintético deja `test.jpg`, no el sqlite.
- `stage_open` de un jpg copia a `stage/` y el snapshot es `image`.
