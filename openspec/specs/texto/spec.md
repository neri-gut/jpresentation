# Carteles de texto

## Purpose

Componer carteles (título, avisos, texto del año, grupo de limpieza) y un **texto diario** manual que sirve de fondo del auditorio entre partes cuando no hay imagen, vídeo ni cántico.

## Requirements

### Requirement: Editor de cartel
Texto enriquecido básico (negrita, cursiva, subrayado, tachado), color, tamaño, familia, alineación (incl. middleCenter), color de fondo, imagen de fondo y márgenes. Plantillas guardadas reutilizables.

#### Scenario: Título de discurso público
- GIVEN un título de discurso
- WHEN el operador formatea y pulsa Mostrar texto
- THEN la previsualización coincide con el auditorio

### Requirement: Texto diario manual
No hay API ni base de citas diarias. El operador SHALL pegar o escribir **cita** (p. ej. `Salmo 83:18`) y **texto** a mano, por perfil y fecha (o “vigente hasta que lo cambie”). Se guarda en SQLite del perfil. MUST NOT consultar WOL ni GETPUBMEDIALINKS para rellenar este cartel.

El texto diario es el **reposo solo del auditorio**. MUST NOT enviarse a la superficie Orador.

Se muestra en auditorio únicamente cuando **no hay recurso abierto**: el operador cerró el medio o el vídeo/cántico llegó a fin. **Pausa no cuenta como cierre**: el auditorio se queda en el fotograma/audio pausado.

Al Abrir/Play de un recurso, el diario se oculta hasta el cierre o el fin natural.

#### Scenario: Entre partes sin medio
- GIVEN texto diario guardado y ningún recurso abierto
- WHEN el tesoro termina en el cronómetro
- THEN el auditorio muestra el texto diario
- AND el orador ve solo el HUD (parte siguiente armada), sin el cartel

#### Scenario: Pausa de cántico
- GIVEN un cántico en el auditorio
- WHEN el operador pausa
- THEN el auditorio sigue en ese vídeo pausado
- AND no aparece el texto diario
- AND el orador sigue viendo el mismo vídeo pausado + HUD

#### Scenario: Fin o cierre
- GIVEN un vídeo de la guía
- WHEN llega al final o el operador cierra el recurso
- THEN el auditorio pasa al texto diario
- AND el orador pasa a HUD sobre negro (sin diario)

### Requirement: Texto del año y otros carteles
El perfil SHALL poder guardar un cartel de **texto del año**. A la hora de inicio de la reunión (ver `cronometro`) ese cartel se muestra en el **auditorio**. “Mostrar texto” también lo envía a mano. El orador no lo ve.

El reposo **entre partes** (recurso cerrado) es el texto diario, no el del año.

### Requirement: No perder el medio
Mostrar un cartel a mano MAY sustituir el medio. Ocultar restaura el medio anterior si sigue disponible; si no, el reposo (texto diario).
