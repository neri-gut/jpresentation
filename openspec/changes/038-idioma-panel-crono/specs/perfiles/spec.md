# Delta — Perfiles (038)

## MODIFIED Requirements

### Requirement: Idiomas solo en Configuración
`ui_locale` y `content_langwritten` SHALL editarse **solo** en Configuración. MUST NOT haber un menú Idiomas en la barra del operador. Cambiar `content_locale` MUST ser lo que usa el fetch de `mwb`/`w`.

#### Scenario: Cambia el contenido a español
- GIVEN un perfil con `content_locale = E`
- WHEN el operador elige español (`S`) en Configuración y pulsa «Obtener guía»
- THEN el catálogo se consulta con `langwritten=S`
- AND las filas de plantilla (si no hay HTML) están en español
