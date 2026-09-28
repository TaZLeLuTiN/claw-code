//! La génération Ollama se lit EN FLUX et n'est abandonnée que sur un SILENCE — jamais sur une durée prédite.
//!
//! Le constat (2026-09-27, `cc-symphony`) : `gemma4:31b` échouait 2 fois sur 2 (« error sending request ») après
//! plus de 120 s, Ollama vivant. `compute_timeout` prédisait la durée avec des vitesses ÉCRITES EN DUR (`gemma4:31b` =
//! 12 tok/s) et 600 jetons de sortie. Mesuré le 2026-09-28 sur le Mac (prompt de 4,6 Ko) : **6,3 tok/s** et **1942
//! jetons** de sortie, 341 s — le délai prédit (~366 s) tenait à 7 % près. Une prédiction fausse de moitié sur la
//! vitesse et du triple sur la longueur ne se corrige pas en changeant ses constantes : on retire la prédiction.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::time::Duration;

use harnais_mcp::tools::{generer_en_flux, Flux, Silences};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

fn ligne(reponse: &str, fini: bool) -> String {
    if fini {
        format!("{{\"response\":\"{reponse}\",\"done\":true,\"eval_count\":20,\"eval_duration\":4000000000}}\n")
    } else {
        format!("{{\"response\":\"{reponse}\",\"done\":false}}\n")
    }
}

// ── l'assemblage, pur ────────────────────────────────────────────────────────────────────────────────────────
#[test]
fn le_flux_se_reassemble_meme_coupe_au_milieu_d_une_ligne() {
    let tout = format!(
        "{}{}{}",
        ligne("Bon", false),
        ligne("jour", false),
        ligne("", true)
    );
    let (a, b) = tout.split_at(17);
    let mut f = Flux::default();
    f.pousser(a.as_bytes()).unwrap();
    assert!(!f.fini());
    f.pousser(b.as_bytes()).unwrap();
    assert!(f.fini());
    assert_eq!(f.texte(), "Bonjour");
}

#[test]
fn la_vitesse_est_mesuree_depuis_la_derniere_ligne_jamais_supposee() {
    let mut f = Flux::default();
    f.pousser(ligne("x", false).as_bytes()).unwrap();
    assert_eq!(
        f.vitesse_tok_s(),
        None,
        "avant la fin : aucune vitesse, pas une valeur par défaut"
    );
    f.pousser(ligne("", true).as_bytes()).unwrap();
    assert_eq!(f.vitesse_tok_s(), Some(5.0));
}

#[test]
fn une_erreur_dans_le_flux_est_une_erreur() {
    let mut f = Flux::default();
    let e = f.pousser(b"{\"error\":\"model not found\"}\n").unwrap_err();
    assert!(format!("{e:#}").contains("model not found"));
}

// ── le transport, contre un vrai serveur HTTP local ──────────────────────────────────────────────────────────
async fn serveur(lignes: Vec<(u64, String)>) -> String {
    let l = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let adresse = format!("http://{}", l.local_addr().unwrap());
    tokio::spawn(async move {
        let (mut s, _) = l.accept().await.unwrap();
        let mut tampon = [0u8; 8192];
        let _ = s.read(&mut tampon).await;
        s.write_all(
            b"HTTP/1.1 200 OK\r\nContent-Type: application/x-ndjson\r\nConnection: close\r\n\r\n",
        )
        .await
        .unwrap();
        for (attente_ms, texte) in lignes {
            tokio::time::sleep(Duration::from_millis(attente_ms)).await;
            if s.write_all(texte.as_bytes()).await.is_err() {
                return;
            }
        }
    });
    adresse
}

fn silences_courts() -> Silences {
    Silences {
        premier: Duration::from_millis(300),
        entre: Duration::from_millis(300),
    }
}

#[tokio::test]
async fn une_generation_lente_mais_continue_aboutit_quelle_que_soit_sa_duree_totale() {
    // 10 lignes à 150 ms d'écart = 1,5 s au total, CINQ fois le silence toléré : l'ancienne logique aurait coupé.
    let mut lignes: Vec<(u64, String)> = (0..9).map(|_| (150, ligne("a", false))).collect();
    lignes.push((150, ligne("", true)));
    let hote = serveur(lignes).await;
    let g = generer_en_flux(&hote, "m", "p", silences_courts())
        .await
        .unwrap();
    assert_eq!(g.texte, "aaaaaaaaa");
    assert_eq!(g.tok_s, Some(5.0));
}

#[tokio::test]
async fn un_silence_abandonne_et_le_message_dit_que_c_est_un_silence() {
    let hote = serveur(vec![(10, ligne("a", false)), (900, ligne("", true))]).await;
    let e = generer_en_flux(&hote, "gemma4:31b", "p", silences_courts())
        .await
        .unwrap_err();
    let m = format!("{e:#}");
    assert!(m.contains("MUET") && m.contains("gemma4:31b"), "{m}");
}

#[tokio::test]
async fn un_flux_coupe_avant_done_est_une_erreur_pas_une_reponse_partielle() {
    let hote = serveur(vec![(10, ligne("a", false))]).await;
    let e = generer_en_flux(&hote, "m", "p", silences_courts())
        .await
        .unwrap_err();
    assert!(format!("{e:#}").contains("interrompu"));
}
