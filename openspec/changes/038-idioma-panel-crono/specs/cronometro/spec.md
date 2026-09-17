# Delta — Cronómetro (038)

## MODIFIED Requirements

### Requirement: Sin cánticos en el cronómetro
Las filas de cántico (Song / Canción / `sjjm`) MUST NOT aparecer en la tabla ni en el panel. Siguen en `MeetingWeek.media` para Canciones. Terminar arma la siguiente parte **no** canción.

#### Scenario: Guía con tres cánticos
- GIVEN un `mwb` con tracks `sjjm` 1, 128 y 143
- WHEN se carga el cronómetro
- THEN no hay filas «Song 1» ni equivalentes
- AND tesoro / perlas / lectura sí están

### Requirement: Reloj operativo solo a la derecha
La pestaña Cronómetro SHALL editar el programa (guía, plantillas, filas). Iniciar / Pausar / Terminar y la lista para armar MUST vivir en el panel derecho, no duplicados en la pestaña.

## ADDED Requirements

### Requirement: Nombres de sección según contenido
Si el outline sale de plantilla (Content no HTML), los títulos SHALL seguir el `content_locale` del perfil. Español (`S`): nombres de la reunión (Tesoros, Perlas, …). Inglés: los equivalentes. MUST NOT usar el menú de UI para decidir el `mwb` descargado.
