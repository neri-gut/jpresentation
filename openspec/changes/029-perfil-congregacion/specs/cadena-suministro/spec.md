# Delta — Cadena de suministro (029)

## ADDED Requirements

### Requirement: Verificar en CI, no solo auditar
Cada PR y `main` SHALL, además de `npm audit --omit=dev`:

- `npm ci --ignore-scripts` + `npm run build` (incluye `vue-tsc --noEmit`)
- `cargo test --manifest-path src-tauri/Cargo.toml --locked`
- `cargo audit` (o `cargo deny advisories`)

El job de `cargo audit` MAY degradar a warning si el token de GitHub no está (PRs de fork); MUST NOT impedir que `frontend` y `rust` reporten su propio fallo.

#### Scenario: PR con test de dominio roto
- GIVEN un PR que rompe `ProfileStore::delete`
- WHEN corre CI
- THEN el job `rust` falla
- AND no se mergea en verde
