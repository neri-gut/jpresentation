# Delta — Cronómetro (037)

## ADDED Requirements

### Requirement: Guía sin HTML usable
Si el `mwb`/`w` se abre pero `Document.Content` no es HTML (p. ej. blob cifrado), el sistema SHALL rellenar el cronómetro con la **plantilla de sistema** de esa reunión y superponer los números de cántico `sjjm` leídos de `Multimedia`. MUST NOT descifrar. Los medios embebidos siguen en la semana.

#### Scenario: Paquete de la semana con Content ilegible
- GIVEN un `mwb` de la semana en cache y Content no HTML
- WHEN el operador pulsa «Obtener guía»
- THEN la tabla muestra tesoro, perlas, lectura, AYF, vida cristiana y CBS con minutos de plantilla
- AND las filas de cántico llevan el track `sjjm` de esa semana si existe

### Requirement: Plantillas de usuario en la pestaña
El operador SHALL poder aplicar una plantilla de sistema (entre semana, fin de semana, visita de circuito) o una de **usuario** a la reunión abierta. SHALL poder añadir, modificar y borrar filas (título 1–80, minutos 1–180, tono) y guardar la tabla como plantilla de usuario. Las de sistema no se borran. Restaurar relee el programa cacheado de esa fecha.

#### Scenario: Guardar un evento
- GIVEN una tabla editada (p. ej. Conmemoración)
- WHEN guarda como plantilla de usuario «Memorial»
- THEN queda en el perfil
- AND aplicarla otro día sustituye las filas de esa reunión

#### Scenario: Visita de circuito entre semana
- GIVEN la guía o la plantilla entre semana cargada
- WHEN aplica «Visita de circuito — entre semana»
- THEN desaparece el Estudio bíblico de congregación
- AND aparecen recapitulación, presentación y discurso de servicio 30 min
