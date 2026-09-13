# Prácticas de código y capas

## Purpose

Mismo estilo en Rust, TypeScript y comandos Tauri: contratos (DTO), funciones cortas, documentación de negocio y errores que no tumben el auditorio.

En este stack las tres capas no son “Spring”. El mapeo es:

| Capa | Dónde |
|------|--------|
| Controller / adaptador | Comando Tauri + composable Vue que hace `invoke` |
| Service | Crate de dominio (`meeting`, `media`, `clock`, …) |
| Repository | SQLite, fs, HTTP de catálogo — solo Rust |

Vue no habla con el repository. El comando no ejecuta SQL.

## Requirements

### Requirement: DTOs entre capas
Toda frontera SHALL usar un tipo nombrado (DTO / struct / interface), no una bolsa de primitivos.

- Vue → Rust: argumento único de comando (`StartPartDto`, no `(id, mins, flag, ts)` sueltos)
- Rust service → repo: `SaveWeekRecord`, no 8 `String`/`i64`
- Rust → Vue: `StageSnapshot`, `ClockSnapshot`, `AppErrorDto` (ver `sincronizacion`)
- Eventos Tauri: el mismo DTO de lectura, no JSON anónimo

Prohibido `any`, `serde_json::Value` como contrato público, y `HashMap<String, String>` para datos de reunión. IDs: newtypes (`ProfileId`, `PartId`), no `i64` crudo en la UI.

Si una función necesitaría 5+ datos, se agrupan en un DTO — no se relaja el límite de parámetros.

#### Scenario: Iniciar parte
- GIVEN el operador pulsa Iniciar
- WHEN el composable llama al backend
- THEN `invoke("clock_start", { payload: StartPartDto })` 
- AND el service recibe `StartPartDto`, no cuatro primitivos
- AND el repo persiste `ClockTickRecord`

### Requirement: Máximo 4 parámetros
Una función (Rust o TS exportada) MUST tener ≤ 4 parámetros. El 4º no es un “options bag” anónimo: si hay contexto extra, DTO. Getters triviales y `invoke` generado quedan fuera. Closures de UI de una línea no cuentan como API.

### Requirement: Documentar el negocio
Toda función **pública** (comando Tauri, método de service, composable exportado, puerto) SHALL llevar:

- Rust: `///` rustdoc (qué hace, errores, invariante)
- TypeScript: TSDoc `/** */` en exports

Se documenta el **porqué** y el contrato, no la línea obvia. Módulos de dominio: un párrafo al inicio del archivo con el flujo (p. ej. “Terminar arma la siguiente; no abre medios”).

CI MAY fallar si falta rustdoc/TSDoc en `pub` / `export` del workspace `src-tauri` y `src/`.

### Requirement: Errores sin tumbar la app
- Servicios y repos devuelven `Result<T, AppError>`. `AppError` es enum (`NotFound`, `UnreadablePub`, `Network`, `Busy`, `Invariant`, …), serializable a `AppErrorDto`.
- Comandos Tauri: `Result<T, AppErrorDto>`. MUST NOT `unwrap` / `expect` / `panic` en caminos de reunión. `unwrap` solo en tests o invariantes de arranque que ya impiden subir.
- Vue: todo `invoke` en `try`. Fallo = toast en consola + estado del snapshot **sin** tocar el auditorio. El stage no se cierra solo.
- Parser JWPUB ilegible, 404 de catálogo, disco lleno: degradar (EPUB, local, pendiente), no `process.exit`.
- Tokio/tareas: panic en un job de descarga se captura y se marca el ítem fallido.
- Logs: código de error + `rev`; sin rutas de usuario ni tokens.

#### Scenario: Guía corrupta
- GIVEN un `mwb` que no abre
- WHEN el operador pide Esta Semana
- THEN la consola muestra `UnreadablePub`
- AND el auditorio sigue en diario o medio actual
- AND la app no se cierra

### Requirement: Otras prácticas del repo
- TypeScript `strict`, `noUncheckedIndexedAccess`. Clippy `pedantic` selectivo + `deny(unwrap_used)` en crates de dominio (salvo tests).
- SQL solo en repository. Migraciones versionadas.
- Un comando = una acción. Sin “god command” que mezcle reloj y descarga.
- Tests del dominio sin WebView. Un escenario OpenSpec crítico → al menos un test Rust.
- i18n: cero literales de UI en Vue; claves en `locales/`.
- Listas virtualizadas; un decoder de stage (`rendimiento`).
- Capabilities por superficie (`seguridad`).
- PRs: `npm ci`, `cargo --locked`, audit (`cadena-suministro`).
