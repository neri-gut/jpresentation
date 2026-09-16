# Design

## Explorer

Main pane: CSS grid `auto-fill minmax(9rem)`. Tiles: folder / video / image, caption under thumb, `object-fit: contain` on the thumb so squares are not cropped. Folders appear in the grid (enter on click) as well as under Places.

Toolbar: Up, clickable breadcrumb (path relative to the active Place), filter, Add folder (native dialog), download week.

## Audience image

Do **not** set both `width: 100%` and `height: 100%` on the `<img>` (WebKitGTK stretches and ignores `object-fit` in a grid). Center with `max-width/max-height: 100%; width/height: auto`. Video may keep contain in a full-size box.
