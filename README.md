# dotfiles

Configs versionnées de mon desktop **Hyprland + Noctalia** (CachyOS).

Le dépôt vit directement dans `~/.config` avec un `.gitignore` *whitelist* :
tout est ignoré par défaut, seuls les dossiers explicitement listés sont suivis.

## Contenu suivi

- `hypr/` — config Hyprland (moniteurs, raccourcis, window rules, hypridle)
- `noctalia/` — config du shell Noctalia (settings, couleurs, colorschemes)

> La config Neovim est dans un dépôt séparé : https://github.com/vyrkem/config.nvim

## Setup d'une nouvelle machine

```bash
# 1) dotfiles dans ~/.config (sans écraser l'existant)
cd ~/.config
git init -b main
git remote add origin https://github.com/Vyrkem/dotfiles.git
git fetch origin
git checkout -f main   # récupère hypr/ et noctalia/

# 2) Neovim (dépôt séparé)
git clone https://github.com/vyrkem/config.nvim.git ~/.config/nvim
```

⚠️ `hypr/monitors.conf` et certaines valeurs de `noctalia/settings.json`
(noms de moniteurs, wallpaper) sont spécifiques à chaque machine — à ajuster
après le clone.

## Workflow

```bash
git -C ~/.config add -A && git -C ~/.config commit -m "..." && git -C ~/.config push
```
