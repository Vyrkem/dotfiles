//! idle-inhibit-fullscreen — pont Hyprland → inhibiteur Noctalia.
//!
//! Écoute le socket d'événements de Hyprland (`.socket2.sock`). Dès qu'une
//! fenêtre est en plein écran (n'importe quel workspace), on active le Keep
//! Awake de Noctalia (`idleInhibitor enable`) ; sinon on le relâche.
//!
//! Le Keep Awake de Noctalia crée un vrai inhibiteur Wayland au niveau du
//! compositeur, ce qui neutralise TOUS les consommateurs d'`ext-idle-notify`
//! (le module idle de Noctalia inclus). Résultat : plus de mise en veille
//! pendant un jeu/sim plein écran — au pad, au clavier/souris ou au HOTAS —
//! sans dépendre de gamemode.
//!
//! Réimplémente en espace utilisateur la windowrule `idle_inhibit fullscreen`
//! (cassée sur Hyprland 0.55).

use std::env;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::process::Command;

/// Événements Hyprland susceptibles de changer l'état « plein écran visible ».
/// On ne réinterroge le compositeur que sur ceux-là ; le reste est ignoré.
const RELEVANT: &[&str] = &[
    "fullscreen",
    "openwindow",
    "closewindow",
    "movewindowv2",
    "workspace",
    "workspacev2",
    "focusedmon",
    "activewindowv2",
];

/// Dossier des sockets de l'instance Hyprland courante.
fn hypr_dir() -> String {
    let xdg = env::var("XDG_RUNTIME_DIR").expect("XDG_RUNTIME_DIR non défini");
    let his = env::var("HYPRLAND_INSTANCE_SIGNATURE")
        .expect("HYPRLAND_INSTANCE_SIGNATURE non défini (pas dans une session Hyprland ?)");
    format!("{xdg}/hypr/{his}")
}

/// Envoie une commande au socket de requêtes de Hyprland et renvoie la réponse.
/// Protocole : on se connecte, on écrit la commande (préfixe `j/` = JSON), on
/// lit jusqu'à EOF. Pas de spawn `hyprctl`.
fn hypr_request(cmd: &str) -> std::io::Result<String> {
    let mut sock = UnixStream::connect(format!("{}/.socket.sock", hypr_dir()))?;
    sock.write_all(cmd.as_bytes())?;
    let mut resp = String::new();
    sock.read_to_string(&mut resp)?;
    Ok(resp)
}

/// Interprète le champ `fullscreen` d'un client : vrai si non nul.
/// Robuste aux versions : entier (0/1/2) OU booléen selon le build de Hyprland.
fn is_fullscreen(v: &serde_json::Value) -> bool {
    match v {
        serde_json::Value::Number(n) => n.as_i64().is_some_and(|x| x != 0),
        serde_json::Value::Bool(b) => *b,
        _ => false,
    }
}

/// True si une fenêtre mappée est en plein écran, tous workspaces confondus.
/// (mode 1 = FULLSCREEN, 2 = FULLSCREEN_MAXIMIZED — gamescope est en 2.)
fn has_fullscreen() -> bool {
    let resp = match hypr_request("j/clients") {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[idle-inhibit] requête Hyprland échouée : {e}");
            return false;
        }
    };
    let clients: serde_json::Value = match serde_json::from_str(&resp) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[idle-inhibit] JSON invalide : {e}");
            return false;
        }
    };
    clients.as_array().is_some_and(|arr| {
        arr.iter().any(|c| {
            c["mapped"].as_bool().unwrap_or(false) && is_fullscreen(&c["fullscreen"])
        })
    })
}

/// Active ou relâche le Keep Awake de Noctalia via son IPC.
fn set_inhibit(on: bool) {
    let action = if on { "enable" } else { "disable" };
    let _ = Command::new("qs")
        .args(["-c", "noctalia-shell", "ipc", "call", "idleInhibitor", action])
        .status();
    eprintln!("[idle-inhibit] Keep Awake -> {action}");
}

fn main() -> std::io::Result<()> {
    // Flux d'événements du compositeur.
    let events = UnixStream::connect(format!("{}/.socket2.sock", hypr_dir()))?;
    let reader = BufReader::new(events);

    // Synchronise l'état initial (un jeu peut déjà être lancé au démarrage).
    let mut current = has_fullscreen();
    set_inhibit(current);

    // Chaque ligne a la forme `EVENT>>data`. On ne garde que le nom d'événement.
    for line in reader.lines() {
        let line = line?;
        let name = line.split(">>").next().unwrap_or("");
        if !RELEVANT.contains(&name) {
            continue;
        }
        let desired = has_fullscreen();
        if desired != current {
            set_inhibit(desired);
            current = desired;
        }
    }
    Ok(())
}
