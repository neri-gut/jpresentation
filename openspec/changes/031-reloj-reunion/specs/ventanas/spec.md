# Delta — Ventanas (031)

## ADDED Requirements

### Requirement: Modo del orador
Cada perfil SHALL guardar `speaker_mode`: `mirror` (espejo+HUD, defecto) o `hud_only` (solo HUD). `hud_only` MUST pintar el cronómetro a pantalla completa aunque el stage tenga un medio. `mirror` usa el HUD compacto encima del medio cuando `kind ≠ none`, y el HUD a pantalla cuando no hay medio. Celdas `surfaces` antiguas sin el campo MUST leerse como `mirror`.

#### Scenario: Solo HUD
- GIVEN `use_speaker = true` y modo `hud_only`
- WHEN hay (o habrá) un recurso en stage
- THEN el orador sigue en negro con el reloj a pantalla
- AND el auditorio no recibe el overlay

### Requirement: Lista de monitores etiquetada
El selector de pantallas SHALL marcar la que ocupa el operador («console») y la primaria del SO («primary»). La opción de id vacío SHALL llamarse ventana de previsualización, sin implicar que solo existe un monitor.

#### Scenario: Dos monitores, preview
- GIVEN escritorio extendido y Auditorio en preview
- WHEN el operador abre Configuración
- THEN ve «Preview window» como opción vacía
- AND el display de la consola figura como console
