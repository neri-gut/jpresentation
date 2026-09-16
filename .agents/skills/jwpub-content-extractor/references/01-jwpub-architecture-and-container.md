# 📦 01. Arquitectura del Contenedor JWPUB (.jwpub)

El formato `.jwpub` es un contenedor de publicación diseñado para empaquetar de forma eficiente textos, multimedia y bases de datos relacionales.

---

## 📂 1. Estructura de Doble Capa ZIP

```text
archivo.jwpub (Archivo ZIP Externo)
│
├── manifest.json            # Metadatos globales de la publicación
└── contents                 # Archivo binario comprimido (Archivo ZIP Interno)
    │
    ├── <symbol>_<lang>_<issue>.db   # Base de datos SQLite interna (ej. mwb_E_202509.db)
    ├── <media_id>_cvr.jpg           # Imagen de portada en alta resolución
    ├── <media_id>_cnt_1.jpg         # Imágenes de contenido para artículos
    ├── <media_id>_sqr-600x600.jpg   # Miniaturas cuadradas para listas
    └── <media_id>_lsr-1200x600.jpg  # Miniaturas horizontales
```

---

## 📑 2. El Archivo `manifest.json`

El archivo `manifest.json` en la raíz del `.jwpub` describe los metadatos de la publicación:

```json
{
  "name": "mwb_E_202509",
  "symbol": "mwb",
  "mepsLanguage": "E",
  "language": "en",
  "year": 2025,
  "issue": "202509",
  "type": "mwb",
  "formatVersion": 1,
  "schemaVersion": 1
}
```

---

## 🛠️ 3. Procedimiento de Apertura y Extracción

1. **Lectura de la Capa Externa:** Abrir el stream binario de bytes del archivo `.jwpub` con un lector ZIP (ej. crate `zip` en Rust o `JSZip` / `bun:ffi`).
2. **Extracción de `contents`:** Leer la entrada `contents` como arreglo de bytes en memoria (`Vec<u8>`).
3. **Lectura de la Capa Interna:** Abrir el arreglo de bytes de `contents` como un segundo archivo ZIP en memoria.
4. **Separación de Recursos:**
   - Encontrar la entrada que termina en `.db` $\rightarrow$ Base de datos SQLite.
   - Encontrar todas las entradas con extensiones multimedia (`.jpg`, `.png`, `.mp4`, `.mp3`) $\rightarrow$ Almacenarlas en la carpeta de recursos de la app (`storage_path/media/jwpub_<symbol>_<issue>/`).
