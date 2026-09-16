# Proposal: Contain a pantalla y plantillas de cronómetro

## Intent

El `cover` de `036` recorta las fotos cuadradas. El operador quiere **llenar el monitor sin recortar ningún borde**: si la proporción coincide, la imagen ocupa toda la pantalla; si no (cuadrada, 4:3), hay franjas negras. MUST NOT dejar las 16:9 de sala como una estampilla.

«Obtener guía» descarga el `mwb`/`w` pero la tabla queda vacía: `Document.Content` no es HTML legible y **no se descifra**. Hay que armar el programa con la plantilla de sistema + cánticos `sjjm` del paquete. El operador MUST poder aplicar plantillas de sistema (entre semana, fin de semana, visita de circuito) y **crear las suyas** (asamblea, Conmemoración, evento).

## Scope

In scope:

- Auditorio y orador-espejo: imagen `contain` a caja completa (escala arriba y abajo, nunca recorte, nunca estirar). Vídeo sigue `contain`.
- Si el parser HTML no saca filas, rellenar con plantilla de sistema de ese tipo de reunión y superponer números de cántico desde `Multimedia`.
- Biblioteca de plantillas: 4 de sistema (no se borran) + CRUD de usuario por perfil. Aplicar a la fecha/reunión abierta. Añadir / modificar / borrar filas; guardar la tabla actual como plantilla de usuario. Restaurar relee el `MeetingWeek` cacheado.
- Cronómetro: botón Visita de circuito, selector de plantilla, editor de fila (tono, título, minutos).

Out of scope: AES / claves maestras, historial Real/Hora, himnario MP4, PDF, columnas de desfase.

## Specs

- `ventanas` — contain a pantalla
- `cronometro` — fallback de guía + plantillas de usuario
