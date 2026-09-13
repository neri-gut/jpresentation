# Arquitectura

## Purpose

Contrato de arquitectura de JPresentation: Tauri 2 estable, Vue 3 + TypeScript en la consola, Rust como dueño de E/S, medios, monitores y SQLite. El frontend no toca el disco ni la red de contenidos salvo a través de comandos e eventos tipados. El núcleo de dominio no depende de ventanas de escritorio para poder portarse a móvil. Capas, DTOs, límite de parámetros y errores: `specs/practicas`.

## Requirements

### Requirement: Stack congelado para la cimentación
El sistema SHALL usar Tauri 2.11.x estable (no 3.0 alpha), Vue 3 con `<script setup>` y TypeScript estricto, Vite, Pinia y Rust según el template oficial `vue-ts`. SQLite SHALL operar en modo WAL con migraciones versionadas en Rust.

#### Scenario: Scaffold oficial
- GIVEN un entorno con Rust, Node y WebView del SO
- WHEN se crea el proyecto con `create-tauri-app` plantilla `vue-ts`
- THEN el identificador es `org.jpresentation.app`
- AND `tauri.conf.json` declara capabilities mínimas por superficie

### Requirement: Núcleo de dominio sin UI
La lógica de reunión, perfiles, programa, catálogo, cache de medios y reloj SHALL vivir en crates o módulos Rust independientes de Tauri windowing. Tauri commands son adaptadores. Vue es otro adaptador.

#### Scenario: Añadir un comando no toca el dominio
- GIVEN el crate de dominio `meeting`
- WHEN se expone un comando `program_start_part`
- THEN el comando valida IPC y delega
- AND no contiene reglas de desfase ni de plantilla

### Requirement: Separación consola / salida
El sistema SHALL separar la consola del operador de las superficies de salida (auditorio y cronómetro del orador). El estado de lo proyectado SHALL vivir en Rust (`OutputPort`) y replicarse por eventos Tauri. Semántica: `specs/sincronizacion`.

#### Scenario: Reinicio de la consola
- GIVEN el auditorio mostrando una imagen
- WHEN la ventana de operador se recarga
- THEN el auditorio no parpadea ni se cierra
- AND al reconectar, la consola refleja el medio actual

### Requirement: Comandos tipados y permisos mínimos
Cada capacidad nativa SHALL exponerse como comando Rust con tipos serde + interfaces TypeScript a la par. Las capabilities SHALL limitar fs, http, dialog y ventana al scope necesario. El webview del auditorio MUST NOT tener comandos de escritura en disco.

#### Scenario: Webview de auditorio restringido
- GIVEN un contenido web mostrado en auditorio
- WHEN ese webview intenta invocar un comando de fs o de descarga
- THEN Tauri lo rechaza por capability
- AND queda registro en el log de la consola

### Requirement: Persistencia local
El sistema SHALL persistir perfiles, programación, preferencias de monitores, atajos, provider activo, idioma de contenido y estado de descargas/cache en SQLite bajo `app_data_dir`. Preferencias volátiles de UI MAY usar `tauri-plugin-store`.

#### Scenario: Cerrar y abrir
- GIVEN un perfil con canciones 38 / 99 / 112 y un programa de entre semana
- WHEN el operador cierra y vuelve a abrir
- THEN recupera perfil, canciones y programa
- AND las rutas de cache se revalidan; si faltan, se marcan descargables sin romper el arranque

### Requirement: Rendimiento predecible
Ver `specs/rendimiento`. Indexado, thumbnails y descargas fuera del hilo de UI. Listas virtualizadas. Miniaturas por hash. Un decoder de stage.

#### Scenario: Lista de 150 canciones
- GIVEN el catálogo indexado
- WHEN el operador filtra por texto
- THEN el filtrado ocurre en memoria o SQL y la lista no monta 150 nodos pesados
- AND el scroll se mantiene fluido en hardware típico de salón
