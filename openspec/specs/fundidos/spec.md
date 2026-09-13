# Fundidos, zoom de imagen y PDF

## Purpose

Tres controles de **presentación**, no de Zoom.us ni de red.

## Requirements

### Requirement: Fundido = transición de volumen o de imagen
No es un delay de red. Son milisegundos que el operador oye/ve:

| Fundido | Qué hace | Defecto | Rango |
|---------|----------|---------|--------|
| Ambiente → cántico | Baja el ambiente al pulsar Play de himnario | **1200 ms** out | 0–3000 |
| Ambiente al reanudar | Sube el ambiente si vuelve a Play ambiente | **800 ms** in | 0–3000 |
| Imagen → imagen | Crossfade al Anterior/Siguiente de fotos | **400 ms** | 0–1500 (`0` = corte) |
| Cerrar recurso | Paso a texto diario | **0 ms** (corte) | — |
| Vídeo / cántico | Empiezan en corte, sin fade de imagen | 0 | — |

Valores por perfil. 0 = instantáneo.

### Requirement: Zoom de imagen = ampliar en el stage
El botón Zoom del panel **aumenta o encuadra la foto/PDF** en auditorio y orador. No tiene relación con la app Zoom.

- Ajustes: `fit` (cabe entera), `fill` (llena y recorta), `100/125/150/200 %`
- Defecto al abrir: `fit`
- Pan si hay overflow (arrastre en **consola**, no en orador)
- No aplica a vídeo en play (el vídeo usa su frame)

### Requirement: PDF por defecto
Cuando el explorador local o la guía abren un `.pdf`:

- Defecto de perfil: **`embedded`** — visor en el stage (páginas; solo el operador pagina, igual que Biblia)
- Alternativa: **`system`** — el visor del SO; el stage no muestra el PDF

Casos: aviso de limpieza, programa impreso, folleto en USB. Un PDF dentro de un `.jwpub` extraído se trata igual. No es la Biblia (eso es texto paginado de `BibleSource`).
