# Proposal: Himnario permanente, guía semanal y cronómetro encadenado

## Intent

Fijar el flujo operativo que el usuario definió:

1. Canciones: listar claves `sjjm` del idioma y descargar **todo** el himnario como biblioteca permanente; el programa solo enlaza números.
2. Multimedia: de la guía de actividades de esta semana, bajar solo imágenes/vídeos de esa semana.
3. Cronómetro: mostrar los horarios de esa guía.
4. Al terminar una sección, pasar solo a la siguiente.

## Scope

In scope: specs de canciones, multimedia, programa, cronómetro y este change.
Out of scope: código, barrido histórico de todos los `mwb`, autoplay de vídeo al cambiar de parte.

## Approach

Dos almacenes: `hymnal/{lang}` permanente vs `week/{lang}/{date}` temporal. Un `MeetingWeek` une números de cántico, `MediaRef` y partes del cronómetro.
