# Licencia, créditos y actualizaciones

## Purpose

Repo abierto, con atribución visible. El binario se actualiza con el **plugin oficial de Tauri** leyendo GitHub Releases. No hace falta un servidor propio.

## Requirements

### Requirement: Licencia y créditos
El código SHALL publicarse con una licencia OSI (recomendado: MIT o Apache-2.0, o dual como Tauri). `LICENSE` + `NOTICE` en la raíz.

Acerca de y el paso 1 del asistente SHALL listar:

- JPresentation y que no es un producto Watch Tower  
- La licencia y que se agradece el **reconocimiento** si se reutiliza el código  
- Tecnologías: Tauri, Vue, Rust, SQLite y demás OSS del `NOTICE` (sin pegar licencias enteras)

Forks y empaquetadores MUST conservar `LICENSE`/`NOTICE` (obligación de la licencia elegida).

### Requirement: Updater = Tauri + GitHub Releases
v1 usa `tauri-plugin-updater`:

- Artefactos firmados (minisign/ed25519) en cada GitHub Release  
- Un `latest.json` (o endpoint estático equivalente) en esa Release  
- El cliente comprueba al arrancar o desde Configuración; la actualización es **opcional**  
- El paquete de update MUST NOT incluir himnario, Biblias ni `week/`

No es mejor inventar un updater propio. Winget / Flathub / App Store son **canales extra**, no sustituyen la firma de Tauri en escritorio OSS. Un servidor de updates propio solo haría falta si el repo fuera privado.

#### Scenario: Hay v1.1
- GIVEN v1.0 instalada y Release `v1.1` con `latest.json` + firmas
- WHEN el operador acepta actualizar
- THEN se descarga el paquete de su plataforma, se verifica la firma y se relanza
- AND los perfiles y caches locales siguen
