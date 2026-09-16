# Delta — Multimedia (033)

## MODIFIED Requirements

### Requirement: Explorador de archivos, no outline
La pestaña Multimedia SHALL ser un explorador de ficheros. Esta/próxima semana se muestran como **carpetas de imagen y vídeo** en cache (`img/`, `vid/`), no como partes del cronómetro. Raíces locales del perfil. `.jwpub` se inspecciona extrayendo medios. Doble clic o Abrir de una imagen o vídeo SHALL enviarlo al stage (auditorio y orador-espejo). El outline (títulos, minutos) MUST NOT pintarse aquí; vive en Cronómetro.

#### Scenario: Foto local al auditorio
- GIVEN una raíz `~/Congregacion` con `aviso.jpg`
- WHEN el operador abre ese fichero
- THEN el auditorio muestra la imagen
- AND el orador en modo espejo también
- AND el cronómetro no cambia
