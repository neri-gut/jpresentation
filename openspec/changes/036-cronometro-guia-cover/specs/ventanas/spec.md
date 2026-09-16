# Delta — Ventanas (036)

## ADDED Requirements

### Requirement: Imagen a pantalla
Una imagen en el auditorio SHALL cubrir el monitor (`cover`: proporción conservada, recorte si hace falta). MUST NOT verse como una estampilla pequeña ni estirada.

#### Scenario: Foto 16:9 en 1080p
- GIVEN un jpeg apaisado de calidad de sala
- WHEN está en stage
- THEN llena el auditorio
- AND no se deforma
