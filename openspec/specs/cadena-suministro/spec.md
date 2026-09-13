# Cadena de suministro (repo OSS)

## Purpose

El repo público no debe ser un vector vía npm o crates. Lockfiles, CI de auditoría y releases firmadas. Sin secretos ni medios con copyright.

## Requirements

### Requirement: Lockfiles canónicos
SHALL commitear `package-lock.json` (o `pnpm-lock.yaml` si se elige pnpm en el scaffold) y `Cargo.lock`. CI instala con `npm ci` / `cargo --locked`, nunca `npm install` suelto en release. Prohibido `*` y rangos en dependencias de runtime salvo el propio workspace.

### Requirement: Auditoría en CI
Cada PR y `main`:

- `npm audit --omit=dev` con umbral configurable (fail en critical/high de runtime)
- `cargo audit` (o `cargo deny advisories`)
- licencia: no GPL accidental en el binario si el proyecto es más permisivo (la licencia del repo se decide en el scaffold; deny.toml la refleja)

Dependabot o Renovate: PRs de parches. Tauri y plugins oficiales se suben con change SDD, no en silencio.

### Requirement: Scripts de instalación
MUST NOT ejecutar `postinstall` de terceros que bajen binarios opacos. Binarios nativos: crates Tauri / toolchain Rust. `package.json` scripts solo llaman a Vite/Tauri/eslint.

### Requirement: Secretos y contenidos
`.gitignore` de `node_modules`, `target`, cache de medios, `.jwpub` de usuario, `.env`. No hay tokens de JW. Releases: updater ya firmado (`seguridad`). SBOM (`npm sbom` / `cargo cyclonedx`) MAY adjuntarse al release.

### Requirement: Contribuciones
PR desde fork: CI igual. No se mergea lockfile “a mano” sin diff revisable. `CODEOWNERS` para `/src-tauri` y `/openspec`.
