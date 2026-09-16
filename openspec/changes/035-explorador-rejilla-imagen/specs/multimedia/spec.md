# Delta — Multimedia (035)

## ADDED Requirements

### Requirement: Rejilla de miniaturas
El panel principal de Multimedia SHALL mostrar una rejilla de miniaturas (carpeta, vídeo, imagen) con el nombre debajo. Las subcarpetas MUST poder abrirse desde esa rejilla. Un filtro SHALL acotar por nombre en la carpeta actual.

### Requirement: Proporción en auditorio
Una imagen en stage MUST conservar su relación de aspecto en el auditorio (bandas negras si no llena el monitor). MUST NOT estirarse al marco 16:9.

#### Scenario: Lámina cuadrada
- GIVEN un jpeg 1:1 en stage
- WHEN el auditorio es 1920×1080
- THEN la imagen se ve cuadrada centrada
- AND el orador-espejo también
