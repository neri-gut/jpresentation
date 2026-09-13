# Idiomas

Dos ejes: UI (BCP-47) vs contenido (`langwritten` JW).

## Contenido

`GETPUBMEDIALINKS?pub=mwb&issue=202609&alllangs=1&langwritten=E` devuelve `languages` con **503** códigos.

Cada entrada: `{ name, direction, locale, script }`.

Códigos no son ISO. Ejemplos:

| JW | locale | nombre | dir |
|---|---|---|---|
| E | en | English | ltr |
| S | es | español | ltr |
| F | fr | français | ltr |
| I | it | italiano | ltr |
| X | de | German | ltr |
| T | pt | Portuguese (Brazil) | ltr |
| U | ru | Russian | ltr |
| J | ja | Japanese | ltr |
| KO | ko | Korean | ltr |
| CHS | cmn-hans | Chinese Mandarin (Simplified) | ltr |
| CH | cmn-hant | Chinese Mandarin (Traditional) | ltr |
| A | ar | Arabic | rtl |
| G | el | Greek | ltr |
| O | nl | Dutch | ltr |
| P | pl | Polish | ltr |
| K | uk | Ukrainian | ltr |
| IN | id | Indonesian | ltr |
| TG | tl | Tagalog | ltr |
| AM | am | Amharic | ltr |
| HI | hi | Hindi | ltr |
| Z | sv | Swedish | ltr |

`MepsLanguageIndex` del JWPUB (p. ej. `1` = español) solo se usa al leer el paquete. Hacia el API se envía `langwritten`.

### Disponibilidad (sep 2026)

`mwb/202609`: los 21 de la semilla tienen JWPUB.  
`sjjm` track 1: E S F A CHS J OK (árabe incluido).  
`nwtsty`: 404 en A, G, AM.

## UI

Fuente: `locales/es.json`. v1: `es` + `en`. Comunidad: JSON + manifiesto, sin Rust.

No copiar los 503 nombres al repo como lista muerta; cachear la respuesta `alllangs`.
