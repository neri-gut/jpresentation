# Visión y alcance

## Purpose

Definir qué es JPresentation, a quién sirve y qué se migra desde JMultimedia 2.8.01. JPresentation es una aplicación multiplataforma para operar reuniones en Salón del Reino o Asamblea: controla canciones, multimedia, cronómetro, textos bíblicos y carteles, con salida a auditorio y opcionalmente al orador.

JPresentation **no** es un producto oficial ni está afiliado a Watch Tower Bible and Tract Society ni a la Congregación Cristiana de los Testigos de Jehová. Es un complemento operativo de la aplicación oficial (JW Library), no un sustituto.

El grabador de audio de 2.8.01 **queda fuera de alcance**.

## Requirements

### Requirement: Nombre de producto
El sistema SHALL presentarse como **JPresentation** en título de ventana, instalador, Acerca de e identificador de paquete `org.jpresentation.app`.

#### Scenario: Acerca de
- GIVEN la consola abierta
- WHEN el operador abre Acerca de
- THEN ve el nombre JPresentation, el stack Tauri/Vue/Rust y el aviso de no afiliación

### Requirement: Paridad funcional con JMultimedia 2.8.01 (salvo grabador)
El sistema SHALL ofrecer las capacidades operativas observadas en JMultimedia 2.8.01 excepto la grabación de audio: canciones de reunión (inicio/central/final y catálogo), cronómetro de asignaciones, explorador multimedia (vídeo/imagen/audio/documento), Biblia en pantalla, editor de carteles de texto, navegador web embebido, perfiles por congregación/idioma y configuración de monitores.

#### Scenario: Operador reconoce el flujo
- GIVEN un operador que ya usa JMultimedia 2.8.01
- WHEN abre JPresentation por primera vez
- THEN encuentra las mismas áreas de trabajo (pestañas + panel de control derecho + salida de auditorio)
- AND no ve controles de grabación
- AND puede completar una reunión de entre semana o de fin de semana sin depender de Windows ni de .NET

### Requirement: Grabador expresamente retirado
El sistema MUST NOT incluir UI, comandos, permisos de micrófono ni carpetas Record destinadas a grabar la reunión.

#### Scenario: Panel derecho
- GIVEN la consola en cualquier pestaña
- WHEN el operador mira el panel derecho
- THEN no existe el bloque Grabador de 2.8.01
- AND el espacio lo ocupan Canciones, Multimedia, Biblia (si aplica) y Cronómetro

### Requirement: Escritorio ahora, móvil después
v1 SHALL publicarse para Windows, macOS y Linux. La arquitectura MUST permitir añadir targets Android e iOS de Tauri 2 sin reescribir la lógica de reunión, medios, cronómetro ni perfiles. Lo que cambia en móvil es la capa de superficie (una pantalla, sin escritorio extendido).

#### Scenario: Mismo núcleo en otro target
- GIVEN los puertos de dominio (MediaProvider, OutputPort, MeetingClock, ProfileStore)
- WHEN se añada un runtime móvil
- THEN esos puertos se reutilizan
- AND solo se sustituye el adaptador de ventanas/monitores

### Requirement: Aprovechar el equipo anfitrión
El sistema SHALL adaptar hilos de E/S, decodificación multimedia, generación de miniaturas, descargas y base de datos a la CPU, RAM, GPU y red disponibles. En equipos modestos SHALL degradar con gracia (menos previews, una descarga a la vez). En equipos potentes SHALL paralelizar indexado, thumbnails, precarga y descargas con techo configurable.

#### Scenario: Equipo de baja gama
- GIVEN un PC de 2 núcleos y 4 GB RAM con un disco HDD
- WHEN el operador abre una semana con muchos medios
- THEN la UI permanece interactiva
- AND miniaturas y descargas corren en segundo plano con límite de concurrencia bajo

### Requirement: Uso en vivo seguro
El sistema SHALL priorizar que un error de UI, red o archivo no deje la pantalla del auditorio en un estado irrecuperable. Toda acción que oscurezca o cambie la salida SHALL ser reversible en un atajo o un clic.

#### Scenario: Archivo de vídeo corrupto
- GIVEN un vídeo seleccionado para el auditorio
- WHEN el decodificador falla
- THEN el auditorio muestra un fondo negro controlado o el último fotograma válido
- AND la consola muestra el error y permite volver a logo/texto del año o al medio anterior

### Requirement: Contenido oficial fuera del repositorio
El sistema MUST NOT incluir en el código, instalador o repositorio catálogos de letras, textos bíblicos con copyright, vídeos ni himnarios. El catálogo y los binarios se obtienen en tiempo de ejecución (guía, himnario, Biblia `.jwpub`) o una carpeta local. SQLite MAY guardar metadatos e índice de versículos derivado; no el paquete original en el repo.

#### Scenario: Instalación limpia
- GIVEN una instalación nueva sin cache
- WHEN el operador abre Canciones o Multimedia
- THEN puede elegir idioma/semana y pedir al provider que liste o descargue
- AND no se reproduce contenido embebido de terceros
