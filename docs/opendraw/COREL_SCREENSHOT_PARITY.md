# OD-015 — OpenDraw AI: corrispondenza con le schermate CorelDRAW 2026

## Criterio di accettazione

Sono state fornite quattro schermate: le prime due di OpenDraw Alpha 2 e le altre due di CorelDRAW 2026, una con pagina vuota e una con oggetti. L'obiettivo è **replicare con alta fedeltà la disposizione e il flusso di lavoro** del riferimento, senza riutilizzare icone, logo, font proprietari o codice dell'applicazione commerciale.

Dimensioni e contenuto non possono essere identici *pixel per pixel* finché il motore OpenDraw non implementa le stesse funzionalità di CorelDRAW: non bisogna confondere corrispondenza della disposizione con compatibilità documentale.

## Interventi in questa PR

- **Aspetto chiaro predefinito**: stessa famiglia di toni chiari del riferimento (top bar, barra proprietà, canvas, pannelli). Lo schema scuro resta selezionabile. Aggiornamento delle preferenze Alpha 2 una volta sola: vecchio layout recuperabile da `Alpha 2 (layout precedente)`, scorciatoie e configurazioni salvate preservate.
- **Home e Info OpenDraw**: testi originali in italiano, nessun collegamento promozionale ArtCraft/VectorCraft/Discord nella schermata iniziale né nella scheda principale Info; attribuzioni MIT/Apache-2.0, ISC e OFL conservate nei file di licenza e nella finestra Info.
- **Barra strumenti sinistra**: colonna stretta, strumenti operativi principali in ordine da lavoro Corel: Puntatore, Forma, Ritaglio, Zoom, Disegno, Rettangolo/Ellisse, Testo, Modifica, Colore e Navigazione. Varianti collegate a strumenti reali, nessun pulsante simulato.
- **Barra proprietà superiore dedicata**: modifica reale di coordinate X/Y, larghezza e altezza tramite `object.setBounds`; accesso a rotazione, riempimento e contorno. Senza selezione: dimensioni reali della pagina, document setup, unità e pannelli.
- **Pannello destro**: nomi Oggetti, Proprietà e Librerie; vecchi nomi `Layers` e `Properties` ancora accettati come alias dei comandi.
- **Navigazione multipagina in basso**: aggiungi pagina, pagina precedente/successiva, schede delle pagine. Comandi effettivi `artboard.new` e `view.goToArtboard`.
- **Protezione upstream**: le modifiche sono specifiche della feature `opendraw`, non sostituiscono i testi o gli strumenti in VectorCraft standard.

## Cosa richiede ancora iterazione visiva e funzionale

- Riprogettazione delle icone originali per una somiglianza nella gerarchia, senza copie proprietarie.
- Dimensioni esatte dei pannelli su display 100–200% e tooltip / layout con selezioni differenti.
- Pannello Oggetti con ricerca, albero Pagina → Livello → Oggetti e gestione PowerClip completa.
- Tutti i controlli contestuali (testo, bitmap, PowerClip, stili) e interazione precisa di CorelDRAW.
- Immagini/PDF/CDR: importazione CDR 2026 e PDF/X-4 certificato non implementati in questa modifica.

## Verifica

```sh
cargo fmt --all -- --check
cargo test --locked -p vectorcraft-ui-egui --features opendraw opendraw_home_has_original_brand_and_no_upstream_links --lib
cargo test --locked -p vectorcraft-ui-egui --features opendraw alpha_two_upgrade_uses_light_classic_and_saves_old_toolbar --lib
cargo test --locked -p vectorcraft-ui-egui --features opendraw tests_home --lib
cargo test --locked -p vectorcraft-ui-egui --features opendraw menus::tests --lib
cargo test --locked -p vectorcraft-ui-egui tests_home --lib
cargo build --release --locked -p vectorcraft --features opendraw
```

Prima di considerare la release accettata serve un collaudo con screenshot Windows 11 della finestra 1600×900 con documento vuoto e con un oggetto selezionato, confrontato con i riferimenti dell'utente. Nessun file CDR/PDF privato viene caricato in CI o su GitHub.
