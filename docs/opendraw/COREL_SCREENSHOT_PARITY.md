# OD-015 — OpenDraw AI: corrispondenza con le schermate CorelDRAW 2026

## Criterio di accettazione

Riferimenti aggiornati il 2026-10-12: nove PNG CorelDRAW recuperabili dai dieci screenshot dichiarati e il video Registrazione 2026-10-12 162059.mp4 (1914×1078, 30 fps, 60,83 s). La decima immagine non risulta disponibile tra gli allegati recuperati. L'obiettivo è **replicare con alta fedeltà la disposizione e il flusso di lavoro** del riferimento, senza riutilizzare icone, logo, font proprietari o codice dell'applicazione commerciale.

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

## Iterazione operativa del 2026-10-12

Stato: implementazione da collaudare, parità ancora parziale. Nessun materiale privato o asset Corel viene pubblicato.

- Screenshot completo: barra standard circa 33 px, barra contestuale circa 43 px, toolbox circa 30 px, palette circa 20 px; sono misure del riferimento a scala 100%, non un risultato già verificato su OpenDraw.
- Video campionato ogni 3 s: 0–6 s testo e pannello Proprietà; 9–18 s elenco pannelli; 21 s Oggetti; 24 s Pagine; 27–39 s ricerca; 42–48 s rettangolo, selezione, riempimento e contorno; 51–57 s menu contestuale. Non si replica il contenuto del documento mostrato.
- Barre OpenDraw più compatte (33/40 px), icone esistenti e attribuite nella barra standard, scorrimento su finestre strette; toolbox con pulsanti contenuti nella colonna, separatori compatti e triangoli accessibili. Flyout di puntatore, modifica nodi, ritaglio e disegno ricomposti usando solo strumenti registrati. Corretto il gruppo Ritaglia invisibile: cropImage è un tool interno senza metadati di catalogo, quindi il gruppo mostra Coltello/Forbici/Gomma; ritaglio bitmap disponibile dal menu e dai controlli dell'immagine.
- Barra proprietà: font/faccia/corpo e B/I/U per testo selezionato o in modifica; grassetto e corsivo usano le vere facce del font, segnalando quelle mancanti; sottolineatura conserva il comportamento per intervalli di testo. Informazioni immagini collegate per bitmap; spessore contorno editabile; geometria e riempimento esistenti mantenuti. Nessuna parità dichiarata per testo senza oggetti, docking arbitrario o tutti gli effetti Corel.
- Pannello Classica richiudibile con × e riapribile dalle linguette verticali con nomi completi; palette più densa con clic fill/stroke e swatch documentali preservati.
- Regressione workspace: l'asserzione obsoleta toolbar_advanced=true era incompatibile con il layout compatto della Alpha 3. Verifica ora una sola colonna e strumenti compatti.
- Corretto il blocco Clippy nonminimal_bool nel comando dei livelli con una semplificazione boolean equivalente.
- Pipeline desktop: niente cancellazione della build precedente; macOS dipende dal completamento positivo di build/test/package Windows x64. Aggiunti test workspace OpenDraw e upstream, migrazione, shell e profilo macOS.

Baseline: SHA 868c73e9faa0a53a8ad6b188760be0a583c8e636. Run desktop 38258069820 completato con entrambi gli artifact, ma run workspace 38258069935 fallito. Gli artifact baseline non contengono questa iterazione. Verifica runtime e confronto visivo della nuova build ancora richiesti prima di dichiarare pronta Alpha 3.
