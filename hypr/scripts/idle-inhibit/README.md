# idle-inhibit-fullscreen

Petit démon qui empêche la mise en veille pendant les jeux/sims en **plein
écran** sous Hyprland (pad, clavier/souris ou HOTAS), sans gamemode.

Il écoute le socket d'événements de Hyprland et, dès qu'une fenêtre passe en
plein écran (mode 1 ou 2 — les jeux sous gamescope sont en 2), active le
**Keep Awake de Noctalia** (`idleInhibitor enable`), qui pose un inhibiteur
Wayland au niveau du compositeur. Relâché dès qu'aucune fenêtre n'est en plein
écran.

Réimplémente la windowrule `idle_inhibit fullscreen`, cassée sur Hyprland 0.55.

## Build (à faire une fois par machine)

```sh
cargo build --release
```

Le binaire se retrouve dans `target/release/idle-inhibit-fullscreen` — c'est ce
chemin qui est lancé via `exec-once` dans `hyprland.conf`. `target/` n'est pas
versionné (voir `.gitignore`).

## Dépendances

- Hyprland (sockets `.socket.sock` et `.socket2.sock`)
- Noctalia (`qs -c noctalia-shell`, IPC `idleInhibitor`)
