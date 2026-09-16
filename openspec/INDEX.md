# JPresentation — SDD

Migración de **JMultimedia 2.8.01** (Windows / .NET) a **Tauri 2.11 + Vue 3 + TypeScript + Rust + SQLite**.

Producto: **JPresentation** (`org.jpresentation.app`). Complemento no oficial de JW Library. Specs de v1 **listas**. Change de código actual: `031-reloj-reunion` (`001`, `029` y `030` cerrados).

## Cómo leer

1. `vision` — qué es y qué no
2. Plataforma: `arquitectura`, `practicas`, `sincronizacion`, `extensibilidad`, `seguridad`, `rendimiento`, `tema`, `cache`, `onboarding`, `licencia-updater`, `fundidos`, `cadena-suministro`, `devcontainer`
3. Dominios: `canciones`, `audio`, `atajos`, `alertas`, `cronometro`, `programa-semanal`, `multimedia`, `proveedor-medios`, `biblia`, `texto`, `navegador`, `panel`, `perfiles`, `ventanas`
4. `grabador` — retirado
5. Changes `001` … `031-reloj-reunion`
6. Research: `jwpub-mwb-S-202609`, `pub-media-getpubmedialinks`, `biblia-jwpub`, `idiomas`, `json-locales`, `himnario-sjjm`, `plantillas-evento`, `textos-csv`, `huecos`

## Decisiones

| Tema | Decisión |
|------|----------|
| Nombre | JPresentation |
| Stack | Tauri **2.11.x**, Vue 3 + TS + Vite + Pinia, Rust, SQLite WAL |
| Superficies | Operador / Auditorio / Orador |
| Grabador | Fuera |
| Medios | Catálogo + JWPUB/EPUB + `jwpub-media://` |
| Programa | `mwb` + `w` |
| Himnario | `sjjm` 1–163 MP4; ambiente aparte |
| Audio | Solo auditorio; máster por perfil; sin precucha |
| Atajos | Registry vacío en v1 |
| JWPUB ilegible | Error + EPUB/local; no descifrar |
| Vídeo | `video_quality` única (`best`…240) |
| Plantillas | Entre semana, fin de semana, circuito + usuario |
| Diario / año | Manual; diario = reposo auditorio; año a la hora |
| Cronómetro | Arma siguiente; Iniciar cuenta; deshacer; no lanza medios |
| Pre-reunión | Horario por perfil; countdown; a la hora año + cántico en stage |
| Alertas | Visuales; consola + orador; auditorio off |
| Orador | Configurable; HUD avanzado; sin audio ni input |
| Biblia | Lookup local; guía/Atalaya/CSV; pagina solo el operador |
| Navegador | Página o URL directa; sin ripping |
| Local | Raíces + inspector `.jwpub` |
| Panel | Stage, audio, cronómetro, Biblia, mensajes |
| Sync | Rust dueño; fan-out; un decoder |
| Código | DTO por capa; ≤4 params; rustdoc/TSDoc; `Result` sin tumbar stage |
| Idiomas | UI `en` default/fallback; contenido `langwritten` |
| Tema | Solo consola (claro/oscuro/sistema) |
| Recursos | Un decoder; pestañas perezosas; degradar |
| Repo | Lockfile + audit CI + devcontainer en `001` |
| Cache | Himnario/Biblia fijos; `week/` esta+próxima; reintento manual |
| Onboarding | Asistente; 1+ perfiles; weekday+hora |
| Update | `tauri-plugin-updater` + GitHub Releases firmados |
| Fundidos | Ambiente 1200/800 ms; foto 400 ms; zoom = ampliar stage; PDF embebido |
| Móvil | Fuera de v1; puertos listos |

## Changes

| Id | Tema |
|----|------|
| 001 | Cimentación Tauri + Vue |
| 002 | Decisiones de producto |
| 003 | Programa JWPUB |
| 004 | Himnario / guía / crono |
| 005 | HUD orador |
| 006 | Biblia JWPUB |
| 007 | Idiomas |
| 008 | Himnario 1–163 |
| 009 | Audio |
| 010 | Atajos + JWPUB ilegible |
| 011 | Calidad + plantillas |
| 012 | Texto diario |
| 013 | Reposo / deshacer |
| 014 | Horario pre-reunión |
| 015 | Alertas visuales |
| 016 | Textos + CSV |
| 017 | Monitor orador |
| 018 | HUD avanzado |
| 019 | Navegador stream |
| 020 | Tema, DX, npm |
| 021 | Explorador local |
| 022 | Panel |
| 023 | Sincronización |
| 026 | Prácticas de capas |
| 027 | Cache, onboarding, updater |
| 028 | Fundidos, zoom imagen, PDF |
| 029 | Perfil de congregación (CRUD, contenido, horario, tema, monitores, CI) |
| 030 | Superficies: dueño, preview real, HUD a pantalla |
| 031 | Reloj de reunión (una parte), HUD vivo, modo orador |

## Implementación (cuando salgamos de specs)

1. `001` scaffold + devcontainer + lockfiles — **hecho**  
2. Perfiles, i18n, tema, monitores — change `029` — **hecho**  
3. Superficies + sync + panel — `030` hecho; panel Stage/Audio/Mensajes y sync de vídeo después  
4. Cronómetro — change `031` (una parte suelta + HUD vivo); plantillas + textos después  
5. Adaptador JWPUB + catálogo  
6. Guía → multimedia de la semana  
7. Himnario + audio  
8. Biblia + navegador + local  
9. Empaque Win/macOS/Linux  

## Origen

Capturas `JMulti-0*.png`, jwmultimedia.org. Grabador original no se porta.
