# Biblia y textos de referencia

## Purpose

Consultar la Biblia local, sacar citas de la guía y de otras publicaciones (Atalaya, libros, folletos) y dejarlas listas para el auditorio. Fondo configurable. Si el pasaje no cabe, el operador pagina el stage. También se puede importar un CSV.

## Requirements

### Requirement: Módulo Biblia permanente
El perfil elige edición (defecto `nwtsty` si existe; si 404, otra JWPUB o archivo local). Vive en `bible/{lang}/{symbol}.jwpub`. La semana no lo borra. `BibleSource.lookup(ref)` es la única lectura de versículos. UI no abre el SQLite del JWPUB.

Paquete ilegible: error + re-descarga o archivo local. Sin descifrado.

#### Scenario: Lookup local
- GIVEN `nwtsty` ES indexado
- WHEN busca `1Ti 3:16` o `1 Timoteo 3:16-18`
- THEN el texto sale del índice, sin red

### Requirement: Extraer citas de publicaciones
Las referencias salen de:

1. La **guía** ya parseada (`MeetingWeek`): enlaces `jwpub://b/…` y citas de cada parte  
2. Un **`.jwpub` / `.epub`** de Atalaya (`w`), *¡Despertad!*, libro o folleto bajado del catálogo (`GETPUBMEDIALINKS` + `pub` + `issue`) o abierto en local  
3. Un **CSV** importado  

Un extractor (`ReferenceHarvest`) recorre el paquete, lista `(ref, contexto)` — p. ej. párrafo 8 de la Atalaya — y resuelve el texto con `BibleSource`. MUST NOT usar un API de versículo online. Si una cita no está en el módulo, se deja marcada y sin inventar texto.

#### Scenario: Citas de esta semana
- GIVEN `mwb` parseado con varios `jwpub://b/…`
- WHEN el operador pulsa “preparar textos de la guía”
- THEN la cola se llena con esas referencias ya resueltas
- AND cada ítem sabe de qué parte viene

#### Scenario: Atalaya del estudio
- GIVEN el `w` de esa semana en disco
- WHEN pide extraer citas
- THEN aparecen las escrituras del artículo, agrupadas por párrafo si el parser las trae
- AND el texto se lee de la Biblia local

### Requirement: Cola lista para proyectar
Por perfil y fecha (o “lista suelta”): añadir a mano, desde la guía, desde una publicación o desde CSV. Reordenar, quitar, persistir. Estado: pendiente / en stage / ya mostrado.

### Requirement: Fondo del texto
Cada ítem o la lista usa un fondo de perfil: color sólido, degradado o imagen. Tipografía, márgenes y alineación iguales que los carteles (`specs/texto`). El orador, con texto en stage, ve **la misma página** + HUD (excepción al “solo reloj”: aquí hay recurso de texto abierto).

### Requirement: Stage y páginas
“Enviar al stage” abre el ítem en auditorio y orador. Si no cabe, páginas 1/N. **Solo el operador** pagina y cierra desde la consola. Auditorio y orador son espejo de lectura. El orador no pagina.

Cerrar (operador) → diario en auditorio, HUD solo en orador. El cronómetro MUST NOT enviar ni paginar textos.

#### Scenario: Pasaje largo
- GIVEN Salmo 83 en la cola
- WHEN el operador lo envía y pulsa siguiente
- THEN auditorio y orador muestran página 2 a la vez
- AND el orador no tiene control de página

### Requirement: CSV
Importar UTF-8. Cabecera reconocida:

```
reference,label,note
Pr 3:5-6,Confía en Jehová,párrafo 8
1Ti 3:16,Misterio sagrado,
```

`reference` obligatorio. `text` opcional: si viene, se usa tal cual; si no, `lookup`. Filas inválidas se listan y no tumba el resto. Exportar la cola al mismo formato MAY existir para compartir entre perfiles.

### Requirement: Sin copyright en el instalador
Ni Biblia ni Atalayas en el paquete de la app. Solo cache local del operador.
