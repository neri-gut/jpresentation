# Delta — Cronómetro (033)

## ADDED Requirements

### Requirement: Outline de la guía en Cronómetro
Cuando existe un `MeetingWeek` persistido, la pestaña Cronómetro SHALL listar las partes (título, minutos) de entre semana y fin de semana. «Obtener guía» SHALL disparar el fetch de `032`. Pulsar una fila SHALL armar el reloj con ese título y minutos, sin abrir medios.

#### Scenario: Guía ya en cache
- GIVEN `mwb` parseado para esta semana
- WHEN abre Cronómetro
- THEN ve tesoro, gemas, etc. con minutos
- AND al pulsar tesoro el HUD arma 10:00
- AND Multimedia no muestra esas filas de programa
