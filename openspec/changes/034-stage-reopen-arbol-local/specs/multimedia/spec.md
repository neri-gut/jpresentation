# Delta — Multimedia (034)

## ADDED Requirements

### Requirement: Reemplazar el medio en stage
Mostrar un segundo fichero MUST funcionar sin reiniciar ni desactivar el orador. Cada apertura SHALL usar un path nuevo bajo `media/stage/`. Cerrar MUST soltar el recurso en auditorio y orador antes de borrar el fichero.

#### Scenario: Segunda imagen
- GIVEN una imagen en las pantallas
- WHEN el operador Cierra, elige otra y pulsa Show
- THEN el auditorio muestra la nueva
- AND no hace falta desactivar el monitor orador

### Requirement: Árbol de carpetas locales
El árbol izquierdo SHALL incluir Home y Desktop (si existen) además de Esta/Próxima semana y las raíces del perfil. Las subcarpetas MUST poder expandirse. Añadir carpeta SHALL abrir un diálogo nativo del SO.

#### Scenario: Foto en el escritorio
- GIVEN un jpg en Desktop
- WHEN el operador abre Desktop en el árbol
- THEN ve la miniatura y puede cuearla al panel
