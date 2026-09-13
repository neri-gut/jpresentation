# Audio

## Purpose

Todo lo que oye el salón sale por **un** dispositivo: el del auditorio. Un máster por perfil. Canales internos (cántico, vídeo de guía, ambiente, navegador) existen en el puerto para el futuro, pero v1 no muestra sliders por canal. Sin precucha, sin mute global, sin micrófono.

## Requirements

### Requirement: Una salida pública
`AudioPort` SHALL enviar cánticos, vídeos, ambiente y audio de páginas web al dispositivo ligado a la superficie **Auditorio**. La ventana Orador MUST ir silenciada (espejo visual + HUD). La consola MUST NOT tener un bus de precucha en v1.

Si el SO solo expone un dispositivo, ese es el Auditorio.

#### Scenario: Cántico y orador
- GIVEN MP4 del himnario en Play
- WHEN el orador tiene monitor
- THEN el salón oye el cántico
- AND el monitor del orador no duplica el audio

### Requirement: Máster único, listo para canales
v1: un `volume_master` 0–100 por perfil, más mute por **recurso en curso** (pausa). El dominio SHALL modelar canales (`hymnal`, `week_media`, `ambient`, `browser`) para que un change futuro añada sliders sin romper `AudioPort`. v1 aplica el máster a todos.

Persistencia: el máster (y más adelante cada canal) vive en el perfil.

### Requirement: Un control para todo
Play/pausa/volumen de la consola mandan sobre cualquier fuente en el auditorio. No hay un segundo mezclador por pestaña.

### Requirement: Pausa del recurso, no mute de emergencia
No hay botón “silencio total” aparte. Detener el sonido = **Pausa** del vídeo o recurso activo (cántico, vídeo de guía, ambiente, web). Reanudar sigue en el mismo punto cuando el formato lo permite.

### Requirement: Ambiente solo en auditorio
`ambient/{profile}/` con lista propia. Opciones de lista: **loop** o **terminar al último ítem**. Fade out al lanzar un cántico (ms: `specs/fundidos`).

Arranque:

- `manual` — solo Play del operador
- `clock` — además, reglas ligadas al cronómetro (configurables): p. ej. al estar en idle antes de la primera parte, al cerrar una parte, al terminar la reunión

Defecto: `manual`. MUST NOT empezar al abrir la app.

#### Scenario: Loop
- GIVEN lista de 4 piezas y loop on
- WHEN acaba la 4
- THEN vuelve a la 1 en el auditorio

#### Scenario: Auto según cronómetro
- GIVEN `ambient.mode = clock` y regla “al terminar la reunión”
- WHEN se cierra la última parte
- THEN arranca el ambiente en el auditorio
- AND un cántico de reunión lo para con fade

### Requirement: Flujo himnario MP4 / MP3
Play de ranura o catálogo:

1. Si existe MP4 de la calidad del perfil → auditorio muestra vídeo y oye ese audio  
2. Si no hay MP4 y sí MP3 opcional → auditorio oye el MP3; imagen = negro o último cartel, no un vídeo vacío  
3. Si no hay ninguno → descarga MP4 (calidad del perfil); el MP3 solo si el operador activó “también audio”

No se precucha en consola.

#### Scenario: Solo MP3
- GIVEN track 38 con MP3 y sin MP4
- WHEN Play
- THEN el salón oye el audio
- AND el orador ve negro/cartel + HUD, sin clip mudo
