# Delta — Proveedor de medios (032)

## ADDED Requirements

### Requirement: Adaptador de catálogo en Rust
`PublicationCatalog` SHALL consultar GETPUBMEDIALINKS (`langwritten`, `pub`, `issue` YYYYMM, `output=json`, `track` si aplica) y devolver `CatalogHit` (checksum, tamaño). Vue MUST NOT recibir la URL. Host y path MUST vivir solo en el crate/módulo adaptador.

`mwb` MAY reintentar **una** vez el mes anterior si el issue del lunes da 404 (publicación bimensual). `w` no. Checksum acertado en disco = no hay segundo GET del binario.

#### Scenario: Issue bimensual
- GIVEN lunes 5 oct 2026 y `mwb` de oct 404
- WHEN se pide esta semana
- THEN se prueba `202609` (o el mes anterior)
- AND si existe, se usa
- AND no se recorren 2025–2024
