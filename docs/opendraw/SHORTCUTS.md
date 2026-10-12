# OD-008 — Scorciatoie familiari a chi usa CorelDRAW

La compilazione OpenDraw AI (feature Cargo `opendraw`) propone al primo avvio il set di scorciatoie **OpenDraw Corel-style**. Il set usa il sistema nativo di VectorCraft, non una gestione parallela dei tasti.

| Tasto | Strumento OpenDraw AI | Obiettivo di familiarità |
|---|---|---|
| F5 | Matita (Pencil) | Disegno a mano libera, iniziale |
| F6 | Rettangolo | Rettangolo |
| F7 | Ellisse | Ellisse |
| F8 | Testo (Type) | Testo |
| F10 | Selezione diretta (Direct Selection) | Prima approssimazione alla modifica di nodi/forme |

**Nota:** F5 usa lo strumento Matita, non è ancora la riproduzione completa della Mano libera di CorelDRAW. F10 usa Selezione diretta: comportamenti di editing dei nodi da verificare nel collaudo. La mappatura non equivale alla certificazione di parità funzionale.

Per F5, F6 e F7 gli shortcut originari dei pannelli (pennelli, colore e livelli) vengono liberati dalla mappatura automatica di OpenDraw; i pannelli rimangono raggiungibili dai menu. Le personalizzazioni possono modificare liberamente i tasti.

## Ripristino e personalizzazione

- `Modifica → Scorciatoie da tastiera` (la traduzione completa della UI è ancora in corso) permette di scegliere il set originale VectorCraft o quello Corel-style.
- `shortcuts.preset {"name":"VectorCraft Defaults"}` ripristina i tasti originari; `shortcuts.preset {"name":"OpenDraw Corel-style"}` torna alla modalità familiare.
- Le scorciatoie personali si possono salvare e importare nel profilo separato OpenDraw. Le build normali di VectorCraft non includono il set OpenDraw.

## Verifiche

```sh
cargo fmt --all -- --check
cargo test --locked -p vectorcraft-ui-egui --features opendraw corel_style_shortcuts_activate_tools_without_panel_conflicts --lib
cargo test --locked -p vectorcraft-ui-egui presets_are_the_defaults --lib
```

Successivamente: completare scorciatoie per zoom, selezione, ordinamento, allineamento, raggruppamento, operazioni di intersezione, forme e proprietà. Solo dopo un confronto operativo CorelDRAW 2026 e test dei conflitti confermeremo il comportamento finale.
