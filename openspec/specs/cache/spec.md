# Cache y disco

## Purpose

No volver a bajar lo que ya está bien, y no llenar el disco. Hay **cuatro almacenes**, no un “cache mágico” de Vue.

| Almacén | Qué | ¿Se borra solo? |
|---------|-----|-----------------|
| `hymnal/{lang}/` | MP4 1–163 | No (permanente) |
| `bible/{lang}/` | `.jwpub` de Biblia + índice SQLite | No |
| `week/{lang}/{yyyy-mm-dd}/` | Guía + imágenes/vídeos de esa semana | Sí, al quedar vieja |
| `thumb/{hash}` | Miniaturas | Sí, LRU si pasa el tope |

Además: JSON de catálogo de idiomas (TTL) y el cache HTTP del webview (lo gestiona el motor).

## Requirements

### Requirement: Acierto = no hay red
Antes de GETPUBMEDIALINKS + descarga, el repo mira checksum (o tamaño+mtime si no hay hash). Si el fichero cuadra, el ítem queda `ready`. El rendimiento sale de **no decodificar/bajar dos veces**, no de cachear componentes Vue.

### Requirement: Semana viva
Se conservan **esta semana y la próxima** del perfil. Al pasar a una semana nueva, `week/` de hace ≥ 2 semanas se puede purgar (el operador confirma si hay > N MB). Himnario y Biblia no entran en esa purga. Arrancar la app MUST NOT descargar la semana; el operador pulsa Actualizar. JWPUB, jpeg extraídos y MP4 de guía viven en `week/{langwritten}/{yyyy-mm-dd}/`.

Tope orientativo de `week/` + `thumb/`: 4 GB por perfil (configurable). Al 90 %: aviso, no se apaga el stage.

### Requirement: Miniaturas
Se generan en Rust fuera del UI thread. Clave = hash del binario. Si falta el thumb, se muestra placeholder; no bloquea Play.

### Requirement: Fallo de descarga (manual)
Sin reintento automático. Si un lote o un fichero falla (red, 404, checksum):

1. Pantalla/diálogo de error: qué faltó  
2. El operador elige **Reintentar** (solo esos ítems) o **Continuar sin ellos**  
3. Los que sí bajaron se quedan  
4. El auditorio no se toca  

#### Scenario: Se cae la red a mitad del himnario
- GIVEN 40 de 163 tracks
- WHEN falla el 41
- THEN diálogo con el pendiente
- AND Reintentar pide desde el 41
- AND Continuar deja 40 listas y el resto `pending`
