# Navegador web

## Purpose

Abrir una página en la consola y enviarla al auditorio **sin guardar un archivo** en `week/` ni en el himnario. Webviews aislados. Sin login automático ni scraping.

## Requirements

### Requirement: Navegación en consola
Barra de dirección, atrás/adelante/recargar, favoritos del perfil. Inicio: página en blanco o el favorito de inicio. Solo `https:` (y `http:` en LAN si el operador lo pega). La consola navega a lo que el operador escriba; no hay rastreador ni lista oculta de sitios.

### Requirement: Dos formas de stage (sin fichero)
1. **Mostrar página** — el auditorio abre **esa URL** en un webview kiosco (fullscreen, sin barra, sin DevTools, sin descargas, sin ventanas nuevas). No se copia la sesión de la consola (cookies aparte).
2. **Reproducir URL de medio** — enlace directo `.mp4`, `.webm`, `.jpg`, `.png`, `.webp` o `.m3u8` público. El player de `OutputPort` hace stream. No entra en `MediaCache`. HLS: MAY; si el player no puede, se informa y se ofrece mostrar la página.

Detectar un `<video src="https://…">` en la consola es **best-effort** (MAY). Si no hay `src` http(s) de fichero (blob, MSE, DRM, player de jw.org/YouTube), solo cabe **mostrar página**. MUST NOT extraer streams internos.

Cerrar: solo el operador. Restaura el reposo (texto diario en auditorio).  
Pausa: solo en el player propio (modo 2). En modo página no hay pausa fiable → solo cerrar.  
El cronómetro no envía URLs.

#### Scenario: Artículo
- GIVEN un artículo en la consola
- WHEN “mostrar página”
- THEN el auditorio carga esa URL
- AND no hay archivo nuevo en `week/`

#### Scenario: MP4 directo
- GIVEN `https://ejemplo/file.mp4`
- WHEN “reproducir en auditorio”
- THEN stream en el player de stage
- AND al cerrar no se archiva a propósito

#### Scenario: Player embebido
- GIVEN YouTube o el player de jw.org
- WHEN no hay `src` de fichero
- THEN la UI ofrece solo “mostrar página”
- AND no intenta ripping

### Requirement: Orador
En `espejo+HUD` con **página**: el orador carga la **misma URL** en otro webview aislado + HUD. Pueden ir un instante desfasados (dos cargas). No se comparte cookie con la consola.  
En `solo HUD`, o si la doble carga falla: orador = reloj sobre negro.  
Con **URL de medio** (modo 2): mismo player/fotograma que el auditorio + HUD.

### Requirement: Audio
El sonido de la página o del stream usa el dispositivo Auditorio y el máster. Canal interno `browser` (slider futuro). Sin precucha.

### Requirement: Aislamiento
Webview de auditorio y de orador: sin comandos de fs, SQL, descarga ni grabación. Bloquear `window.open`, archivos y plugin de captura. Allowlist de stage = la URL que el operador envió (navegación in-page del mismo sitio permitida; no saltar a un host que el operador no haya mandado, o pedir confirmación en consola).

MUST NOT automatizar login ni reupload.

### Requirement: Favoritos
Por perfil. Acción: abrir en consola o enviar al auditorio.
