# Panel de control (columna derecha)

## Purpose

Mando único de la consola: lo que está en **stage**, audio, cronómetro del operador, Biblia y mensajes al orador. Siempre visible al cambiar de pestaña. No contiene reglas de reunión: solo invoca `OutputPort`, `AudioPort`, `MeetingClock`, `BibleSource`, `HymnalLibrary`. Sin Grabador.

## Requirements

### Requirement: Persistente y denso
Columna derecha de la ventana Operador. MAY colapsarse a iconos. El perfil guarda ancho y si está colapsado. Cambiar a Canciones / Multimedia / Biblia / Texto / Navegador MUST NOT desmontar el panel ni cortar vídeo, audio o reloj.

### Requirement: Bloque Stage (vídeo / imagen / texto / web)
Previsualización del **mismo** fotograma que el auditorio (copia barata; no un segundo decoder 1080p — ver `rendimiento`). Controles:

- Abrir / Cerrar recurso  
- Anterior / Siguiente (contexto actual: carpeta, jwpub, parte de la guía o cola de textos)  
- Recargar  
- Zoom  
- Play / Pausa / seek si el recurso es vídeo o stream  
- Indicador: qué hay en stage (nombre, tipo, página 1/N)

Cerrar → reposo (texto diario en auditorio). Pausa **no** cierra. Solo el operador.

### Requirement: Bloque Audio
Máster 0–100 del perfil. Play / Pausa / Stop del recurso con sonido (cántico, vídeo, stream, ambiente). Ambiente: Play/Stop y loop según `specs/audio`. Ranuras de cántico **Inicio / Central / Final** (número + Play): resuelven `HymnalLibrary`, no bajan la guía.

### Requirement: Bloque Cronómetro
Parte vigente o armada, asignado, restante o desfase, Iniciar / Pausar reloj / Terminar / **regreso rápido**. Hora actual. No abre medios. Al Terminar arma la siguiente; Iniciar arranca la cuenta. Hasta que exista cadena de partes, Terminar deja `idle`. Colapsado, MUST mostrar al menos el `mm:ss`. Si no hay ventana orador, este bloque SHALL ser el HUD del operador. La pestaña Cronómetro MAY reutilizar los mismos controles.

#### Scenario: Orador desactivado
- GIVEN `use_speaker = false` y una parte running
- WHEN el operador mira el panel
- THEN ve título, restante y barra
- AND no hay tercera ventana

### Requirement: Bloque Biblia
Campo de referencia, Enviar al stage, Anterior / Siguiente **página** (solo operador), Cerrar texto. La cola larga se edita en la pestaña Biblia; aquí va el ítem activo.

### Requirement: Bloque Mensajes al orador
Texto corto o chips (`1 min`, `Concluya`, …). Enviar / Borrar. Solo monitor orador. No tapan el HUD. El auditorio no los ve.

### Requirement: Sin Grabador
MUST NOT aparecer rec, micrófono ni carpeta Record.

#### Scenario: Cambia de pestaña
- GIVEN un vídeo en stage y tesoro en marcha
- WHEN pasa de Multimedia a Navegador
- THEN el panel sigue, el vídeo no parpadea y el reloj no se reinicia

#### Scenario: Paginación desde el panel
- GIVEN un salmo en stage página 1/3
- WHEN pulsa Siguiente en el bloque Biblia
- THEN auditorio y orador muestran página 2
