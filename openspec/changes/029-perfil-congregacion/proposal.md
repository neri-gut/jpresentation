# Proposal: Perfil de congregación listo para reunión

## Intent

Cerrar el contrato de congregación que `001` dejó a medias. La consola ya crea un perfil, cambia el idioma de UI y guarda monitores; aún no es un perfil de reunión: no hay idioma de contenido, no se puede duplicar ni borrar, el horario de las dos reuniones no existe, el tema no expone acento/densidad y el orador se oculta si no hay pantalla asignada.

Este change deja un `ProfileDto` + settings del que puedan colgar cronómetro, catálogo y JWPUB sin otra migración de `profiles`.

## Scope

In scope:

- Dos ejes de idioma en la UI: `ui_locale` (ya) + `content_langwritten` con semilla embebida (`research/idiomas.md`). Sin HTTP.
- CRUD de perfil: renombrar, duplicar (copia settings), borrar con confirmación. MUST NOT borrar el último. FK `ON DELETE CASCADE` en `settings` y `meeting_programs`.
- Horario de las dos reuniones: `weekday` ISO 1–7 + `HH:mm` local, persistido por perfil.
- Tema de consola: paleta cerrada de acento + densidad `compact` / `comfortable` (los tokens CSS ya existen).
- Monitores: Identificar (destello + nombre, 2 s). Orador activo sin pantalla → ventana flotante de previsualización, no se oculta ni se pone fullscreen sobre la consola.
- CI de dominio: `npm ci --ignore-scripts` + `npm run build`, `cargo test --locked`, `cargo audit` (además del `npm audit` de `001`).

Out of scope (changes siguientes):

- Asistente de 9 pasos (`027`); el seed `Default` de `001` se queda.
- Catálogo vivo `alllangs=1`, himnario, guía, JWPUB (`003`+).
- `video_quality`, audio, atajos, fundidos, updater.
- Panel Stage / Audio / Mensajes (`022`); el cascarón de `001` no se reescribe.
- Hotplug de monitores, modo HUD avanzado (`018`), lockfile anti dos perfiles a la vez (`huecos`).
- Traducir las claves nuevas a los 14 JSON que no son `en` (fallback inglés).

## Approach

1. Un change de implementación, no un change de spec suelto: las specs vivas ya describen el comportamiento; este folder fija el recorte y los contratos.
2. Rust sigue dueño de validación, persistencia y colocación de ventanas. Vue solo pinta y despacha un DTO por comando.
3. Settings conocidos siguen siendo celdas JSON tipadas (`appearance`, `surfaces`, `panel` + `meeting_schedule`). No se ensancha `profiles` más que `content_locale`, que ya es columna.
4. La semilla de contenido es un JSON embebido en el crate de dominio. El catálogo HTTP la sustituirá después; no se copian las 503 lenguas al repo.
