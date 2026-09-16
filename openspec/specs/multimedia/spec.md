# Multimedia

## Purpose

Dos orígenes en la misma pestaña: **medios de la guía** (esta / próxima semana) y un **explorador local** (carpetas + `.jwpub` / `.epub`) para recursos propios. Solo el operador abre, pagina y cierra. El stage no descarga por su cuenta al pasear.

## Requirements

### Requirement: Árbol de orígenes
Raíces:

- Esta Semana / Próxima Semana — agrupadas por parte del `MeetingWeek`
- Multimedia por fecha (cache semanal ya bajada)
- **Local** — raíces que el perfil declare (más Escritorio si el SO lo permite)
- Recientes

Sin raíz Record. Cada ítem de guía marca embebido / en cache / pendiente. Los cánticos de la guía son enlaces al himnario, no copias.

### Requirement: Guía de la semana
Igual que antes: parsear `mwb`/`w`, bajar solo imágenes/vídeos de esa semana a `video_quality`. No himnario aquí. No barrer semanas futuras salvo “Próxima Semana”.

La pestaña Multimedia SHALL ser un **explorador de ficheros**. Esta/próxima semana se muestran como carpetas de imagen y vídeo en `week/` (`img/`, `vid/`), no como partes del programa. El árbol izquierdo SHALL incluir Home y Desktop (si existen) y subcarpetas expandibles. Seleccionar un fichero lo deja en el panel; Show proyecta. Reemplazar el medio MUST usar un path nuevo en `media/stage/` sin desactivar el orador. El outline vive en Cronómetro. Abrir la pestaña MUST NOT disparar HTTP.

### Requirement: Explorador local
Lista de carpeta bajo las raíces del perfil (añadir/quitar raíz en Configuración). Extensiones de v1: `jpg png webp gif mp4 webm mp3 pdf jwpub epub`. Paseo perezoso (no indexar el disco al arrancar). Virtualizar si hay cientos de ficheros.

Doble clic:

- imagen / vídeo / audio → stage (player o cartel), **sin copiar** a `week/` salvo que el operador pulse “añadir a esta semana”
- PDF → visor embebido o del SO (preferencia de perfil)
- `.jwpub` / `.epub` → **inspeccionar** (no proyectar el zip entero)

“Abrir en el sistema” sigue disponible.

#### Scenario: Foto propia
- GIVEN `~/Congregacion/avisos/limpieza.jpg` bajo una raíz
- WHEN Abrir
- THEN el auditorio muestra esa imagen
- AND no se duplica en la cache de la guía

### Requirement: Inspeccionar JWPUB local
Al abrir un `.jwpub` (o `.epub`) local, `ScheduleParser` / el lector de paquete SHALL listar:

- título y símbolo de la publicación (`manifest`)
- documentos (nombre)
- filas `Multimedia`: embebido (`FilePath`) vs catálogo (`KeySymbol` + `Track` + issue)

El operador explora ese árbol. Un embebido se extrae a cache local de publicación (`local-pub/{hash}/`) y se puede enviar al stage. Una referencia de catálogo se resuelve con GETPUBMEDIALINKS **solo si** pulsa descargar; si no, queda pendiente.

Textos de documento MAY enviarse al flujo de carteles / cola de Biblia si el extractor saca cita o párrafo. MUST NOT volcar el HTML crudo al auditorio.

El mismo inspector sirve para un `mwb` o `w` abierto a mano (además de “Esta Semana”).

#### Scenario: Folleto en el USB
- GIVEN un `.jwpub` de folleto en una raíz local
- WHEN el operador lo abre y elige una imagen embebida
- THEN esa imagen va al stage
- AND el resto del paquete no se descarga del CDN

#### Scenario: Vídeo solo en catálogo
- GIVEN una fila `mwbv` sin `FilePath`
- WHEN pulsa descargar
- THEN entra el MP4 a calidad del perfil
- AND luego Abrir lo proyecta

### Requirement: Proyección
Abrir / anterior / siguiente / recargar / zoom de imagen (`specs/fundidos`). Vídeo: play/pausa/seek; volumen = máster. Siguiente recorre el contexto. PDF: `embedded` o `system`. Solo el operador.

### Requirement: Grupos
SHOULD agrupar collages de la guía (interruptor “Mostrar grupos”).

### Requirement: Importar a la semana
Diálogo nativo o “añadir a esta semana” copia o enlaza un fichero local como complemento de esa fecha, sin meterse en el himnario.
