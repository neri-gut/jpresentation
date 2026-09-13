# Tema de la consola

## Purpose

La **consola** (operador) tiene tema claro / oscuro / sistema y acento configurables. Auditorio y orador no heredan ese tema: sus fondos son el medio, el texto diario, el texto del año o negro + HUD.

## Requirements

### Requirement: Apariencia de la UI de operador
El perfil SHALL guardar:

- `theme`: `light` | `dark` | `system` (defecto `system`)
- `accent`: token de color de la paleta (no hex libre en v1, para contraste AA)
- `density`: `comfortable` | `compact` (defecto `compact` — reunión)

Tokens CSS (`--jp-bg`, `--jp-fg`, `--jp-accent`, …). Componentes Vue no usan colores mágicos. Cambio en caliente, sin recargar el auditorio.

Contraste de texto de consola MUST cumplir WCAG AA sobre el fondo del tema.

#### Scenario: Oscuro
- GIVEN `theme = dark`
- WHEN abre la consola
- THEN pestañas y panel usan fondo oscuro
- AND el auditorio no cambia de negro a “tema oscuro”
- AND el texto diario / año siguen su propio fondo

### Requirement: No teñir el stage
El tema MUST NOT recolorear vídeos, carteles ni HUD salvo que el HUD use override propio. Identificar monitores y alertas usan sus colores de `alertas`.
