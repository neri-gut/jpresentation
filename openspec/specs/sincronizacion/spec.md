# Sincronización de superficies

## Purpose

Una sola fuente de verdad en Rust. Consola, auditorio y orador se **suscriben**; no se sincronizan entre sí. El operador es el único que escribe. El panel derecho pinta el mismo snapshot que el auditorio, sin otro decoder.

## Estado

Tres snapshots versionados (`rev` monótono):

**`StageSnapshot`** — recurso abierto, o `idle`

- `kind`: `none` | `video` | `image` | `audio` | `bible` | `poster` | `page` | `stream`
- `asset` (ruta o URL), `page` / `pages`, `paused`, `position_ms`, `duration_ms`
- `poster`: `daily` | `year` | `custom` | `none` cuando `kind` es idle/poster

**`ClockSnapshot`**

- parte `running` | `armed` | `idle`, ids, asignado, transcurrido, restante, desfase
- historial de la reunión (para deshacer)

**`AudioSnapshot`**

- máster, ambiente on/off, fuente sonando (`hymnal` | `stage` | `ambient` | `browser` | `none`)

**`SpeakerUi`**

- mensaje corto o vacío  
- flags HUD (no el diseño: eso es perfil)

Vue no guarda una copia “dueña”. Al `invoke` exitoso llega un evento `jp://state` con los snapshots (o un diff por `rev`).

## Requirements

### Requirement: Un escritor
Solo comandos de la ventana Operador mutan estado (`stage.open`, `stage.pause`, `clock.start`, `clock.finish`, `bible.page`, `speaker.say`, …). Auditorio y orador MUST NOT emitir esos comandos. El panel derecho usa los mismos comandos.

### Requirement: Fan-out
Tras cada mutación, Rust publica el snapshot. Destinos:

| Campo | Auditorio | Orador | Panel / pestañas |
|---|---|---|---|
| Vídeo/imagen/audio/stream en Play o **Pausa** | sí | sí si `espejo+HUD` | preview |
| Página web | webview URL | otra webview misma URL, o solo HUD | preview nula o captura ligera |
| Texto Biblia / cartel enviado | sí | sí + HUD | preview |
| Texto diario (idle, recurso cerrado) | sí | **no** (HUD sobre negro) | preview del diario |
| Texto del año (hora de inicio) | sí | **no** | preview |
| Reloj | no | HUD | bloque cronómetro |
| Mensaje al discursante | no | franja | bloque mensajes |
| Máster / ambiente | se oye | silencio | sliders |

Si orador está `off` o `solo HUD`, no recibe fotograma.

### Requirement: Reloj
Tick de reloj en Rust (≈ 4 Hz basta para `mm:ss`; barra MAY 10 Hz). No cada ventana lleva un `setInterval` dueño. Al reconectar, el orador pide snapshot y pinta el restante correcto; no “reinicia la parte”.

### Requirement: Vídeo
Un clock de media en Rust o en el player del auditorio **reportando** posición al puerto (≥ 4 Hz en Play). Panel y orador-espejo siguen ese `position_ms`. Seek y pausa solo desde operador → todos saltan al mismo `rev`.

MUST NOT haber tres decoders 1080p. Orador-espejo: mismo frame o downscale.

### Requirement: Páginas
`bible.page` incrementa `page` en el snapshot. Auditorio y orador pintan el mismo `page` en el siguiente evento. Sin cola en el orador.

### Requirement: Reconexión
Si la consola recarga: auditorio **no** se toca; al `subscribe` la consola recibe el snapshot actual. Si el orador aparece a mitad: HUD + espejo según modo, sin resetear el reloj ni el seek.

#### Scenario: Pausa
- GIVEN vídeo a 00:32
- WHEN el operador pausa
- THEN auditorio, orador-espejo y panel quedan en 00:32
- AND el diario no sustituye el fotograma

#### Scenario: Diario vs orador
- GIVEN stage idle con texto diario
- WHEN hay monitor orador
- THEN auditorio = diario
- AND orador = HUD sobre negro

#### Scenario: Consola se recarga
- GIVEN auditorio con imagen
- WHEN Vite recarga el operador
- THEN la imagen no parpadea
- AND el panel vuelve a mostrar esa imagen al recibir `jp://state`

### Requirement: Webview (excepción)
Dos documentos vivos no se pueden fotograma-a-fotograma. Se sincroniza la **URL** y el `rev` de “página abierta”. Scroll interno de la página no se replica. Cerrar en el operador cierra ambos webviews.
