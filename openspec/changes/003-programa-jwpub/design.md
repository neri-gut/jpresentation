# Design: Programa semanal y medios de publicación

## Flujo

```
Operador: “Esta semana” (locale perfil)
        │
        ▼
PublicationCatalog.locate(lang, symbol=mwb|w, period)
        │  .jwpub preferente; .epub fallback
        ▼
ScheduleParser.parse(bytes) → MeetingWeek[]
        │  contrato alineado a loadPub (sws2apps)
        ▼
MediaExtractor.scan(package + refs)
        │  embedded files
        │  jwpub-media://
        │  catalog keys (símbolo + pista)
        ▼
MediaResolver.resolve(ref) → cache path | pending
        ▼
UI: Cronómetro (partes/tiempos) + Multimedia (medios por parte)
```

## Qué aporta meeting-schedules-parser

Paquete npm MIT que abre `.jwpub`/`.epub` y devuelve semanas (`MWBSchedule` / `WSchedule`): fecha, lectura semanal, canciones 1/2/3, tesoro, gemas, lectura, AYF 1–4 con tiempo/tipo, vida cristiana 1–2 con tiempo, CBS, título de Atalaya y canciones de estudio.

**No** extrae ilustraciones ni vídeos. Por eso JPresentation no puede quedarse solo en `loadPub`.

Opciones de implementación (change de código):

1. Adaptador Node/sidecar que llama `loadPub` — más rápido para validar el mapeo.
2. Parser Rust (zip + SQLite) — mejor para Tauri a largo plazo, sin runtime Node.
3. Híbrido: sidecar en dev, Rust cuando el contrato esté estable.

La UI solo habla `MeetingWeek`.

## Hallazgo en `mwb_S_202609.jwpub`

En esta muestra el SQLite es legible y **no** contiene strings `jwpub-media://`. Los cánticos y vídeos están en `Multimedia` con `FilePath` vacío y clave `(KeySymbol, Track, MepsLanguageIndex, IssueTagNumber)`.

- Cánticos: `sjjm` + número
- Vídeos de la guía: `mwbv` + track + issue `20260900`
- Láminas: `FilePath` jpg dentro de `contents`

`jwpub-media://` es la URI de runtime de JW Library sobre esa clave. JPresentation persiste `MediaRef::Catalog`, no la URI.

Ver `openspec/research/jwpub-mwb-S-202609.md`.

## `jwpub-media://` / clave de catálogo

El adaptador:

- Parsea la URI y/o la fila de metadatos de medio de la publicación
- Si hay fichero en el paquete → `embedded`
- Si no → consulta el catálogo de medios con la clave obtenida
- Nunca se pasa la URI cruda al WebView del auditorio

No se documentan aquí query params ni endpoints.

## Contenedor JWPUB (solo contexto)

Un `.jwpub` es un ZIP con manifiesto y un paquete interno que puede incluir SQLite + ficheros de imagen. El texto del programa y las tablas de medio se leen **solo** dentro del adaptador. No se versionan claves ni procedimientos de descifrado en este repo.

## Mapeo de dominio (entre semana)

| Origen parser | MeetingPart |
|---|---|
| song_first | song / opening |
| tgw_talk + 10 min | treasures |
| tgw_gems | gems (~10) |
| tgw_bread | bible_reading (~4) |
| ayf_partN + time | assignment |
| song_middle | song / middle |
| lc_partN + time | living |
| lc_cbs | congregation_bible_study (~30) |
| song_conclude | song / closing |

Fin de semana: título de estudio + canciones apertura/cierre + medios del artículo.

## Riesgos

- El catálogo o el esquema de URI pueden cambiar: fallar claro + cache.
- Idiomas sin “enhanced parsing”: tiempos AYF/LC pueden faltar; usar plantilla.
- Un `.jwpub` mensual contiene varias semanas: filtrar por fecha.
