# Delta — Ventanas (037)

## MODIFIED Requirements

### Requirement: Imagen a pantalla
Una imagen en el auditorio SHALL llenar la caja **sin recortar ni estirar** (`contain` a escala máxima): se agranda o se reduce hasta que un eje toque el borde. Si la proporción coincide con el monitor, MUST ocupar toda la pantalla. Si no (cuadrada, 4:3), MUST dejar franjas negras. MUST NOT verse como una estampilla en una foto de calidad de sala cuya proporción sí encaja.

#### Scenario: Foto 16:9 en 1080p
- GIVEN un jpeg apaisado de calidad de sala
- WHEN está en stage
- THEN llena el auditorio
- AND no se deforma ni se recorta

#### Scenario: Foto cuadrada
- GIVEN un jpeg 1:1
- WHEN está en stage
- THEN se ve completa
- AND hay franjas negras a los lados (o arriba/abajo), nunca un recorte
