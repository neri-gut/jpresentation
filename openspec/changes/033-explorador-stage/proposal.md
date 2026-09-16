# Proposal: Explorador Multimedia y stage; programa en el cronómetro

## Intent

`032` dejó el **programa de la semana** (partes, minutos, cánticos) en la pestaña Multimedia. Eso no es el producto: Multimedia es un **explorador de archivos** para previsualizar y **enviar al stage** (auditorio + orador) cualquier imagen, vídeo o medio extraído de un `.jwpub`. Los horarios de la guía viven en **Cronómetro**.

El catálogo, el ZIP y la cache `week/` de `032` se quedan. Cambia el sitio donde se pintan y se añade proyección.

## Scope

In scope:

- Multimedia: explorador (esta/próxima semana como **carpetas de ficheros** `img/`+`vid/`, raíces locales del perfil, recientes de sesión). Extensiones `jpg png webp gif mp4 webm jwpub`. Preview en consola. Doble clic / Abrir → stage.
- `.jwpub` local o de cache: extraer solo medios a `local-pub/{hash}/` y navegarlos como archivos. MUST NOT proyectar el zip entero. MUST NOT mostrar el outline de partes aquí.
- Stage: `image` | `video` | `none`. Copia a `media/stage/` y fan-out a auditorio y orador (espejo). `hud_only` = orador sigue en HUD. Cerrar vuelve a negro. Sin seek/zoom/PDF.
- Cronómetro: lista las partes del `MeetingWeek` (título, minutos); «Obtener guía» llama a `week_fetch`; pulsar una fila arma el reloj. MUST NOT abrir medios.
- Raíces locales persistidas por perfil (`explorer` settings). Añadir carpeta con diálogo nativo.

Out of scope:

- Cadena Terminar→siguiente, deshacer, plantillas (`004`).
- Himnario 1–163, PDF embebido, play/pausa/seek, un solo decoder 1080p (`023`).
- AES / claves. Barrido de meses. Raíz Record.

## Approach

1. Un change de corrección de `032` + el primer stage real.
2. Vue no hace fs: `explorer_list` / `stage_open` en Rust, con jail a raíces + `media_root`.
3. El outline `MeetingWeek` se pinta solo en Cronómetro.
