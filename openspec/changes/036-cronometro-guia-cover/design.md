# Design

## Cover

`<img>`: `min-width/min-height: 100%`, centrado, `width/height: auto`. No `width+height: 100%` (WebKitGTK deforma). Vídeo sigue `object-fit: contain`.

## Outline

Bloques HTML (`h2/h3/p/li`) → texto. Si hay `(N min)` / `(N mins)` / `(N minutos)`, es una parte. Cánticos por “Song/Canción N”. Sin cubo “Media” en el cronómetro.

`MeetingPart.tone`: `treasures` | `ayf` | `living` | `song` | `other`.

## Panel

`TimerControls` recibe `parts`. Finish → `clock_finish` + `clock_arm` de la siguiente fila. La pestaña muestra la tabla completa.
