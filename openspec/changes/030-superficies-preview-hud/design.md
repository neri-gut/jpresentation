# Design: Superficies — dueño, preview, HUD

## Cause (029)

`place_audience(None)` con ≥2 monitores **robaba** la secundaria en fullscreen. Con 1 monitor, asignar el único display cubría al operador. El orador, si se creaba, quedaba debajo de ese cover o `hide()`-do. Las ventanas no tenían `parent` ni listener de cierre.

## Placement

```
AudiencePlacement { Preview | Fullscreen(id) }

audience_placement(assigned, known_ids, operator_monitor_id):
  None                         → Preview
  id desconocido               → Preview
  id == monitor del operador   → Preview
  un solo display              → Preview
  else                         → Fullscreen(id)
```

```
speaker_placement(..., operator_monitor_id):
  use_speaker = false          → Hidden
  id vacío                     → Preview
  id desconocido o = auditorio → Preview + missing
  id == monitor del operador   → Preview (no missing: es una elección válida)
  un solo display              → Preview
  else                         → Monitor(id)
```

Preview: ventana **nueva o recreada** con decoraciones, 960×540, resizable, título `JPresentation — Audience` / `JPresentation — Speaker`, posición en cascada desde el operador (`+40/+80` y `+88/+160` px). `show` + `unminimize`. MUST NOT `set_fullscreen(true)`.

Cover (no exclusive fullscreen): sin decoraciones, `position` + `size` del monitor elegido, distinto del operador.

Al cambiar `decorated`, `destroy` + crear de nuevo (mismo label). GTK no cambia el marco en caliente.

`parent(&operator)` al crear auditorio y orador.

## Cierre

En `setup`, el operador escucha `CloseRequested` y `Destroyed`. `close_owned_surfaces` destruye toda ventana cuyo label no sea `operator` (audience, speaker, `identify-*`). No se llama `app.exit` a mano: al cerrar el operador no quedan webviews.

Cerrar solo el auditorio o solo el orador no mata la consola.

## HUD orador

`SpeakerApp` lee `output.stage.kind`:

| `kind` | Layout |
|--------|--------|
| `none` (hoy siempre) | Cronómetro a pantalla: título, `mm:ss` grande (`clamp`), barra inferior a todo el ancho |
| otro (más adelante) | Overlay compacto + la **misma** barra inferior |

Sin tick real: el tiempo sigue siendo el placeholder `00:00`; la barra queda al 0 %. El cableado `kind` evita rehacer el layout cuando llegue el stage.

El **auditorio** no pinta el cronómetro (spec: HUD en orador o en el panel).

## Tests (sin WebView)

- `audience_placement`: None con 2 monitores → Preview; id del operador → Preview; id HDMI con operador en primary → Fullscreen; 1 display → Preview.
- `speaker_placement`: id del operador → Preview sin missing; HDMI libre → Monitor; vacío → Preview.
