# Tasks — 031 Reloj de reunión

## 1. Dominio
- [x] 1.1 `AssignmentClock` + `ClockSnapshot` completo + `clock_hue` / umbral
- [x] 1.2 Tests puros: arm/start/pause/finish, ámbar, desfase, validación
- [x] 1.3 `SurfacesSetting.speaker_mode` con serde default `mirror`
- [x] 1.4 `MonitorDto.is_operator`

## 2. IPC y tick
- [x] 2.1 Comandos `clock_arm` / `clock_start` / `clock_pause` / `clock_finish` (solo operador)
- [x] 2.2 Hilo ~4 Hz emite `timer://changed` si `running`
- [x] 2.3 `output_get` incluye reloj + `speaker_mode`; evento `speaker://ui` al guardar surfaces

## 3. Vue
- [x] 3.1 Store del reloj: subscribe + arm/start/pause/finish
- [x] 3.2 `SpeakerApp` pinta snapshot (hue, barra, overtime)
- [x] 3.3 Bloque Cronómetro del panel + pestaña Cronómetro
- [x] 3.4 Configuración: modo orador, etiquetas consola/primaria, copy de preview
- [x] 3.5 Claves nuevas solo en `en.json`

## 4. Docs
- [x] 4.1 INDEX / AGENTS / README / CONTRIBUTING apuntan a `031`; `030` cerrado
- [x] 4.2 Deltas en `cronometro`, `panel`, `ventanas`
