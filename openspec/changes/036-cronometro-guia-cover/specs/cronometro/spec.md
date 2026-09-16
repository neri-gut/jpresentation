# Delta — Cronómetro (036)

## ADDED Requirements

### Requirement: Tabla de la guía en Cronómetro
Tras un fetch, la pestaña SHALL listar las partes en orden (título, minutos). Clic SHALL armar esa parte. Terminar SHALL armar la **siguiente** (sin iniciar la cuenta, sin medios). El panel derecho SHALL mostrar la parte vigente y los mismos gestos.

#### Scenario: Tesoro luego perlas
- GIVEN tesoro 10 min en marcha y perlas 10 min a continuación
- WHEN el operador pulsa Terminar
- THEN el HUD arma perlas 10:00 detenido
- AND Iniciar hace falta para contar
