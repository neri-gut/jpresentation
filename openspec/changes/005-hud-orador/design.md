# Design: Orador

```
AudienceFrame ──clone──► Speaker compositor
                              ├ capa media (igual que auditorio)
                              ├ capa mensaje (franja, opcional)
                              └ capa HUD (siempre)
                                   título | mm:ss o +desfase
                                   barra inferior verde→ámbar→rojo
```

Un solo `OutputState` de medio. El orador no pide otro decoder: misma textura / mismo clock. HUD y mensaje son overlays locales de esa ventana.

`timer.hud.warn_at`: max(1 min, 20 % del asignado).
`timer.hud.position`: bottom-right compacto; barra full-width 6–8 px.
