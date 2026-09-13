# Alertas visuales

## Purpose

Avisos de tiempo y de estado **solo visuales**, configurables por perfil. Van a consola y, si se activa, al orador. El auditorio no parpadea ni muestra banners de alerta salvo un interruptor explícito (defecto: off).

## Requirements

### Requirement: Catálogo de alertas
El perfil SHALL poder activar, umbral, color y destino de cada alerta:

| Id | Cuándo | Defecto |
|---|---|---|
| `pre.start` | Faltan N min para la hora de reunión | off, 5 min, consola |
| `part.warn` | Queda el último tramo de la parte | on, máx(20 %, 1 min), consola + orador, ámbar |
| `part.over` | 0:00 y desfase positivo | on, consola + orador, rojo |
| `part.armed` | La siguiente parte quedó armada | on, consola |
| `media.ended` | Vídeo/cántico llegó a fin | on, consola |
| `meet.start` | Llegó la hora de inicio | on, consola |

Sin sonido en v1 (el `AudioPort` no se usa para alertas).

### Requirement: Dónde se ven
- **Consola:** badge o franja en el panel de cronómetro; MAY parpadear suave.
- **Orador:** cambia color del HUD / barra (ya definido). Un destello MUST NOT tapar el restante ni el medio. Mensajes de texto al discursante siguen siendo otro canal.
- **Auditorio:** off. Si el perfil lo enciende, solo un recuadro discreto; nunca sustituye el medio ni el texto diario.

### Requirement: Apariencia
Por alerta: color, intensidad, `solid` | `pulse`. El estilo del countdown de pre-reunión (forma del reloj grande) es independiente; `pre.start` solo avisa en consola de que ya falta poco.

Cambiar umbrales no exige recompilar. Valores viven en el perfil.

#### Scenario: Último minuto
- GIVEN tesoro 10 min y `part.warn` = 1 min
- WHEN restante = 1:00
- THEN HUD orador pasa a ámbar
- AND la consola marca aviso
- AND el auditorio no cambia

#### Scenario: Se pasan
- GIVEN `part.over` on
- WHEN desfase > 0
- THEN HUD rojo
- AND la consola lo indica
- AND el medio en auditorio sigue
