# 🗄️ 03. Catálogo del Esquema SQLite del JWPUB

La base de datos SQLite embebida en el archivo `.jwpub` contiene tablas relacionales con la estructura del contenido, índices de búsqueda, multimedia y referencias bíblicas.

---

## 📊 1. Tablas Principales y Propósito

### 1.1 `Publication` (Metadatos de la Publicación)
Contiene la información de identificación de la edición:
- `PublicationId`: Identificador interno de la publicación.
- `MepsLanguageIndex`: Índice del idioma en el sistema MEPS (ej. `0` para inglés, `1` para español).
- `Symbol`: Símbolo de la publicación (ej. `mwb25`, `w25`).
- `Year`: Año de publicación (ej. `2025`).
- `IssueTagNumber`: Número de edición (ej. `20250900` para septiembre de 2025).
- `Title`: Título completo de la publicación.
- `ShortTitle`: Título corto.

---

### 1.2 `Document` (Artículos y Secciones)
Almacena cada documento, artículo o semana de reunión:
- `DocumentId`: ID único incremental.
- `MepsDocumentId`: ID del documento en MEPS (utilizado en enlaces inter-documento).
- `Class`: Clase del documento:
  - `39`: Portada / Cubierta.
  - `40`: Artículo de estudio de La Atalaya.
  - `68`: Tabla de contenidos (TOC).
  - `106`: Semana de reunión Vida y Ministerio Cristianos (MWB).
- `Type`: Tipo de documento.
- `Title`: Título visible del artículo o semana.
- `Content`: **BLOB cifrado** con AES-128-CBC + Zlib.
- `FirstPageNumber` / `LastPageNumber`: Paginación original física.

---

### 1.3 `DatedText` (Mapeo de Fechas de Reunión)
Relaciona los documentos con las fechas de calendario en que se presentan:
- `DatedTextId`: ID único.
- `DocumentId`: Llave foránea hacia `Document.DocumentId`.
- `FirstDateOffset`: Fecha de inicio de la semana (formato numérico de fecha MEPS o YYYY-MM-DD).
- `LastDateOffset`: Fecha de fin de la semana.
- `Caption`: Texto de la fecha (ej. *"1-7 de septiembre"* o *"September 1-7"*).

---

### 1.4 `Multimedia` (Catálogo de Recursos Audiovisuales)
Registra todas las imágenes, videos y audios disponibles:
- `MultimediaId`: ID único del medio.
- `DataType`: Tipo de dato (`1` = Imagen estática, `2` = Audio, `3` = Video).
- `MimeType`: Tipo MIME (ej. `image/jpeg`, `video/mp4`).
- `Width` / `Height`: Dimensiones de la imagen en píxeles.
- `Label`: Etiqueta descriptiva o título del medio.
- `Caption`: Pie de foto o descripción accesible (alt text).
- `FilePath`: Nombre del archivo físico dentro del ZIP `contents` (ej. `202025320_E_cvr.jpg`).
- `Track`: Número de pista (para grabaciones de canciones o audio).
- `KeySymbol`: Símbolo de la publicación a la que pertenece.

---

### 1.5 `DocumentMultimedia` (Vínculo Documento $\leftrightarrow$ Multimedia)
Determina en qué parte de un artículo o semana aparece cada medio:
- `DocumentMultimediaId`: ID único de la relación.
- `DocumentId`: ID del documento contenedor.
- `MultimediaId`: ID del medio audiovisual asociado.
- `BeginParagraphOrdinal`: Número de párrafo donde comienza la aparición del medio.
- `EndParagraphOrdinal`: Número de párrafo donde finaliza la aparición.

---

### 1.6 `BibleCitation` (Citas y Referencias Bíblicas)
Catálogo de textos bíblicos citados en cada párrafo:
- `BibleCitationId`: ID único.
- `DocumentId`: Documento donde se cita el texto.
- `FirstBibleVerseId` / `LastBibleVerseId`: Identificador numérico del versículo inicial y final.
- `ParagraphOrdinal`: Párrafo específico donde se encuentra la cita.
- `MarginalClassification`: Tipo de cita (texto leído en la reunión vs. referencia de estudio).

---

### 1.7 `Extract` y `DocumentExtract` (Citas Textuales y Recuadros)
Guarda recuadros de texto, extractos de publicaciones o versículos leídos:
- `ExtractId`: ID del extracto.
- `Content`: **BLOB cifrado** con el texto del extracto.
- `Title`: Título del recuadro o fuente.
