# Design: Reloj de reunión

## Cause (030)

`ClockSnapshot` solo tenía `state: idle`. `SpeakerApp` pintaba placeholders. El panel era un cascarón. `SurfacesSetting` no tenía modo de orador; el desplegable de monitores no distinguía consola / primaria / preview.

## Clock

```
AssignmentClock
  Idle
  Armed { title, assigned_ms, elapsed_ms }
  Running { title, assigned_ms, elapsed_at_start, started_at }

arm(title, minutes)     → Armed, elapsed 0 (desde cualquier estado)
start                   → Armed → Running; Running es no-op
pause                   → Running → Armed (elapsed congelado)
finish                  → Idle (si no idle)
tick                    → true solo si Running (el hilo emite)
```

Validación: título recortado 1–80; minutos 1–180. Errores: `ClockNotArmed`, `ClockNotRunning`, `Invariant`.

`ClockSnapshot` (IPC snake_case):

| Campo | Notas |
|-------|--------|
| `rev` | monótono; Vue ignora eventos viejos |
| `state` | `idle` \| `armed` \| `running` |
| `title` | `null` si idle |
| `assigned_ms` / `elapsed_ms` | 0 si idle |
| `remaining_ms` | `i64`; negativo en desfase |
| `overtime_ms` | `max(0, -remaining)` |
| `progress_pct` | 0–100; 100 en desfase |
| `hue` | `green` \| `amber` \| `red` |

`clock_hue(assigned, remaining)`: `remaining <= 0` → rojo; `remaining <= max(assigned/5, 60_000)` → ámbar; si no, verde.

Tick: hilo en `setup`, 250 ms. MUST NOT `setInterval` dueño en Vue. `output_get` resincroniza al reconectar.

Comandos (solo operador): `clock_arm` (`ClockArmDto`), `clock_start`, `clock_pause`, `clock_finish`. Tras mutar, `timer://changed`. MUST NOT tocar `StageSnapshot`.

## HUD y panel

Orador: título + `mm:ss` (o `+m:ss`) + barra al `progress_pct`, color = `hue`. Layout a pantalla si `stage.kind === none` **o** `speaker_mode === hud_only`.

Panel: Armar / Iniciar / Pausar / Terminar. Colapsado: solo el tiempo. Misma pestaña Cronómetro. Si `use_speaker = false`, este bloque es el HUD.

El auditorio no pinta el reloj.

## Superficies

`SurfacesSetting.speaker_mode`: `mirror` (defecto) | `hud_only`. `#[serde(default)]` para celdas de `029`/`030`.

`MonitorDto.is_operator`: el display donde está la ventana Operador. UI: «console», «primary», preview = «Preview window».

`OutputState.speaker_mode` se copia al aplicar `surfaces`; `output_get` y evento `speaker://ui` lo publican al orador.

## Tests (sin WebView)

- arm 10 min → remaining 600_000, hue green
- 8 min transcurridos de 10 → ámbar (umbral 2 min)
- 11:20 → overtime 80 s, hue red, progress 100
- pause conserva elapsed; start retoma
- start desde idle → `ClockNotArmed`
- arm minutos 0 / 181 → `Invariant`
- serde: JSON de surfaces sin `speaker_mode` → `mirror`
