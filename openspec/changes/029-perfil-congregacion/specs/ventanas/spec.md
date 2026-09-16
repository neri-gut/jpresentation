# Delta — Ventanas (029)

## ADDED Requirements

### Requirement: Orador activo sin pantalla = previsualización
`usar orador = off` → no hay ventana orador (el HUD vive en la consola). `usar orador = on` y `speaker_monitor_id` vacío, ausente o igual al del auditorio → ventana **flotante de previsualización** (decorada, no fullscreen sobre la consola). MUST NOT ocultarse. MUST NOT mover el auditorio. Si el id guardado ya no existe, la consola muestra `MonitorMissing`.

#### Scenario: Orador sin pantalla asignada
- GIVEN `use_speaker = true` y `speaker_monitor_id` vacío
- WHEN se aplican las superficies
- THEN el orador abre como ventana flotante
- AND no se oculta
- AND no se pone fullscreen sobre la consola

#### Scenario: Monitor de orador desconectado
- GIVEN orador asignado al id `HDMI@0,0` y ese monitor ya no está
- WHEN arranca o se reaplican superficies
- THEN el orador pasa a previsualización
- AND el auditorio permanece donde estaba
- AND la consola muestra `MonitorMissing`

### Requirement: Identificar monitores
Configuración SHALL ofrecer Identificar. El sistema MUST mostrar en **cada** pantalla detectada un destello con el nombre y la resolución durante **2 s**, y luego destruir esas ventanas. MUST NOT cambiar la colocación de auditorio ni orador. Sin IPC en esas ventanas.

#### Scenario: Identificar
- GIVEN dos monitores
- WHEN el operador pulsa Identificar
- THEN cada pantalla muestra su nombre y `width×height` unos 2 s
- AND auditorio y orador siguen en su sitio al terminar
