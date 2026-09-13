# Himnario `sjjm` (2026-09-12)

GET sin `track`, `langwritten=S|E`:

- MP3/MP4 tracks **1–163** = cánticos de reunión
- 164–499 = no hay
- ~600–663 = los mismos con audiodescripción (JPresentation v1 los ignora)
- 163 + 163 = 326 pistas en el JSON crudo
- Títulos en el API; no hace falta `sjj` RTF
- MP4 labels: 240p, 360p, 480p, 720p — tope actual 720p
- Track 200 → 404

Decisiones: ver `specs/canciones` y change `008`.
