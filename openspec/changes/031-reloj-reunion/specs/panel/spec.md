# Delta — Panel (031)

## ADDED Requirements

### Requirement: Bloque Cronómetro operativo
El bloque Cronómetro de la columna derecha SHALL pintar el mismo `ClockSnapshot` que el orador y exponer Armar / Iniciar / Pausar / Terminar. Colapsado, MUST mostrar al menos el `mm:ss` (o desfase). Si `use_speaker = false`, este bloque SHALL ser el HUD del operador. La pestaña Cronómetro MAY reutilizar los mismos controles. MUST NOT desmontar el bloque al cambiar de pestaña.

#### Scenario: Orador desactivado
- GIVEN `use_speaker = false` y una parte running
- WHEN el operador mira el panel
- THEN ve título, restante y barra
- AND no hay tercera ventana
