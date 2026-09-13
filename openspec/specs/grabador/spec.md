# Grabador (retirado)

## Purpose

Documentar que el grabador de audio de JMultimedia 2.8.01 **no** forma parte de JPresentation.

## Requirements

### Requirement: Fuera de alcance
El sistema MUST NOT implementar grabación de la reunión, selección de micrófono, temporizador de grabación ni carpeta Record de capturas de audio.

#### Scenario: Búsqueda en la UI
- GIVEN cualquier pantalla de JPresentation
- WHEN el operador busca “Grabador”, “Iniciar” de micrófono o “Pausa” de grabación
- THEN no existen esos controles
- AND no se solicitan permisos de micrófono al SO
