# Biblia en el catálogo (2026-09-12)

`GETPUBMEDIALINKS` sin `issue`:

| pub | lang | formatos | nota |
|---|---|---|---|
| `nwtsty` | S, E | JWPUB (S solo JWPUB; E también PDF) | Edición de estudio. ~124 MB en S |
| `nwt` | S | 400 | no usar |
| `Rbi8` / `bi12` | S | JWPUB, EPUB, PDF, MP3 | edición 1987, más liviana |

Defecto de perfil: `nwtsty` si existe; si no, la primera edición JWPUB disponible.

Almacén: `bible/{langwritten}/{pub}.jwpub`. Una descarga, no semanal.

Enlaces en la guía: `jwpub://b/NWTR/{book}:{chap}:{verse}-…`. El adaptador mapea la edición del enlace al símbolo del perfil.
