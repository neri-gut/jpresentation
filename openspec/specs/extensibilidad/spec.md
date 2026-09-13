# Extensibilidad y plugins

## Purpose

Permitir añadir proveedores de medios, plantillas de reunión, destinos de salida, idiomas y, más adelante, un runtime móvil, sin refactorizar una función principal ni un `lib.rs` monolítico.

## Requirements

### Requirement: Puertos de dominio estables
El núcleo SHALL exponer traits (o equivalentes) como mínimo:

- `MediaProvider` — orquesta catálogo + parser + resolver
- `PublicationCatalog` — localiza `.jwpub`/`.epub` por idioma, símbolo y periodo
- `ScheduleParser` — extrae `MeetingWeek` de una publicación
- `MediaResolver` — resuelve `jwpub-media://`, embebidos y claves de catálogo a cache
- `MediaCache` — cache semanal y otros binarios no permanentes
- `HymnalLibrary` — himnario `sjjm` por idioma (permanente)
- `AudioPort` — dispositivo auditorio, máster, canales futuros
- `ShortcutRegistry` — ids de acción; bindings después
- `OutputPort` — mostrar/ocultar medio, texto, Biblia
- `MeetingClock` — programa y desfase
- `ProfileStore` — perfiles y settings
- `BibleSource` — índice y `lookup(ref)` sobre el `.jwpub` de Biblia del perfil
- `ReferenceHarvest` — citas desde guía, publicaciones o CSV
- `PlatformSurface` — crear superficies operator/audience/speaker según el SO

Añadir un provider o un destino MUST ser implementar un puerto e registrarlo, no editar el flujo de reunión.

#### Scenario: Nuevo origen de medios
- GIVEN un `MediaProvider` adicional (p. ej. solo carpeta local o un origen futuro)
- WHEN se registra en el contenedor de la app
- THEN Canciones y Multimedia listan ese origen
- AND el código de proyección no cambia

### Requirement: Registro, no condicionales eternos
El arranque SHALL componer implementaciones vía un registry (`ProviderRegistry`, `FeatureModule` en Vue). MUST NOT proliferar `if provider == "jw" { ... }` dentro de comandos de reproducción o del cronómetro.

#### Scenario: Reproducir una canción
- GIVEN un ítem con `provider_id` + `media_id`
- WHEN el operador pulsa Play
- THEN `OutputPort` recibe un `MediaHandle` ya resuelto por el provider
- AND el reproductor no conoce JW ni el sistema de archivos

### Requirement: Features Vue desacopladas
Cada pestaña SHALL ser un feature module (`songs`, `timer`, `media`, `bible`, `text`, `browser`) con store Pinia propio. El shell solo registra rutas/pestañas. Un módulo nuevo MUST poder añadirse sin reescribir `OperatorShell` salvo una línea de registro.

### Requirement: Plugins Tauri opcionales
Capacidades de SO (dialog, fs, http, window-state, updater, y mañana biometric o share en móvil) SHALL entrar como plugins oficiales o crates internos. El dominio MUST compilar en tests sin WebView.

#### Scenario: Test de desfase
- GIVEN `MeetingClock` en un test Rust
- WHEN una parte de 10 min termina a los 11:20
- THEN el desfase es +1:20
- AND el test no instancia Tauri

### Requirement: Listo para Android/iOS
`PlatformSurface` en escritorio usa `WebviewWindow` + monitores. En un change futuro de móvil SHALL mapear audience a pantalla completa de dispositivo y operator a la UI táctil. El resto de puertos MUST permanecer.

#### Scenario: Un solo panel táctil
- GIVEN un runtime móvil futuro
- WHEN no hay escritorio extendido
- THEN existe una vista de previsualización de salida dentro del operador
- AND cronómetro/medios siguen usando `MeetingClock` y `MediaProvider`

### Requirement: Configuración de extensión
Habilitar o deshabilitar un provider o un módulo SHALL ser setting de perfil o flag de compile-time (`feature` de Cargo / flag de UI), no un parche al núcleo.
