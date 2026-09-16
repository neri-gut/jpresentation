---
name: jwpub-content-extractor
description: "Skill especializada para la extracción, desencriptación y consulta de archivos JWPUB (.jwpub) fuera del entorno Android. Cubre la estructura de contenedores ZIP anidados, algoritmo criptográfico AES-128-CBC + Deflate, catálogo del esquema SQLite interno, extracción de programas de reunión (Vida y Ministerio Cristianos y La Atalaya), extracción de recursos multimedia (imágenes, canciones, videos), citas bíblicas, y su integración en la arquitectura Rust (Tauri 2) + Vue 3 del proyecto."
metadata:
  version: "1.0.0"
  scope: "jwpub-parser-plugin"
  technologies: ["rust", "tauri-2", "sqlite", "aes-128-cbc", "zlib", "zip", "vue-3", "pinia", "typescript"]
---

# 📖 JWPUB Content Extractor & Parser Skill

Esta Skill proporciona el conocimiento arquitectónico, algoritmos criptográficos, especificaciones de bases de datos SQLite y procedimientos de extracción de contenido para procesar archivos de publicaciones de JW.ORG (**`.jwpub`**) directamente en el backend de Rust (Tauri 2) y consumirlos en el frontend de Vue 3.

---

## 📚 Documentación de Referencia Detallada

- 📦 **[Estructura del Contenedor JWPUB](./references/01-jwpub-architecture-and-container.md):** Organización en 2 niveles de compresión ZIP, archivo `manifest.json`, archivo `contents` interno y recursos embebidos.
- 🔐 **[Pipeline Criptográfico y Desencriptación](./references/02-crypto-and-decryption-pipeline.md):** Generación de `PubCard`, derivación de clave por XOR con clave maestra, desencriptación AES-128-CBC e inflación Zlib/Deflate.
- 🗄️ **[Catálogo del Esquema SQLite del JWPUB](./references/03-sqlite-schema-catalog.md):** Descripción exhaustiva de tablas (`Publication`, `Document`, `DatedText`, `Multimedia`, `DocumentMultimedia`, `BibleCitation`, `Extract`, etc.).
- 📅 **[Extracción de Programas de Reunión](./references/04-meeting-schedules-parser.md):** Lógica de parseo para la reunión Vida y Ministerio Cristianos (MWB) y Estudio de La Atalaya (W Study) basada en el estándar de `sws2apps/meeting-schedules-parser`.
- 🖼️ **[Gestión de Recursos Multimedia y Media Protocol](./references/05-multimedia-and-assets-pipeline.md):** Extracción de imágenes en alta resolución, asignación de pistas de canciones, marcas de video y resolución de esquemas `jwpub-media://`.
- 💡 **[Ejemplo de Servicio en Rust](./examples/rust-jwpub-service.rs):** Implementación completa de extracción, descifrado y consultas SQLx / rusqlite.
- 💡 **[Tipos y DTOs TypeScript](./examples/jwpub-types.ts):** Contratos de datos tipados para Pinia stores y componentes de Vue.

---

## 🎯 1. Visión General del Formato JWPUB

Un archivo `.jwpub` es un paquete empaquetado diseñado originalmente para la aplicación JW Library (Android/iOS/Desktop). No es un archivo plano, sino un **archivo ZIP de doble capa** que contiene:
1. **Capa Externa:** Contiene `manifest.json` y un archivo binario comprimido sin extensión llamado `contents`.
2. **Capa Interna (`contents`):** Es a su vez otro archivo ZIP que contiene:
   - Una base de datos SQLite con extensión `.db` (ej. `mwb_S_202509.db` o `w_S_202509.db`).
   - Múltiples archivos de imagen (`.jpg`), portadas, miniaturas (`-120x120.jpg`, `-600x600.jpg`), y opcionalmente audios/videos o marcas de tiempo.
3. **Capa de Contenido Cifrado:** Los textos y artículos completos dentro de la tabla `Document` (columna `Content`) están cifrados con **AES-128-CBC** y comprimidos con **Deflate/Zlib**.

---

## 🔐 2. Algoritmo de Desencriptación Paso a Paso

1. **Obtener la "Ficha de Publicación" (PubCard):**
   Consultar en la tabla `Publication` de la base de datos SQLite interna:
   $$\text{PubCard} = \texttt{MepsLanguageIndex} \_ \texttt{Symbol} \_ \texttt{Year} \_ \texttt{IssueTagNumber}$$
   *(Ejemplo: `0_mwb25_2025_20250900` o `1_w25_2025_20250900`)*

2. **Clave Secreta Maestra Base64:**
   ```text
   MTFjYmI1NTg3ZTMyODQ2ZDRjMjY3OTBjNjMzZGEyODlmNjZmZTU4NDJhM2E1ODVjZTFiYzNhMjk0YWY1YWRhNw==
   ```
   *(Decodificada a Hexadecimal: `11cbb5587e32846d4c26790c633da289f66fe5842a3a585ce1bc3a294af5ada7`)*

3. **Derivación de Clave (XOR):**
   - Calcular el hash SHA-256 de la cadena UTF-8 `PubCard` (32 bytes).
   - Aplicar una operación binaria XOR byte a byte entre el hash SHA-256 y la clave maestra hex decodificada (32 bytes).
   - El resultado son 32 bytes (64 caracteres hex):
     - **AES Key (16 bytes):** Los primeros 16 bytes (32 caracteres hex).
     - **AES IV (16 bytes):** Los siguientes 16 bytes (32 caracteres hex).

4. **Descifrado y Decompresión:**
   - Desencriptar el BLOB de `Document.Content` usando **AES-128-CBC** con la clave e IV obtenidos.
   - Quitar el padding PKCS#7.
   - Decomprimir los bytes resultantes con **Zlib / Raw Deflate** (`zlib::decompress` o `flate2::read::DeflateDecoder`).
   - El resultado es el documento **HTML estructurado** listo para parsear.

---

## 🏛️ 3. Integración en el Micro-Core y Plugins

El procesador de JWPUB se integra como un plugin del sistema:

```text
Usuario carga .jwpub ➔ Tauri Command (Guard::authorize)
  ➔ JwpubService::extract_and_parse(bytes)
  ➔ Descomprimir ZIPs & Cargar SQLite en memoria
  ➔ Derivar clave AES ➔ Descifrar documentos
  ➔ Extraer cronograma, canciones, citas bíblicas y multimedia
  ➔ Guardar imágenes en la caché de medios de la organización (storage_path)
  ➔ Retornar AppResponse<ParsedScheduleDTO>
  ➔ Pinia Store (schedule & media staging)
  ➔ Vistas de Operador, Auditorio y Orador
```

---

## 📋 4. Checklist para la Implementación de Módulos JWPUB

- [ ] ¿Se diseñó y presentó el diagrama D2 en `docs/diagrams/plugins/` antes de programar?
- [ ] ¿Se utiliza la descompresión en memoria para el SQLite interno sin bloquear el hilo principal de Tokio?
- [ ] ¿Se implementó el descifrado AES-128-CBC + Deflate con manejo estricto de errores (`?` hacia `AppError`)?
- [ ] ¿Las imágenes extraídas se guardan en el `storage_path` seguro de la organización activa?
- [ ] ¿Se resolvieron correctamente los esquemas de URI `jwpub-media://` para las vistas de Vue?
- [ ] ¿Se ejecutan las pruebas unitarias y de integración con `cargo test` y `bun test`?
