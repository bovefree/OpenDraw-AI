# OD-013 — Accesso a PowerClip dal pannello Oggetti

## Funzioni disponibili

Nella build OpenDraw AI, il clic destro su una riga che rappresenta un gruppo con maschera di ritaglio (equivalente operativo di una cornice PowerClip) apre:

- **Edit PowerClip Contents** — seleziona il contenuto ritagliato, senza convertirlo in bitmap;
- **Edit PowerClip Frame** — seleziona il percorso della cornice, modificabile con gli strumenti vettoriali;
- **Release PowerClip** — rimuove il ritaglio conservando gli oggetti modificabili. L'operazione è annullabile.

Il comando agisce sulla specifica cornice cliccata, anche quando un altro oggetto era precedentemente selezionato sulla pagina. I comandi riutilizzano `select.set`, `object.clippingMask.editContents`, `object.clippingMask.editMask`, e `object.clippingMask.release` presenti nel motore.

## Compatibilità e limiti

Il menu contestuale esiste solo con la feature `opendraw`. Le build normali di VectorCraft non sono modificate.

Questa implementazione è un primo miglioramento dell'accesso alle **maschere di ritaglio già esistenti**: non implementa ancora l'importazione dei PowerClip CDR né la funzione `Inserisci contenuto nella cornice selezionata` per creare una nuova maschera scegliendola dal pannello. L'interazione e la fedeltà visiva richiedono test manuali Windows e Mac.

## Test

```sh
cargo fmt --all -- --check
cargo test --locked -p vectorcraft-ui-egui --features opendraw powerclip_row_commands_target_the_correct_clip_and_preserve_editability --lib
cargo test --locked -p vectorcraft-ui-egui panels::layers::tests --lib
```

Non sono stati pubblicati file CDR, PDF o font dell'utente.
