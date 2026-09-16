# Delta — Multimedia (032)

## ADDED Requirements

### Requirement: Explorador de la guía en la pestaña Multimedia
La pestaña Multimedia SHALL mostrar un árbol de **esta semana** y **próxima semana** (idioma `content_locale` del perfil), agrupado por reunión (entre semana / fin de semana) y por parte del `MeetingWeek` cuando exista. Si el parser no obtuvo partes, SHALL listar los medios de ese documento en orden de párrafo. Cada ítem MUST mostrar estado: embebido / listo / pendiente / himnario / fallido.

Un panel de previsualización SHALL pintar la imagen seleccionada desde cache. Vídeo y cántico muestran ficha (título, clave, estado), no un decoder. Doble clic MUST NOT enviar al stage en este change. MUST NOT haber raíz Record. MUST NOT disparar descargas al abrir la pestaña; el operador pulsa Actualizar o Descargar.

#### Scenario: Semana con ilustraciones y un vídeo
- GIVEN `mwb` parseado con 2 jpeg embebidos y 1 `mwbv` pendiente
- WHEN el operador abre Multimedia → Esta semana
- THEN ve carpetas de partes (o Media) con las 2 imágenes listas y el vídeo pendiente
- AND al seleccionar una imagen, el preview la muestra
- AND el auditorio sigue negro

#### Scenario: Cántico de la guía
- GIVEN una fila `sjjm` track 1
- WHEN lista la semana
- THEN el ítem figura como enlace de himnario (`pending_hymnal`)
- AND no se copia un MP4 a `week/`
