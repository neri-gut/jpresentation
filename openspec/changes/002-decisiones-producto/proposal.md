# Proposal: Decisiones de producto (nombre, grabador, medios, extensión)

## Intent

Cerrar cuatro decisiones de producto sobre la spec viva antes de implementar la cimentación.

## Scope

In scope:
- Renombrar el producto a **JPresentation** (`org.jpresentation.app`)
- Retirar por completo el grabador de audio
- Introducir `MediaProvider` enchufable con adaptador JW.org + carpeta local
- Exigir arquitectura de puertos/plugins para extender y para móvil futuro

Out of scope:
- Implementar el adaptador JW (change posterior)
- Publicar builds Android/iOS
- Detalle de endpoints o protocolos de catálogo

## Approach

Actualizar specs vivas (ya aplicado) y dejar este change como registro. El change 001 se alinea al nuevo nombre y a los traits vacíos.
