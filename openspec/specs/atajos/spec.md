# Atajos de teclado

## Purpose

Dejar el **registro de acciones** listo para enlazar teclas más adelante. v1 no asigna atajos de fábrica y no obliga al operador a configurar ninguno. Cuando se implementen, serán por perfil y no exigirán reescribir Play, cronómetro ni `OutputPort`.

## Requirements

### Requirement: Catálogo de acciones, no teclas
El dominio SHALL exponer `ShortcutRegistry` con ids estables (`timer.next`, `output.pause`, `songs.playStart`, …). Cada acción apunta a un comando ya existente. MUST NOT incrustar `Ctrl+…` en Vue ni en Rust de reunión.

v1: el registro existe (aunque la UI de asignación llegue después). Combinaciones por defecto = **ninguna**.

#### Scenario: Change futuro
- GIVEN el registry y `timer.next` ya implementado
- WHEN un change posterior guarda `Space` → `output.pause` en el perfil
- THEN pulsar Space ejecuta pausa
- AND no se toca `MeetingClock` ni `AudioPort`

### Requirement: Persistencia prevista
Las asignaciones, cuando existan, SHALL vivir en el perfil. Un perfil nuevo nace sin atajos. La consola es la única superficie que los recibe; auditorio y orador no capturan teclas de operador.
