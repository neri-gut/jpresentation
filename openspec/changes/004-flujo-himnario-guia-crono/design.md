# Design: Himnario + guía + cadena de cronómetro

## Almacenes

```
app_data/
  hymnal/{langwritten}/{track}.{mp4|mp3}   # permanente
  week/{lang}/{issue}/mwb.jwpub
  week/{lang}/{date}/media/*               # solo esa semana
```

Purgar semana no toca `hymnal/`.

## Flujo operador

1. **Canciones** — `PublicationCatalog.list_tracks("sjjm", lang)` → descargar faltantes → Play resuelve himnario.
2. **Multimedia / Cronómetro** — `locate(mwb, issue)` → parse `MeetingWeek` de la fecha → extraer `MediaRef` no-sjjm → descargar lote semanal.
3. Ranuras  inicio/central/final = números del `MeetingWeek` → `HymnalLibrary`.
4. Cronómetro consume las mismas partes. `MeetingClock.complete_current()` activa la siguiente.

## Avance

```
start(part[0])
  on zero + auto_advance → complete → start(part[i+1])
  on next_pressed        → complete → start(part[i+1])
  on zero + !auto_advance → overtime
```

No llama a `OutputPort.show` salvo setting futura `present_part_media_on_advance` (fuera de este change).

## API

- Himnario: `GETPUBMEDIALINKS?pub=sjjm&langwritten=&track=`
- Guía: `pub=mwb&issue=YYYYMM` → JWPUB
- Vídeos de guía: `pub=mwbv|jwb-…` según `Multimedia`
