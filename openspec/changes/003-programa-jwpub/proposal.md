# Proposal: Programa semanal desde JWPUB y `jwpub-media://`

## Intent

Definir cómo JPresentation obtiene cada semana el orden de la reunión y sus medios: publicar `mwb` + `w` → parsear → extraer embebidos → resolver referencias `jwpub-media://` que no vienen en el paquete.

## Scope

In scope:
- Modelo `MeetingWeek` / `MeetingPart` / `MediaRef`
- Puertos `PublicationCatalog`, `ScheduleParser`, `MediaResolver`
- Mapeo al contrato público de meeting-schedules-parser (campos de programa, no copia del código)
- Tratamiento de `jwpub-media://` como identificador, no como HTTP
- Importación del programa al cronómetro
- Árbol multimedia agrupado por parte

Out of scope:
- Implementar endpoints concretos
- Descifrar o documentar el formato interno del `.db` de JWPUB
- Sidecar Node vs parser Rust (se decide en el change de código)

## Approach

Specs de comportamiento + adaptador aislado. El parser de sws2apps es la *forma* de la salida de programa; los vídeos se resuelven en otra capa.
