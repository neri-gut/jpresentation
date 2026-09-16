# Tasks — 030 Superficies preview y HUD

## 1. Dominio
- [x] 1.1 `AudiencePlacement` + `audience_placement` (None / operador / 1 display = Preview)
- [x] 1.2 `speaker_placement` recibe `operator_monitor_id`; no cubre la consola
- [x] 1.3 Tests puros de ambos mapeos + fake `PlatformSurface`

## 2. Escritorio
- [x] 2.1 Preview flotante 960×540, título de rol, cascada desde el operador, sin fullscreen
- [x] 2.2 Cover solo en monitor ≠ operador y ≥2 pantallas; recrear ventana si cambia `decorated`
- [x] 2.3 `parent(&operator)` al crear audience/speaker
- [x] 2.4 `close_owned_surfaces` en `CloseRequested`/`Destroyed` del operador

## 3. HUD
- [x] 3.1 `SpeakerApp`: layout a pantalla completa si `stage.kind === "none"`
- [x] 3.2 Barra inferior a todo el ancho (lista para el overlay futuro)
- [x] 3.3 Claves nuevas solo en `en.json` si hacen falta

## 4. Docs
- [x] 4.1 INDEX / AGENTS / README apuntan a `030`
- [x] 4.2 Deltas en `ventanas`; checklist manual: cerrar operador, preview a 1 monitor, HUD grande
