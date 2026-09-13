# Lectura de `mwb_S_202609.jwpub`

Muestra local (Guía de actividades, español, sep–oct 2026). Sirve para fijar el contrato de datos. No se versiona el contenido de los artículos.

## Contenedor

```
mwb_S.jwpub          ZIP
├── manifest.json    metadatos de publicación
└── contents         ZIP interno (contentFormat: z-a)
    ├── mwb_S_202609.db      SQLite (en esta muestra, cabecera SQLite estándar)
    └── *.jpg                portada, thumbs y láminas univ_cnt / univ_sqr
```

`manifest.publication` relevante:

| Campo | Valor en la muestra |
|---|---|
| symbol / uniqueSymbol | `mwb26` (`undatedSymbol`: `mwb`) |
| language | `1` (coincide con fichero `_S_` = español) |
| year / issueNumber / issueId | 2026 / 9 / `20260900` |
| publicationType | Meeting Workbook |
| fileName | `mwb_S_202609.db` |
| First/Last fechas (en DB) | `20260907` … `20261101` |

Un `.jwpub` mensual cubre **varias semanas**. Cada semana es un `Document` class `106` + fila `DatedText`.

## Tablas que importan

- `Publication` — símbolo, idioma MEPS, issue, rango de fechas
- `Document` — 1 portada + 8 semanas (título = rango de fechas)
- `DatedText` — `FirstDateOffset` / `LastDateOffset` (yyyymmdd) → DocumentId
- `Multimedia` — cada imagen o vídeo
- `DocumentMultimedia` — en qué documento y párrafo cae el medio
- `Hyperlink` — `jwpub://b/…` (Biblia) y `jwpub://p/…` (publicación). En esta muestra **no** hay strings `jwpub-media://`
- `RefPublication` — otras pubs citadas (`sjj`, `lmd`, `nwtsty`, `w25`, …)

`meeting-schedules-parser` lee el HTML de `Document.Content`. Los medios **no** salen de ahí: salen de `Multimedia`.

## Dos clases de medio (esta muestra: 59 filas)

1. **Embebido** — `FilePath` no vacío, `image/jpeg`. Está en `contents`. No hay que ir a red.
2. **Externo** — `FilePath` vacío, `MimeType = video/mp4`. Clave de catálogo:

`KeySymbol` + `Track` + `MepsLanguageIndex` + `IssueTagNumber`

### Cánticos

`KeySymbol = sjjm`, `Track = número del cántico` (1, 17, 21, 28, …).  
Himnario referido: `sjj`. El vídeo musical no viaja en el workbook.

### Vídeos de la guía

`KeySymbol = mwbv`, `Track = 1|2|3`, `IssueTagNumber = 20260900` (el mismo issue).

### Otros vídeos

Símbolos de emisión / pubs (`jwb`, `jwb-098`, `jwb-125`, `jwbvod26`, `pk`, …) + `Track` + a veces issue.

## Semana ejemplo (7–13 sep, DocumentId 1)

Orden por párrafo:

| Párrafo | Tipo | Clave o archivo |
|---|---|---|
| (doc) | imagen sqr | `202026252_univ_sqr.jpg` |
| 3 | vídeo | `sjjm` / 1 |
| 9 | imagen | `…_cnt_1.jpg` |
| 10 | vídeo | `jwb-098` / 7 |
| 27 | vídeo | `sjjm` / 128 |
| 30 | imagen | `…_cnt_2.jpg` |
| 31 | vídeo | `mwbv` / 1 + issue 20260900 |
| 47 | vídeo | `sjjm` / 143 |

Eso cuadra con canción inicial, medio de tesoro, canción central, vídeo de vida cristiana, canción final.

## Resolución (adaptador, no spec pública)

El catálogo de publicaciones/medios de JW Library habla el mismo idioma que esta tabla:

- publicación del mes: símbolo `mwb`, issue `202609`, idioma escrito `S`, formato `JWPUB`
- cántico: símbolo `sjjm`, track N, idioma `S`, formato `MP4` (audio equivalente si se ofrece)
- vídeo de la guía: símbolo `mwbv`, track N, issue `20260900`, idioma `S`, formato `MP4`

Host, path y cabeceras viven **solo** en el crate adaptador. Si cambian, no se toca el dominio.

`jwpub-media://` en JW Library es la URI de runtime sobre esa misma clave. En este `.jwpub` la clave canónica es la fila `Multimedia`, no un string en el HTML.

## Implicación para JPresentation

```
MediaRef::Embedded { path }
MediaRef::Catalog {
  key_symbol,   // sjjm | mwbv | jwb-… | …
  track,        // u32
  lang_meps,    // 1 = S en esta muestra
  issue_tag,    // 0 o 20260900
  mime
}
```

`MediaResolver` usa `Catalog`. `ScheduleParser` usa `Document.Content` + `DatedText`. No mezclar las dos lecturas en un solo módulo.
