# Proposal: Superficies — dueño, preview real y HUD a pantalla

## Intent

Tres fallos de la consola en `029`:

1. Auditorio y orador no están atados al operador: se quedan abiertos si se cierra la consola.
2. La opción «ventana de previsualización» no produce ventanas flotantes. Con un monitor (o con el desplegable en vacío) el auditorio cubre toda la pantalla — a veces encima de la consola — y el orador no llega a verse.
3. El HUD del orador es una pastilla pequeña. Sin medio en stage, el cronómetro MUST ocupar toda la superficie; con vídeo/imagen más adelante quedará compacto + barra inferior.

## Scope

In scope:

- Cerrar operador cierra auditorio, orador e identify; el proceso no queda huérfano.
- Preview de verdad: `audience_monitor_id = None` es flotante 960×540, **nunca** auto-fullscreen en la secundaria. Fullscreen solo en un monitor distinto al del operador y solo si hay ≥2 pantallas.
- Misma regla para el orador: no fullscreen sobre la consola. Preview visible, decorada, título propio, desplazada respecto al operador.
- Recrear la ventana nativa al cambiar de bordeado ↔ sin borde (GTK no conmuta decoraciones bien).
- HUD orador a pantalla completa mientras `stage.kind = none` (hoy siempre). Barra inferior a todo el ancho. El overlay compacto sobre vídeo queda cableado al `kind`, sin medios reales.

Out of scope:

- Reloj que cuenta, partes, desfase (`004` / `014`).
- Espejo de vídeo/imagen (`023` + multimedia).
- Panel de consola `022`, sync de snapshots extra.
- Hotplug de monitores.

## Approach

1. El operador es la ventana dueña (`parent` + `CloseRequested` / `Destroyed`).
2. La decisión preview vs cover vive en funciones puras de dominio (`audience_placement`, `speaker_placement`) con el id del monitor del operador.
3. Vue solo pinta el HUD; Rust sigue colocando las ventanas.
