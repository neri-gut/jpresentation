# Proposal: Cover en auditorio y cronómetro de la guía

## Intent

Las imágenes de calidad para pantalla se ven pequeñas (letterbox). El operador quiere **cubrir** el monitor conservando proporción (se recorta si hace falta).

El cronómetro aún no es la tabla de JMulti-02: hay que extraer **nombres y minutos** de la guía y mandarlos en orden al reloj del panel (Terminar arma la siguiente).

## Scope

In scope:

- Auditorio y orador-espejo: imagen a pantalla (`cover`), sin estirar.
- Parser de `Document.Content` HTML: filas con `(N min)` + cánticos, en orden. `tone` tesoro/AYF/vida para color de fila.
- Pestaña Cronómetro: tabla Tema | Min. Clic arma. Panel: parte vigente, Iniciar/Pausar/Terminar; Terminar arma la siguiente. MUST NOT abrir medios.

Out of scope: historial Real/Hora inicio, deshacer, plantillas de circuito, PDF, himnario MP4.
