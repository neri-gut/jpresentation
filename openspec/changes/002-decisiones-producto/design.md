# Design: Decisiones de producto

## Nombre e identidad

- Display name: JPresentation
- Identifier: org.jpresentation.app
- DB file: jpresentation.db
- Carpeta de proyecto prevista: `jpresentation/`

## Grabador

Eliminado del panel, de capabilities, de perfiles y del árbol Multimedia. No se pide micrófono. El bloque UI de 2.8.01 no se porta.

## Medios

Hexágono:

```
Vue feature (songs/media)
        │ invoke
        ▼
Tauri command (adapter)
        ▼
MediaProvider (trait)
   ┌────┴────┐
   ▼         ▼
JwProvider  LocalFolderProvider
   │         │
   └────┬────┘
        ▼
   MediaCache (disco del perfil)
        ▼
   OutputPort → audience
```

El adaptador JW habla con el catálogo público de contenido multimedia en el idioma del perfil. Endpoints y formato **no** se clavan en la spec: si cambian, solo se toca el adaptador. Descargas explícitas, cache local, sin login, sin redistribución.

## Extensión

- Rust: traits + registry
- Vue: feature modules registrados
- SO: plugins Tauri
- Móvil: otro `PlatformSurface` (v1 no lo implementa)

`lib.rs` solo ensambla. Tests de dominio sin WebView.

## HTTP

Capability de operador con allowlist que definirá el adaptador en su change. Audience/speaker sin http de contenidos.
