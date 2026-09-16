# Delta — Programa semanal (032)

## ADDED Requirements

### Requirement: Esta y próxima semana a demanda
El operador SHALL poder pedir Esta semana y Próxima semana. El sistema SHALL localizar `mwb` y `w` del periodo, descargar el JWPUB si falta, parsear y persistir un `MeetingWeek` por perfil + lunes ISO + kind (`midweek` | `weekend`). MUST NOT barrer meses ni descargar al arrancar.

Si `Document.Content` no es HTML legible, el `MeetingWeek` MUST existir igual con los medios de `Multimedia`/`DatedText` y partes genéricas. MUST NOT descifrar el HTML.

#### Scenario: Primera vez en el mes
- GIVEN cache vacía y red
- WHEN pulsa Actualizar esta semana
- THEN se obtiene `mwb`+`w` del issue, se parsea y Multimedia lista la semana
- AND un 404 de issue no publicado se muestra y no entra en bucle
