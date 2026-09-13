# Rendimiento y recursos

## Purpose

Que un PC de salón (2 núcleos, 8 GB, iGPU) lleve consola + auditorio + orador sin tirar fotogramas del medio. Trabajo pesado en Rust; la UI solo pinta.

## Requirements

### Requirement: Un decodificador de stage
Como máximo **un** vídeo a resolución de `video_quality` en el auditorio. El orador en `espejo+HUD` MUST reutilizar ese fotograma (o una copia reducida), no abrir un segundo decoder 720p/1080p. Páginas web: dos webviews (auditorio + orador) son el techo; la consola MAY hibernar el suyo si no está la pestaña Navegador.

### Requirement: Carga perezosa
Cada pestaña Vue se carga al primer uso. Himnario, miniaturas de la semana e índice de Biblia no se recorren al arrancar. Arranque: perfil + monitores + stage negro < presupuesto de “usable en 3 s” en SSD típico.

### Requirement: Listas y disco
Listas virtualizadas (himnario 163, cola de textos, carpeta local). Miniaturas: presupuesto de N en RAM, resto en cache por hash. Descargas: concurrencia acotada (`proveedor-medios`), prioridad al ítem que el operador acaba de pedir. SQLite WAL; consultas de lookup en el hilo de tokio, no en el de UI.

### Requirement: Degradar
Si el SO señala memoria baja o el decoder falla a `best`/`1080p`, SHALL bajar un escalón de calidad en esa sesión y avisarlo. Ambiente y webview de consola se pausan si no son visibles. Sin precucha = un grafo de audio.

### Requirement: Presupuesto de superficies
Consola visible siempre. Auditorio solo si hay monitor o preview. Orador solo si está on. Cerrar orador libera su webview/copia.
