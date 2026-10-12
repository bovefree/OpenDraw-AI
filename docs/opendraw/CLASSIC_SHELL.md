# OpenDraw AI Alpha 2 — Interfaccia Classica (OD-014)

## Perché l'Alpha 1 sembrava VectorCraft

Il primo eseguibile distribuiva l'applicazione esistente, con un'identità OpenDraw e una cartella preferenze distinta, ma lasciava in gran parte l'ambiente grafico originale. OD-014 introduce un'architettura UI **condizionata dalla feature `opendraw`**, senza sostituire il motore vettoriale.

## Elementi implementati

- **Identità visibile anche su Windows**: OpenDraw AI nella barra applicazione oltre al titolo del sistema.
- **Menu principali secondo il lavoro di CorelDRAW**: File, Modifica, Visualizza, Layout, Oggetto, Effetti, Bitmap, Testo, Tabella, Strumenti, Finestra, Aiuto. I comandi che funzionano sono collegati ai comandi effettivi del motore; **Tabella è esplicitamente disabilitato** perché ancora assente.
- **Barra dei comandi rapidi**: Nuovo, Apri, Salva, Annulla, Ripeti, Adatta pagina, Adatta tutto, Oggetti, Proprietà.
- **Barra contestuale delle proprietà**, **strumenti verticali a sinistra** e **pannelli Oggetti/Proprietà a destra** sono riutilizzati con l'area Classica delle PR #5/#6, già incluse nel ramo di base.
- **Tavolozza verticale a destra**: i campioni *del documento* sono rappresentati con il loro colore; sinistro applica al riempimento, destro al contorno. Non appiattisce i colori in screenshot e non modifica i campioni CMYK/spot presenti nel documento.
- Il pulsante comunità specifico della base VectorCraft non viene più mostrato nella barra principale OpenDraw.

## Portata e compatibilità

La traduzione completa dell'interfaccia (pannelli, finestre, help, finestre di esportazione) **non è ancora conclusa**. Anche se i menu principali hanno struttura familiare, non è dimostrata la parità CorelDRAW. Non sono ancora supportati CDR 2026, esportazione PDF/X-4 certificata, tabelle native, alcune funzioni PowerClip, tutte le scorciatoie e strumenti Corel.

OD-014 dipende da #5 e #6 per workspace e scorciatoie. Il comportamento Pick, le modifiche al pannello Campioni, il controllo prestampa, l'affidabilità e PowerClip dal pannello Oggetti restano in altri rami e **non fanno parte automaticamente di questa PR**.

La build standard senza `opendraw` deve rimanere invariata; il codice della shell non viene compilato in quella configurazione.

## Verifiche

```sh
cargo fmt --all -- --check
cargo test --locked -p vectorcraft-ui-egui --features opendraw classic_ --lib
cargo test --locked -p vectorcraft-ui-egui --features opendraw menu --lib
cargo test --locked -p vectorcraft-ui-egui menu --lib
cargo build --release --locked -p vectorcraft --features opendraw
```

Dopo CI: screenshot su Windows 11 con menu e palette visibili, prova di selezione oggetti, colori, salvataggio e riapertura. La distribuzione deve usare un archivio **Alpha 2** distinto da quello già scaricato, non sovrascrivere i file di prova dell'utente.
