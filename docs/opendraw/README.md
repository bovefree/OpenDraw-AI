# OpenDraw AI — Piano tecnico iniziale (OD-001)

OpenDraw AI è una derivazione sperimentale di [VectorCraft](https://github.com/storytold/vectorcraft), destinata a Windows e macOS. L'obiettivo è aggiungere un flusso di lavoro per impaginazione e stampa, interfaccia in italiano e interoperabilità con CorelDRAW, preservando l'architettura Rust e la possibilità di automazione MCP.

> Stato: sviluppo sperimentale. Il fork usa ancora il codice e il marchio VectorCraft. CDR apre per ora solo miniature di pagina con avviso esplicito; l’importazione modificabile non è ancora disponibile.

## Principi tecnici

1. **Un solo motore:** utilizzare `vectorcraft-engine`, `vectorcraft-doc`, `vectorcraft-render`, `vectorcraft-pdf` e l'interfaccia egui già presenti. La vecchia demo HTML non diventa un secondo editor da mantenere.
2. **Funzioni richiamabili:** ogni nuova funzionalità che modifica il documento deve passare dal sistema di comandi, così da essere raggiungibile da GUI, CLI e MCP.
3. **File originali tutelati:** usare copie temporanee e test sintetici; non sovrascrivere automaticamente file importati in modo incompleto.
4. **Attribuzioni e licenze:** mantenere gli avvisi MIT/Apache-2.0, verificare ogni dipendenza aggiunta e non riutilizzare marchi, loghi o altri asset ArtCraft nella futura interfaccia personalizzata.
5. **Compatibilità legittima:** specifiche pubbliche, librerie con licenze compatibili e documenti di cui il tester detiene i diritti; non copiare codice, risorse o interfacce proprietarie.

## Sequenza di lavoro

| ID | Obiettivo | Criterio verificabile |
|---|---|---|
| OD-001 | Stabilire la baseline tecnica | `cargo check --locked -p vectorcraft -p vectorcraft-cli` e `cargo test --locked -p vectorcraft-format --lib` su Windows/macOS; risultati CI registrati |
| OD-002 | Identità OpenDraw AI | Nome, icone originali, dati applicazione e cartelle configurazione dedicati; test di avvio e salvataggio |
| OD-003 | Preset di stampa | Nuovo documento 1400 × 2000 mm, abbondanza configurabile, export PDF/X-4 e controllo dimensioni |
| OD-004 | Test interoperabilità | Corpus di SVG/PDF di riferimento, controlli su livelli, testi, gradienti e immagini; rapporti di perdita |
| OD-005 | Import CDR | Lettore ZIP controllato e anteprima multipagina; prossimo traguardo: parsing clean-room di root/page RIFF e importazione di forme native verificata su file autorizzati |
| OD-006 | Automazione AI | Esporre e testare workflow di impaginazione e adattamento formati usando gli strumenti MCP esistenti |

## Primo documento campione

Un manifesto sintetico, interamente originale:
- Dimensione: 1400 × 2000 mm.
- Abbondanza: 5 mm per lato, configurabile.
- Tre livelli: sfondo, elementi grafici, testo.
- Due blocchi di testo modificabili, una forma Bézier, una sfumatura e un'immagine incorporata con licenza adatta.
- Esportazioni: formato nativo, SVG e PDF/X-4.

Verifiche previste: geometria, numero e nomi dei livelli, conservazione dei testi, ritaglio, bleed, profili colore, apertura ed esportazione ripetuta. Una grafica che appare corretta a video non equivale automaticamente a un file valido per tipografia.

## Procedura di sviluppo

- Lavorare su branch `opendraw/...` e aprire pull request verso `main`; non modificare la base senza verifica.
- Prima dei cambiamenti Rust, usare le indicazioni di `AGENTS.md` e `ROADMAP.md` del progetto originario.
- Per la verifica manuale usare copie di lavori propri, senza caricare nei repository pubblici documenti personali o file di terzi.
- Questa prima CI controlla **compilabilità** e test del formato, non produce installer. Gli installer Windows/macOS, inclusa la verifica Apple Silicon, saranno una fase successiva.
