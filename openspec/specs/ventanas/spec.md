# Ventanas y monitores

## Purpose

Gestionar la consola de operador, la salida de auditorio y el cronómetro del orador sobre escritorio extendido en v1. En un target móvil futuro las mismas superficies se resuelven vía `PlatformSurface` sin reescribir el dominio.

## Requirements

### Requirement: Tres roles de ventana
El sistema SHALL soportar:
1. **Operador**: ventana principal con pestañas, menú (Idiomas, Herramientas, Configuración) y panel de control derecho.
2. **Auditorio**: fullscreen en el monitor elegido, fondo negro, sin menú ni cursor persistente.
3. **Orador** (opcional): espejo del auditorio + HUD de tiempo siempre visible.

#### Scenario: Dos monitores
- GIVEN un PC con escritorio extendido de 2 pantallas
- WHEN el operador asigna “Auditorio” a la pantalla secundaria
- THEN esa pantalla pasa a fullscreen de salida
- AND la consola permanece en la primaria

#### Scenario: Un solo monitor
- GIVEN un equipo con un monitor
- WHEN no hay pantalla secundaria
- THEN la salida de auditorio puede abrirse como ventana previsualizable (Picture-in-Picture / ventana flotante)
- AND no se fuerza fullscreen sobre la consola

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

#### Scenario: Runtime móvil futuro
- GIVEN un target Android/iOS
- WHEN no existen monitores extendidos
- THEN `PlatformSurface` ofrece previsualización de audience dentro del operador
- AND el dominio de salida no cambia

### Requirement: Ventana del orador = auditorio + tiempo
La superficie Orador SHALL mostrar el **mismo fotograma** que el auditorio (imagen, vídeo, canción, texto, Biblia, negro). Encima, un HUD **compacto** que nunca se oculta:

- título de la parte vigente
- tiempo restante `mm:ss` mientras queda cupo
- si ya se pasó: indicador de desfase (`+1:20`) en color de alerta
- minutos asignados (secundario)
- barra inferior de progreso: verde al inicio → ámbar en el último tramo → rojo al cumplirse y en desfase

El HUD MUST ocupar poco (esquina o franja) para no tapar el medio. MUST contrastar sobre vídeo claro u oscuro (placa semitransparente). Si el auditorio está en negro / sin medio, el orador SHALL ver el cronómetro **a pantalla completa** (título, `mm:ss` grande, barra inferior a todo el ancho) sobre fondo oscuro. Cuando hay vídeo o imagen, el HUD compacto no tapa el medio y la barra inferior MUST seguir visible.

Los **mensajes** al discursante SHALL ir a una franja propia (arriba o sobre el medio) que MUST NOT tapar ni sustituir el HUD de tiempo. Cerrar el mensaje no apaga el cronómetro.

Esta ventana no tiene menú de operador ni cursor persistente.

#### Scenario: Vídeo en auditorio
- GIVEN el auditorio proyecta un vídeo de la guía
- WHEN hay monitor de orador
- THEN el orador ve el mismo vídeo, al mismo tiempo
- AND el restante y la barra siguen visibles encima

#### Scenario: Se pasa de tiempo
- GIVEN una parte de 10 min a 11:20
- WHEN el orador mira su pantalla
- THEN ve `+1:20` (o restante negativo) en rojo
- AND la barra está llena en rojo
- AND el medio del auditorio no se interrumpe

#### Scenario: Mensaje “1 minuto”
- GIVEN el HUD visible y un vídeo en curso
- WHEN el operador envía “1 minuto”
- THEN el aviso aparece en la franja de mensaje
- AND el cronómetro y la barra siguen a la vista
- AND el auditorio no muestra el texto

#### Scenario: Auditorio en texto diario
- GIVEN el auditorio mostrando el texto diario (sin recurso abierto)
- WHEN hay monitor de orador
- THEN el orador ve solo título + restante + barra sobre negro
- AND no ve el cartel diario

#### Scenario: Recurso en pausa
- GIVEN un vídeo pausado en auditorio
- WHEN el orador mira su pantalla
- THEN ve ese mismo fotograma pausado + HUD
- AND no ve el texto diario

#### Scenario: Sin monitor de orador
- GIVEN solo consola + auditorio
- WHEN no hay superficie Orador
- THEN el HUD vive en el panel de la consola
- AND el auditorio no recibe el overlay

### Requirement: Configurar monitor orador
En Configuración, por perfil:

- **Usar monitor orador**: on/off (defecto off si hay < 3 pantallas; on si hay una libre además de consola y auditorio)
- **Pantalla**: lista de monitores detectados (nombre, resolución). MUST NOT ser la misma que Auditorio. MAY ser la de consola solo como ventana flotante, no fullscreen encima de la UI
- **Modo**: `espejo+HUD` (`mirror`, defecto, si hay recurso abierto) | `solo HUD` (`hud_only`: siempre reloj sobre negro, aunque el auditorio tenga vídeo). Celdas antiguas sin el campo = `mirror`.
- **HUD**: opciones avanzadas (posición, escala, campos)
- **Arrancar con la app**: abrir la superficie orador al cargar el perfil
- **Identificar**: destello/nombre 2 s para saber qué pantalla es

Sin audio. Sin cursor persistente. Sin menú. Off = no se crea la ventana; el HUD queda en la consola.

Si el monitor asignado no está, se avisa y no se roba el auditorio.

`usar orador = off` → no hay ventana orador (el HUD vive en la consola). `usar orador = on` y `speaker_monitor_id` vacío, ausente o igual al del auditorio → ventana **flotante de previsualización** (decorada, no fullscreen sobre la consola). MUST NOT ocultarse. MUST NOT mover el auditorio. Si el id guardado ya no existe, la consola muestra `MonitorMissing`.

Identificar: el sistema MUST mostrar en **cada** pantalla detectada un destello con el nombre y la resolución durante **2 s**, y luego destruir esas ventanas. MUST NOT cambiar la colocación de auditorio ni orador.

#### Scenario: Orador sin pantalla asignada
- GIVEN `use_speaker = true` y `speaker_monitor_id` vacío
- WHEN se aplican las superficies
- THEN el orador abre como ventana flotante
- AND no se oculta
- AND no se pone fullscreen sobre la consola

#### Scenario: Monitor de orador desconectado
- GIVEN orador asignado al id `HDMI@0,0` y ese monitor ya no está
- WHEN arranca o se reaplican superficies
- THEN el orador pasa a previsualización
- AND el auditorio permanece donde estaba
- AND la consola muestra `MonitorMissing`

#### Scenario: Identificar
- GIVEN dos monitores
- WHEN el operador pulsa Identificar
- THEN cada pantalla muestra su nombre y `width×height` unos 2 s
- AND auditorio y orador siguen en su sitio al terminar

### Requirement: Opciones avanzadas del HUD
Todo por perfil. El tiempo (restante o desfase) MUST seguir visible; no hay modo “ocultar reloj”.

| Opción | Valores | Defecto |
|---|---|---|
| Posición | sup-izq, sup-der, inf-izq, inf-der, inf centrado | inf centrado |
| Escala | 75 / 100 / 125 / 150 % | 100 |
| Placa | opacidad 0–90 %, color | 55 %, negro |
| Contraste | auto (placa si el vídeo es claro) / fija | auto |
| Título de la parte | on/off | on |
| Minutos asignados | on/off | on |
| Siguiente parte (nombre) | on/off | off |
| Hora de reloj | on/off | off |
| Barra 0–100 % | on/off; arriba o abajo del bloque | on, abajo |
| Segundos | siempre / solo último minuto | siempre |
| Modo compacto | esconde asignado y título, deja `mm:ss` + barra | off |
| Colores holgado / aviso / desfase | enlazados a `specs/alertas` o override | alertas |

`espejo+HUD` y `solo HUD` comparten este bloque. Compacto no quita el restante. Mensajes al discursante siguen en franja aparte y no pisan el `mm:ss`.

#### Scenario: Vídeo claro
- GIVEN contraste auto y un clip muy blanco
- WHEN el HUD está encima
- THEN la placa se oscurece
- AND el `mm:ss` se lee

#### Scenario: Compacto
- GIVEN modo compacto on
- WHEN hay parte en marcha
- THEN el orador ve sobre todo el tiempo y la barra
- AND no desaparece al cambiar el medio

#### Scenario: Tres pantallas
- GIVEN consola, proyector y un monitor al atril
- WHEN asigna Orador al atril y modo espejo+HUD
- THEN el atril muestra el medio del salón + tiempo
- AND el proyector no lleva overlay

#### Scenario: Orador desactivado
- GIVEN `usar orador = off`
- WHEN inicia la reunión
- THEN no hay tercera ventana
- AND el tiempo se ve en el panel

### Requirement: Lista de monitores etiquetada
El selector SHALL marcar la pantalla de la consola («console») y la primaria del SO («primary»). La opción de id vacío SHALL llamarse ventana de previsualización, sin implicar que solo existe un monitor.

#### Scenario: Dos monitores, preview
- GIVEN escritorio extendido y Auditorio en preview
- WHEN el operador abre Configuración
- THEN ve «Preview window» como opción vacía
- AND el display de la consola figura como console

### Requirement: Detección y persistencia de monitores
El sistema SHALL enumerar monitores al arrancar y al conectar/desconectar pantallas. La asignación operador/auditorio/orador SHALL persistirse por perfil. Si un monitor desaparece, la ventana asociada SHALL moverse a la primaria sin perder el medio actual.

#### Scenario: Desconexión de proyector
- GIVEN el auditorio en el proyector
- WHEN el proyector se apaga
- THEN la ventana de salida se reubica o se oculta de forma controlada
- AND al reconectar el mismo monitor, se restaura la asignación si sigue disponible

### Requirement: Panel de control siempre visible
Ver `specs/panel`. Columna derecha persistente: stage, audio, cronómetro, Biblia, mensajes al orador. Sin Grabador. MAY colapsarse.
