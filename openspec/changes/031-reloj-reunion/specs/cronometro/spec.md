# Delta — Cronómetro (031)

## ADDED Requirements

### Requirement: Una parte suelta sin guía
Hasta que un change posterior cargue un `MeetingWeek`, el operador SHALL poder armar **una** parte (título 1–80, minutos 1–180), iniciarla, pausarla y terminarla. El tick MUST vivir en Rust (~4 Hz). Vue MUST NOT ser dueña de un `setInterval` de cuenta. Iniciar / Pausar / Terminar MUST NOT abrir, pausar ni cerrar un recurso de stage.

Pausar SHALL dejar la parte `armed` con el transcurrido. Terminar SHALL pasar a `idle` (no arma la siguiente). A 0:00 la misma parte sigue y el desfase crece (`auto_advance_on_zero` no aplica en este change).

#### Scenario: Tesoro de 10 minutos
- GIVEN el reloj idle
- WHEN el operador arma «Tesoro» 10 min e Iniciar
- THEN orador y panel muestran restante `10:00` que decrece
- AND el auditorio no cambia de medio

#### Scenario: Pausa y retoma
- GIVEN una parte running a 4:00 transcurridos de 10
- WHEN pausa y luego Iniciar
- THEN el restante sigue desde 6:00
- AND no se resetea el asignado

#### Scenario: Se pasan
- GIVEN 10 min y transcurridos 11:20
- WHEN el orador mira el HUD
- THEN ve `+1:20` en rojo
- AND la barra está al 100 %
- AND Terminar es lo que deja idle

#### Scenario: Start sin armar
- GIVEN idle
- WHEN pulsa Iniciar
- THEN Rust responde `ClockNotArmed`
- AND el auditorio no se cierra
