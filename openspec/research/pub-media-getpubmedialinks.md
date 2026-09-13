# GETPUBMEDIALINKS

Verificado contra `https://b.jw-cdn.org/apis/pub-media/GETPUBMEDIALINKS` y `example/sample.ts` de meeting-schedules-parser (12 sep 2026). Solo metadatos; no se versionan URLs de binarios.

## Cómo lo usa sws2apps

```ts
const JW_CDN = 'https://b.jw-cdn.org/apis/pub-media/GETPUBMEDIALINKS?';
const url = JW_CDN + new URLSearchParams({
  langwritten: language, // p.ej. 'S'
  pub,                   // 'mwb' | 'w'
  output: 'json',
  issue: issueDate,      // 'YYYYMM'  →  '202609'
});
const result = await (await fetch(url)).json();
const JWPUB = result.files[language].JWPUB; // [{ file: { url, checksum, ... } }]
await loadPub({ url: JWPUB[0].file.url });
```

Descubrimiento en el sample: recorre meses (`mwb` de 2 en 2, `w` de 1 en 1) hasta HTTP 404. JPresentation **no** hace ese barrido: solo esta/próxima semana o un issue pedido.

## Parámetros que funcionan

| Query | Uso |
|---|---|
| `langwritten` | Código escrito (`S` español). Coincide con `_S_` del fichero |
| `pub` | Símbolo: `mwb`, `w`, `sjjm`, `mwbv`, `jwb-098`, … |
| `issue` | `YYYYMM` (`202609`). `20260900` también responde y se normaliza a `202609` |
| `track` | Pista: cántico o vídeo. Sin track en `mwbv`+issue lista todas |
| `output` | `json` |
| `fileformat` | Opcional (`MP4`, `JWPUB`). Si se omite, vienen todos los formatos |

No hace falta autenticación en estas pruebas. Cabecera `Accept: application/json` basta.

## Forma de la respuesta

```
{
  pub, issue, track, pubName, formattedDate,
  languages: { S: { name, locale, direction } },
  files: {
    S: {
      JWPUB | EPUB | PDF | MP4 | MP3 | … : [
        { title, label, track, pub, filesize, mimetype,
          file: { url, checksum, modifiedDatetime } }
      ]
    }
  }
}
```

`files[langwritten][FORMAT][i].file.url` apunta al CDN de objetos (`cfp2.jw-cdn.org` en la prueba). El dominio de ficheros puede cambiar; el adaptador no lo hardcodea.

## Mapeo desde `Multimedia` del JWPUB

| Clave JWPUB | Llamada |
|---|---|
| publicación del mes | `pub=mwb\|w`, `issue=YYYYMM`, formato `JWPUB` |
| `sjjm` + Track N | `pub=sjjm`, `track=N` → MP4 (240/360/480/720) y MP3 |
| `mwbv` + Track + IssueTag | `pub=mwbv`, `track`, `issue=YYYYMM` → MP4 |
| `jwb-098` + Track 7 | `pub=jwb-098`, `track=7` → MP4 |
| `sjj` + Track | himnario texto (RTF), **no** el audio/vídeo |

Calidad: el adaptador elige un `label` (p.ej. 360p en equipos modestos, 720p si hay margen). Checksum para validar cache.

## Reglas para JPresentation

- GET de catálogo y GET del fichero son dos pasos. El dominio solo ve `MediaRef.catalog`.
- Preferir JWPUB sobre EPUB para el parser.
- 404 = issue/idioma no publicado todavía; no reintentar en bucle.
- Host `b.jw-cdn.org` es el de la muestra de sws2apps; si deja de responder, el adaptador puede apuntar a otro host del mismo API sin tocar Vue.
