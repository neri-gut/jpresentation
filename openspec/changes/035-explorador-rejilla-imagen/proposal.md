# Proposal: Rejilla del explorador e imagen del auditorio

## Intent

El explorador de `034` funciona, pero se usa como lista de filas (thumb + nombre). Es poco denso y difícil de ojear. En el auditorio las imágenes **cuadradas se deforman**; en el orador se ven bien (el img está `position: absolute` + `object-fit`).

## Scope

In scope:

- Multimedia: rejilla de miniaturas (carpetas, vídeos, imágenes), migas clicables, filtro por nombre, Places a la izquierda. Sin campo de ruta pegada como flujo principal.
- Auditorio: la imagen MUST conservar proporción (letterbox en negro). Nunca estirar al 16:9.

Out of scope: cronómetro, PDF, himnario, seek, Rec.
