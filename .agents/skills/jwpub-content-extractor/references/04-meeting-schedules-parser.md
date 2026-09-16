# 📅 04. Extracción de Programas de Reunión (MWB & Watchtower)

Este módulo documenta las reglas de extracción semántica para los dos formatos principales de reunión a partir del HTML descifrado.

---

## 🏛️ 1. Reunión Vida y Ministerio Cristianos (MWB - Meeting Workbook)

Documentos con `Document.Class = '106'`.

### 1.1 Estructura Semántica del HTML de MWB:
- **Fecha de la semana:** `<h1>` principal (ej. `SEPTEMBER 1-7` o `1-7 DE SEPTIEMBRE`).
- **Lectura semanal de la Biblia:** `<h2>` (ej. `1 CORINTIOS 1-3`).
- **Canciones:** Encabezados `<h3>` que contienen la clase `dc-icon--music` o texto `"Canción"` / `"Song"`.
  - **Canción Inicial:** Primer `<h3>` de canción.
  - **Canción Intermedia:** Segundo `<h3>` de canción (antes de la sección Nuestra Vida Cristiana).
  - **Canción Final:** Tercer `<h3>` de canción.
- **Tesoros de la Biblia (TGW - Treasures from God's Word):**
  - Discurso principal (10 min).
  - Perlas Escondidas (10 min).
  - Lectura de la Biblia (4 min).
- **Seamos Mejores Maestros (AYF - Apply Yourself to the Field Ministry):**
  - Se identifican mediante bloques con clase `.du-color--gold-700` o identificadores de asignación estudiantil.
  - Partes 1 a 4 con tiempos extraídos de los paréntesis `(X min)` o `(X mins)`.
  - Tipo de asignación: Lectura, Primera Conversación, Revisita, Curso Bíblico, Discurso.
- **Nuestra Vida Cristiana (LC - Living as Christians):**
  - Secciones con clase `.du-color--maroon-600`.
  - Partes 1 y 2 con títulos y tiempos asignados.
  - **Estudio Bíblico de la Congregación (CBS):** Parte de 30 min.
  - Palabras de conclusión.

---

## 📖 2. Estudio de La Atalaya (W - Watchtower Study)

Documentos con `Document.Class = '40'` y tabla de contenidos `Class = '68'`.

### 2.1 Estructura Semántica del HTML de La Atalaya:
- **Fecha de Estudio:** Extraída de la tabla de contenidos (`Class = '68'`) o del encabezado descriptivo `.desc`.
- **Título del Artículo:** `<h1>` o `<h3>` del artículo (`Class = '40'`).
- **Canciones:** Extraídas de los párrafos con clase `.pubRefs`:
  - **Canción de apertura:** Primer `.pubRefs`.
  - **Canción de conclusión:** Último `.pubRefs`.
- **Párrafos de Estudio:** Elementos `<p data-pid="X">` que contienen el cuerpo del texto para lectura.
- **Preguntas:** Elementos `<p class="qu">` asociados a cada párrafo numerado.
- **Citas Bíblicas:** Enlaces `<a class="b">` que referencian textos bíblicos leídos o consultados.
