# OD-012 — Clic sulla tavolozza come in CorelDRAW

Obiettivo: conservare il gesto di lavoro familiare agli utenti CorelDRAW senza copiare materiali proprietari.

## Modalità OpenDraw AI

- **Clic sinistro** su un campione colore: modifica il **riempimento** degli oggetti selezionati, indipendentemente dal proxy attivo.
- **Clic destro** su un campione colore: modifica il **contorno**, indipendentemente dal proxy attivo.
- Il clic destro non cambia la selezione dei campioni del pannello e non cambia il proxy attivo.
- **Nessun colore** ([None]): il clic destro rimuove il contorno e lascia il riempimento invariato.
- **Doppio clic** e trascinamento continuano a usare l'interazione precedente per modificare e riordinare campioni.
- **Shift e Command/Ctrl + clic sinistro** mantengono la selezione multipla dei campioni senza dipingere.

## Separazione dal progetto upstream

La modifica è compilata esclusivamente con la feature `opendraw`. Con la normale build VectorCraft resta invariata la logica originale: clic sul campione per dipingere il proxy attivo; Alt per dipingere l'altro proxy.

## Test

```sh
cargo fmt --all -- --check
cargo test --locked -p vectorcraft-ui-egui --features opendraw opendraw_swatch_primary_sets_fill_secondary_sets_stroke --lib
cargo test --locked -p vectorcraft-ui-egui --features opendraw panels::swatches::tests --lib
cargo test --locked -p vectorcraft-ui-egui panels::swatches::tests --lib
```

## Limiti

La modifica riguarda **il pannello Campioni**, non ancora una tavolozza verticale CorelDRAW lungo il bordo della finestra; l'eventuale barra dedicata avrà lo stesso comportamento. I test automatici non sostituiscono il collaudo visivo e del mouse su Windows 11/macOS. La gestione degli spessori dei contorni rimane distinta dalla scelta del colore.

Questa PR è indipendente dalle altre modifiche sulle aree di lavoro, prestampa e test di affidabilità. Non contiene documenti privati.
