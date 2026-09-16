# Design: Stage reopen + local tree

## Stage

`stage_open` copies to `media/stage/{rev}.{ext}` (rev after bump). Never overwrite a live file. `stage_close` publishes `none` first so webviews drop the handle, then deletes files in `stage/`. Audience/speaker `<img>`/`<video>` use `:key="stage.rev"`.

## Tree

`default_local_roots()`: `$HOME`, `$HOME/Desktop` or `XDG_DESKTOP_DIR`, Pictures, Videos if they are directories. `explorer_list("")` returns week folders + these + profile roots.

`path_allowed` accepts any path under those prefixes.

Left tree: roots; click expands child **directories** (not img/vid, those still flatten into the grid). `explorer_pick_root` uses a native folder dialog (`rfd`).

## Tests

- Two opens produce two different dest names.
- Home is a default root; a file under Home is allowed.
- `list_dir` still returns subdirectories.
