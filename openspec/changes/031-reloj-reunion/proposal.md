# Proposal: Reloj de reunión, HUD vivo y resto de pantallas

## Intent

El HUD de `030` ocupa la ventana del orador, pero el tiempo es un `00:00` falso. El panel derecho no controla nada. En Configuración faltan el modo del orador (`espejo+HUD` / `solo HUD`) y etiquetas claras de qué pantalla es la consola.

Este change deja un `MeetingClock` en Rust (una parte suelta) que pintan orador y panel, sin guía JWPUB y sin lanzar medios.

## Scope

In scope:

- Reloj de dominio: `idle` / `armed` / `running`. Armar (título + minutos), Iniciar, Pausar, Terminar. Tick en Rust (~4 Hz). `ClockSnapshot` con restante, desfase, barra 0–100 %, hue verde→ámbar→rojo.
- HUD del orador: `mm:ss` real, `+m:ss` en desfase, barra inferior. Si el orador está off, los mismos números viven en el bloque Cronómetro del panel.
- Bloque Cronómetro del panel (no toda la columna `022`): mismos gestos. La pestaña Cronómetro reutiliza los controles.
- Superficies: `speaker_mode` `mirror` | `hud_only`. Lista de monitores: marcar consola y primaria. Preview no dice «un solo monitor».

Out of scope:

- Cadena de partes, deshacer, plantillas, `MeetingWeek` / JWPUB (`004`).
- Countdown de pre-reunión, texto del año (`014`).
- Mensajes al discursante, Stage / Audio / Biblia (`022`).
- Vídeo en stage, sync de decoder (`023`).
- HUD avanzado: posición, escala, placa, compacto (`018`).
- Hotplug de monitores.

## Approach

1. Un change de implementación. El reloj es dueño en Rust; Vue solo pinta el último snapshot y despacha comandos.
2. Pausar vuelve a `armed` con el transcurrido; Terminar deja `idle` (no arma la siguiente).
3. Umbral ámbar = `max(20 % del asignado, 1 min)` (`specs/alertas`). Desfase = rojo. `auto_advance_on_zero` no entra: a 0:00 la parte sigue y el desfase crece.
4. `speaker_mode` se persiste en la celda `surfaces` (serde default `mirror` para celdas viejas). Sin medio, ambos modos se ven igual; `hud_only` ya fuerza HUD completo si más adelante hay `kind ≠ none`.
