# Delta — Tema (029)

## ADDED Requirements

### Requirement: Paleta cerrada de acento y densidad en Configuración
`accent` MUST ser uno de: `blue` | `teal` | `violet` | `amber` (defecto `blue`). No hex libre. `density` MUST ser `compact` | `comfortable` (defecto `compact`). Configuración SHALL exponer theme (ya), acento y densidad. Un valor fuera de la paleta MUST rechazarse al persistir. El cambio es en caliente y MUST NOT recolorear auditorio ni orador.

#### Scenario: Acento teal
- GIVEN consola en tema claro
- WHEN elige acento `teal`
- THEN pestañas y botón primario usan el token teal
- AND el auditorio sigue negro / con su medio
