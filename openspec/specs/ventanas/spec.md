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

El HUD MUST ocupar poco (esquina o franja) para no tapar el medio. MUST contrastar sobre vídeo claro u oscuro (placa semitransparente). Si el auditorio está en negro / sin medio, el orador sigue viendo el cronómetro sobre fondo oscuro.

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
- **Modo**: `espejo+HUD` (defecto, si hay recurso abierto) | `solo HUD` (siempre reloj sobre negro, aunque el auditorio tenga vídeo)
- **HUD**: opciones avanzadas (posición, escala, campos)
- **Arrancar con la app**: abrir la superficie orador al cargar el perfil
- **Identificar**: destello/nombre 2 s para saber qué pantalla es

Sin audio. Sin cursor persistente. Sin menú. Off = no se crea la ventana; el HUD queda en la consola.

Si el monitor asignado no está, se avisa y no se roba el auditorio.

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

### Requirement: Detección y persistencia de monitores
El sistema SHALL enumerar monitores al arrancar y al conectar/desconectar pantallas. La asignación operador/auditorio/orador SHALL persistirse por perfil. Si un monitor desaparece, la ventana asociada SHALL moverse a la primaria sin perder el medio actual.

#### Scenario: Desconexión de proyector
- GIVEN el auditorio en el proyector
- WHEN el proyector se apaga
- THEN la ventana de salida se reubica o se oculta de forma controlada
- AND al reconectar el mismo monitor, se restaura la asignación si sigue disponible

### Requirement: Panel de control siempre visible
Ver `specs/panel`. Columna derecha persistente: stage, audio, cronómetro, Biblia, mensajes al orador. Sin Grabador. MAY colapsarse.
