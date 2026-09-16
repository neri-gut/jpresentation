# Delta — Ventanas (030)

## ADDED Requirements

### Requirement: El operador es dueño del proceso
Cerrar la ventana Operador MUST destruir Auditorio, Orador e identify y dejar el proceso sin webviews huérfanos. Cerrar solo Auditorio u Orador MUST NOT cerrar la consola.

#### Scenario: Cierra la consola
- GIVEN operador, auditorio y orador abiertos
- WHEN el operador cierra la consola
- THEN no quedan ventanas de JPresentation
- AND el proceso no sigue solo con el auditorio negro

### Requirement: Preview nunca cubre la consola
`audience_monitor_id` vacío SHALL abrir una ventana flotante decorada (≈960×540), aunque haya un segundo monitor. MUST NOT auto-asignar la secundaria. Fullscreen (cover del monitor, sin exclusive fullscreen del SO) solo si el id existe, hay ≥2 pantallas y **no** es el monitor donde está el operador. La misma regla aplica al orador: monitor del operador o único display → flotante visible, nunca encima a pantalla completa de la UI.

#### Scenario: Un monitor, previsualizar
- GIVEN un equipo con una sola pantalla
- WHEN el operador deja Auditorio en «ventana de previsualización» y activa Orador sin pantalla dedicada
- THEN aparecen dos ventanas flotantes (auditorio y orador) junto a la consola
- AND ninguna se pone fullscreen sobre el operador

#### Scenario: Dos monitores, preview explícito
- GIVEN escritorio extendido
- WHEN Auditorio está en «ventana de previsualización» (id vacío)
- THEN el auditorio es flotante en el monitor del operador
- AND la secundaria no se cubre sola

#### Scenario: Dos monitores, proyector asignado
- GIVEN consola en la primaria y Auditorio asignado a la secundaria
- WHEN se aplican las superficies
- THEN la secundaria queda cubierta por el auditorio
- AND la consola permanece usable en la primaria

### Requirement: HUD a pantalla cuando no hay medio
Si el stage no tiene recurso (`kind = none` u holgura equivalente: negro / reposo), la superficie Orador SHALL pintar el cronómetro a **pantalla completa** (título, `mm:ss` grande, barra inferior a todo el ancho). Cuando haya vídeo o imagen, el HUD compacto no tapará el medio y la barra inferior MUST seguir visible. El auditorio no muestra este overlay.

#### Scenario: Orador sin medio
- GIVEN orador abierto y stage `none`
- WHEN el discursante mira su ventana
- THEN el tiempo ocupa la superficie
- AND hay una barra inferior a todo el ancho
