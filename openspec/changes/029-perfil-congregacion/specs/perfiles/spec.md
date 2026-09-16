# Delta — Perfiles (029)

## ADDED Requirements

### Requirement: No borrar el último perfil
El sistema MUST NOT borrar el único perfil que quede. La consola pide confirmación antes de borrar cualquiera. Si se borra el perfil activo y hay otros, el sistema SHALL seleccionar el más reciente de los restantes y rehidratar la consola. Las celdas de `settings` (y `meeting_programs`) del perfil borrado SHALL eliminarse en la misma transacción (`ON DELETE CASCADE`).

#### Scenario: Único perfil
- GIVEN solo existe `Default`
- WHEN el operador confirma Borrar
- THEN el perfil sigue ahí
- AND la consola muestra `LastProfile`
- AND el auditorio no se cierra

#### Scenario: Borrar el activo habiendo otro
- GIVEN “Cong A” (activo) y “Cong B”
- WHEN borra “Cong A”
- THEN queda “Cong B” seleccionado
- AND los settings de A ya no están

### Requirement: Duplicar copia settings
Duplicar SHALL crear un perfil nuevo (otro id, `ui_locale` y `content_locale` iguales, nombre `"{name} (copy)"` recortado a 80) y copiar **todas** las celdas de `settings` del origen. MUST NOT seleccionar el duplicado por sí solo.

#### Scenario: Duplicar congregación
- GIVEN “Cong A” con UI `es`, contenido `S`, tema `dark` y auditorio en el monitor 2
- WHEN duplica
- THEN existe “Cong A (copy)” con los mismos locale, tema y monitores
- AND “Cong A” sigue activo

### Requirement: Semilla de contenido hasta el catálogo
Hasta que un change posterior pegue a `GETPUBMEDIALINKS?alllangs=1`, el selector de idioma de contenido SHALL listar la semilla embebida de lenguas frecuentes (`research/idiomas.md`: `E S F I X T U J KO CHS CH A G O P K IN TG AM HI Z`). El selector MUST ser filtrable por nombre o código. Un `content_langwritten` fuera de la semilla MUST rechazarse al persistir (`UnknownContentLanguage`). Perfil nuevo: `content_locale = E`.

#### Scenario: UI en español, contenido tagalo
- GIVEN un perfil con `ui_locale = es`
- WHEN elige contenido `TG`
- THEN los menús siguen en español
- AND `content_locale` queda `TG`
- AND no hay petición de red

#### Scenario: Código inventado
- GIVEN un `profile_update` con `content_locale = "ZZZ"`
- WHEN Rust valida
- THEN responde `UnknownContentLanguage`
- AND la fila no cambia

### Requirement: Horario de las dos reuniones
Cada perfil SHALL guardar, en hora **local del equipo**:

- entre semana: `midweek_weekday` (ISO 1=lunes … 7=domingo) + `midweek_time` (`HH:mm` 24h)
- fin de semana: `weekend_weekday` + `weekend_time`

Defecto: martes 19:00 / domingo 10:00. Weekday fuera de 1–7 o tiempo que no cumpla `HH:mm` MUST rechazarse. Este dato vive en settings (`meeting_schedule`); el asistente de `027` lo reutilizará, no lo redefine.

#### Scenario: Martes por la noche
- GIVEN un perfil nuevo
- WHEN el operador pone entre semana martes 19:30 y guarda
- THEN `meeting_schedule.midweek_weekday = 2` y `midweek_time = "19:30"`
- AND el auditorio no cambia
