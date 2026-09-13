# Primer arranque y asistente de perfil

## Purpose

Nadie aterriza en una consola vacía. Mínimo **un perfil**. Crear perfil (el primero o uno nuevo) es un **asistente ordenado**, no diez pantallas sueltas.

## Requirements

### Requirement: Día de reunión
Cada perfil guarda, en zona horaria **local del equipo**:

- entre semana: `weekday` + `time` (ej. martes 19:00)
- fin de semana: `weekday` + `time` (ej. domingo 10:00)

La “semana de guía” es **lunes 00:00 – domingo 23:59** local (la del `mwb`).  
“Hoy es entre semana” = el `weekday` entre semana.  
“Hoy es fin de semana” = el `weekday` de fin de semana.  
Si hoy no es ninguno, Cronómetro ofrece plantilla o la próxima fecha configurada.

### Requirement: Un perfil al menos
Instalación limpia → asistente obligatorio → perfil `Default` (nombre editable). No se entra a la consola sin perfil. Se pueden crear más después (mismo asistente, paso 1 = nombre).

Arranque siguiente: último perfil usado.

### Requirement: Pasos del asistente
Orden fijo; se puede Atrás. Cancelar el primero no deja la app usable (vuelve al paso 1). Cancelar un perfil extra no borra los demás.

1. Bienvenida + licencia OSS + créditos de tecnologías  
2. Nombre del perfil  
3. Idioma de UI + idioma de contenido  
4. Tema de consola (claro/oscuro/sistema + acento)  
5. Días y horas de las dos reuniones  
6. Monitores (auditorio / orador / identificar)  
7. Cronómetro: modo cuenta, estilo HUD / countdown pre-reunión  
8. Fondo al proyectar Biblia (sólido / imagen)  
9. Descargas iniciales (opcionales, con progreso y error manual):  
   - himnario del `content_langwritten`  
   - guía de **esta** semana + medios de esa guía  
10. Resumen → Abrir consola  

El paso 9 MAY omitirse (“hacerlo luego”). Sin red: Continuar sin archivos; quedan `pending` en Canciones/Multimedia.

Nuevo perfil: mismos pasos (1 se resume a créditos cortos).

#### Scenario: Sala nueva
- GIVEN primer launch
- WHEN termina el asistente en español / contenido `S` / martes 19:00
- THEN existe un perfil
- AND si el paso 9 tuvo red, hay himnario y `MeetingWeek` de esta semana
- AND si falló un vídeo, el diálogo de cache dejó elegir reintentar o seguir
