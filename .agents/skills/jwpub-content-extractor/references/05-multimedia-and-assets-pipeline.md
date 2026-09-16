# 🖼️ 05. Gestión de Recursos Multimedia y Media Protocol

Uno de los principales beneficios de parsear archivos `.jwpub` es la disponibilidad inmediata de todas las imágenes y recursos multimedia sin necesidad de descargas manuales.

---

## 📂 1. Ubicación y Extracción de Medios

Dentro del archivo `contents` (el ZIP interno), los archivos multimedia se encuentran en la raíz:
- Imágenes de alta calidad: `<id>_cvr.jpg` (portadas), `<id>_cnt_1.jpg` (ilustraciones de artículos).
- Miniaturas: `<id>_sqr-600x600.jpg`, `<id>_lsr-1200x600.jpg`.

---

## 🔌 2. Esquema de URI `jwpub-media://` y Custom Protocol en Tauri 2

En el HTML descifrado, las imágenes se referencian con el esquema:
```html
<img src="jwpub-media://202025320_E_cvr.jpg" alt="Descripción de la imagen" />
```

### 2.1 Estrategia de Resolución para Frontend:

El backend de Tauri 2 provee dos opciones:
1. **Extracción a Directorio de Caché:**
   - Descomprimir las imágenes del ZIP a `storage_path/organizations/<org_id>/media/cache/<pub_symbol>/`.
   - Reemplazar en el HTML `jwpub-media://archivo.jpg` por la URL local de Tauri `http://asset.localhost/organizations/<org_id>/media/cache/<pub_symbol>/archivo.jpg` o convertirlas a base64 Data URLs para visualización instantánea.
2. **Carga en Staging para Presentación Multipantalla:**
   - Cuando el operador selecciona una ilustración de la reunión, el `PresentationStore` envía la ruta física del archivo local a las ventanas secundarias (`audience-display` y `speaker-display`), permitiendo mostrarla en pantalla completa de inmediato.

---

## 🎵 3. Canciones y Videos

- **Canciones:** El número de canción extraído (ej. `Canción 84`) se mapea automáticamente con el repositorio de pistas de audio locales (`storage_path/songs/sjj_084.mp3` o video `sjj_084_720p.mp4`).
- **Videos del programa:** Las referencias a videos dentro del programa de reunión (ej. dramatizaciones, introducciones de libros) se extraen con su título y marcas de tiempo para búsqueda automática en la biblioteca de medios.
