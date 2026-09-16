# Cronómetro y programa de reunión

## Purpose

Mostrar los horarios de la guía de actividades de **ese día/semana**, correrlos en vivo y, al terminar una sección, pasar solo a la siguiente. Origen: `MeetingWeek` parseado del `mwb` (entre semana) o del `w` (estudio de fin de semana).

## Requirements

### Requirement: Una parte suelta sin guía
Hasta que un `MeetingWeek` esté cargado, el operador SHALL poder armar **una** parte (título 1–80, minutos 1–180), iniciarla, pausarla y terminarla. El tick MUST vivir en Rust (~4 Hz). Vue MUST NOT ser dueña de un `setInterval` de cuenta. Esos gestos MUST NOT abrir ni cerrar medios.

Pausar SHALL dejar la parte `armed` con el transcurrido. Terminar SHALL pasar a `idle` (no arma la siguiente mientras no haya cadena). A 0:00 la misma parte sigue y el desfase crece.

#### Scenario: Tesoro de 10 minutos
- GIVEN el reloj idle
- WHEN el operador arma «Tesoro» 10 min e Iniciar
- THEN orador y panel muestran restante `10:00` que decrece
- AND el auditorio no cambia de medio

#### Scenario: Pausa y retoma
- GIVEN una parte running a 4:00 transcurridos de 10
- WHEN pausa y luego Iniciar
- THEN el restante sigue desde 6:00

#### Scenario: Se pasan
- GIVEN 10 min y transcurridos 11:20
- WHEN el orador mira el HUD
- THEN ve `+1:20` en rojo
- AND la barra está al 100 %

### Requirement: Horarios de la guía del día
Al abrir Cronómetro con una fecha, el sistema SHALL cargar el `MeetingWeek` de esa fecha si existe. Las filas son las secciones de la guía: canciones (con número), tesoro, gemas, lectura, AYF 1–n, vida cristiana 1–n, CBS, más introducción/conclusión si el parser o la plantilla las aportan. Minutos, título y color vienen de la guía; el operador MAY editar.

#### Scenario: Entre semana ya parseada
- GIVEN `mwb` de sep 2026 parseado y fecha 7–13 sep
- WHEN abre Cronómetro
- THEN ve el orden de esa semana con minutos AYF/LC
- AND las filas de canción muestran el número enlazado al himnario
- AND no hace falta pulsar “cargar plantilla vacía” primero

#### Scenario: Guía aún no consultada
- GIVEN cache de programa vacía
- WHEN abre Cronómetro
- THEN ofrece “Obtener guía de esta semana”
- AND hasta entonces puede usarse la plantilla genérica

### Requirement: Cadena de secciones
Una sola parte **en marcha**. Al **Terminar** una (operador, o 0:00 si `auto_advance_on_zero`), esa fila se cierra y se **arma** la siguiente: título, minutos y HUD listos. El cronómetro de la siguiente MUST NOT empezar a contar hasta que el operador pulse **Iniciar**. MUST NOT abrir ni cerrar medios.

#### Scenario: Cierra y arma
- GIVEN tesoro en marcha
- WHEN el operador pulsa Terminar
- THEN tesoro queda en historial (inicio, fin, reales, restante)
- AND perlas quedan armadas en panel y orador
- AND el reloj de perlas no corre hasta Iniciar
- AND el auditorio no cambia de medio por este gesto

#### Scenario: 0:00
- GIVEN `auto_advance_on_zero = true` y tesoro a 0:00
- WHEN llega el cero
- THEN tesoro se cierra y perlas se arman
- AND Iniciar queda a un toque
- AND ningún vídeo ni cántico arranca

#### Scenario: Se pasan con auto off
- GIVEN `auto_advance_on_zero = false`
- WHEN llega a 0:00
- THEN la misma parte sigue y el desfase crece
- AND Terminar es lo que arma la siguiente

### Requirement: Deshacer y historial
Cada parte de la reunión en curso SHALL guardarse en historial: id, título, asignado, inicio real, fin real, transcurrido, restante al cerrar. El operador (y un control de **regreso rápido** pensado para corregir un Terminar prematuro) SHALL poder:

- volver a la parte anterior armada o en marcha
- reabrir una parte del historial y **continuar** desde el restante / transcurrido guardado

MUST NOT borrar el historial al deshacer. El orador ve el HUD de la parte restaurada. Los medios no se tocan.

#### Scenario: Cerraron tesoro demasiado pronto
- GIVEN tesoro cerrado a 6:00 de 10 y perlas armadas
- WHEN pulsa regreso rápido / deshacer
- THEN tesoro vuelve a vigente con 4:00 restantes (o el transcurrido 6:00, según modo de cuenta)
- AND Iniciar retoma esa cuenta
- AND perlas dejan de estar armadas

### Requirement: Horario de las dos reuniones
Cada perfil SHALL guardar día + hora de **entre semana** y de **fin de semana**. Esos valores alimentan la cuenta atrás de pre-reunión y qué plantilla/`MeetingWeek` se considera “la de hoy”.

### Requirement: Cuenta atrás de pre-reunión en el auditorio
Antes de la hora configurada, el auditorio MAY mostrar un cronómetro de **inicio de reunión** (tiempo exacto que falta, o la plantilla de tiempos si el operador lo elige). Es independiente del HUD del orador. El perfil SHALL poder personalizar color, forma, tipografía y fondo de ese cronómetro.

El orador, en pre-reunión, sigue viendo solo HUD/negro, no ese diseño de auditorio salvo que el operador lo clone a propósito (v1: no se clona).

#### Scenario: Faltan 12 minutos
- GIVEN entre semana a las 19:00 y reloj 18:48
- WHEN la pre-reunión está activa
- THEN el auditorio muestra `12:00` (estilo del perfil)
- AND el orador no ve ese gráfico

### Requirement: A la hora de inicio
Al llegar el horario (o si el operador pulsa “iniciar reunión” antes):

1. El auditorio muestra el **texto del año** (cartel del perfil)
2. El **cántico inicial** de la guía de ese día, o el que el operador haya fijado a mano, queda **cargado en el stage** (ranura Inicio lista, archivo resuelto)
3. Play del cántico sigue siendo del operador — esta secuencia MUST NOT reproducir sola

La cadena de partes queda con la primera fila armada; Iniciar del cronómetro de partes es otro gesto.

#### Scenario: 19:00
- GIVEN guía con cántico 3 y texto del año definido
- WHEN llega las 19:00
- THEN el auditorio pasa del countdown al texto del año
- AND la ranura Inicio queda en 3, lista para Play
- AND no suena nada hasta que el operador pulse Play

### Requirement: El cronómetro de partes no lanza medios
Iniciar / Terminar / 0:00 / armar la siguiente parte MUST NOT abrir, pausar ni cerrar un recurso. Canciones y Multimedia son Play/Cerrar. La secuencia de **hora de inicio** solo carga el cántico en el stage y muestra el texto del año; no hace Play. Sin recurso abierto: auditorio = diario (después de la pre-reunión) o texto del año en el instante de inicio; orador nunca ve esos carteles.

### Requirement: Marcha en vivo
Cuenta atrás (o adelante, preferencia), hora de reloj, inicio/conclusión reales, desfase, inicio estimado de las siguientes.

#### Scenario: Parte de 10 minutos que se pasa 1:20 (avance off)
- GIVEN tesoro 10 min
- WHEN detiene a los 11:20
- THEN reales = 11:20, desfase = +1:20
- AND las siguientes recalcan inicio estimado

### Requirement: Mensaje al discursante
Mostrar/borrar un aviso en la ventana del orador, no en el auditorio.

### Requirement: Persistencia
Limpiar borra tiempos reales, no el esqueleto de la guía. Restaurar recarga el `MeetingWeek` de la fecha (no una plantilla vacía si hay guía). Fecha + perfil identifican el programa.

### Requirement: Plantillas de evento
Biblioteca por perfil: plantillas de **sistema** (no se borran; se duplican para editar) y de **usuario**. El operador aplica una a **una fecha**. No se autoelige. Avance en cadena igual. Canciones = números, himnario permanente.

#### Plantilla sistema: Entre semana (sin guía)
Usar si aún no hay `mwb`. Con guía, los minutos AYF/LC y títulos sustituyen los huecos.

| Orden | Parte | Min |
|---|---|---|
| 1 | Cántico inicial + oración | 5 |
| 2 | Palabras de introducción | 1 |
| 3 | Tesoros de la Biblia | 10 |
| 4 | Busquemos perlas escondidas | 10 |
| 5 | Lectura de la Biblia | 4 |
| 6–9 | Seamos mejores maestros (AYF 1–4) | 3 c/u (editable) |
| 10 | Cántico intermedio | 3 |
| 11–12 | Vida cristiana (1–2) | 15 / 0 (editable) |
| 13 | Estudio bíblico de congregación | 30 |
| 14 | Recapitulación / anuncios | 3 |
| 15 | Cántico final + oración | 5 |

Total de referencia ~1 h 45 min. Filas AYF de más o de menos según la semana.

#### Plantilla sistema: Fin de semana (sin guía)
| Orden | Parte | Min |
|---|---|---|
| 1 | Cántico inicial + oración | 5 |
| 2 | Discurso público | 30 |
| 3 | Cántico intermedio | 3 |
| 4 | Estudio de La Atalaya | 60 |
| 5 | Cántico final + oración | 5 |

Con `w` parseado: título del artículo en la fila 4; cánticos del número de la Atalaya.

#### Plantilla sistema: Visita de circuito — entre semana
Igual que la guía de esa semana **hasta** vida cristiana. **No** hay Estudio bíblico de congregación. En su lugar:

| Tras vida cristiana | Parte | Min |
|---|---|---|
| | Recapitulación, avance de la próxima semana, anuncios | 3 |
| | Presentación del superintendente | 1 |
| | Discurso de servicio (superintendente) | 30 |
| | Cántico final (elige el superintendente) + oración | 5 |

Duración total de referencia: 1 h 45 min. Sin clases auxiliares en el cronómetro. Los números de cántico inicial e intermedio siguen saliendo de la guía; el final queda editable (elección del superintendente).

#### Plantilla sistema: Visita de circuito — fin de semana
| Orden | Parte | Min |
|---|---|---|
| 1 | Cántico inicial + oración | 5 |
| 2 | Discurso público (superintendente) | 30 |
| 3 | Cántico intermedio | 3 |
| 4 | Estudio de La Atalaya | 60 |
| 5 | Cántico final + oración | 5 |
| 6 | Discurso de servicio extra (opcional, off) | 30 |

#### Plantillas de usuario
Duplicar cualquiera, renombrar, añadir/quitar/reordenar filas, cambiar minutos y color. Ejemplos: asamblea (sin reunión de congregación), Conmemoración. Quedan en el perfil.

#### Scenario: Semana de circuito
- GIVEN `MeetingWeek` del `mwb` y plantilla “Visita de circuito — entre semana”
- WHEN el operador la aplica a esa fecha
- THEN tesoro, perlas, lectura, AYF y vida cristiana vienen de la guía
- AND la fila CBS desaparece
- AND aparecen presentación + discurso 30 min + cántico final editable
- AND “cargar semana” restaura el plan completo del `mwb`

#### Scenario: Fin de semana de circuito
- GIVEN plantilla de circuito fin de semana
- WHEN se aplica
- THEN el discurso público figura como superintendente, 30 min
- AND el estudio usa el artículo `w` si está parseado

### Requirement: HUD en la superficie Orador
El estado del cronómetro (parte vigente, asignado, restante, desfase, barra 0–100 %) SHALL publicarse al `OutputPort` de orador en cada tick. Al avanzar de sección, el HUD cambia solo. Semántica de color:

- resto holgado → verde
- último tramo configurable (defecto: 20 % o 1 min, el que sea mayor) → ámbar
- 0:00 y desfase positivo → rojo

Umbrales y destinos de color: `specs/alertas`. El mensaje al discursante es otro canal. Ver `specs/ventanas`.

#### Scenario: Avance de parte
- GIVEN tesoro en rojo por desfase y el orador viendo el medio
- WHEN el operador (o el avance automático) pasa a gemas
- THEN el HUD pasa a gemas en verde con el nuevo cupo
- AND el fotograma de auditorio/orador no se resetea salvo que el operador cambie el medio
