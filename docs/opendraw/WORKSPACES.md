# OpenDraw AI — Aree di lavoro (OD-007)

Obiettivo: mantenere il flusso operativo familiare agli utenti di CorelDRAW, utilizzando il sistema di pannelli nativo di VectorCraft senza copiarne marchi, iconografia proprietaria o altri asset. L'interfaccia OpenDraw resta originale.

## Prima configurazione

La compilazione `--features opendraw` parte in **Classica**: strumenti a sinistra, barra contestuale, righelli, pannello livelli, barra di stato e tavolozza già disponibili nell'interfaccia. Le altre compilazioni mantengono integralmente i workspace preesistenti di VectorCraft.

## Selezione in base al lavoro

Il selettore in alto a destra o il menu **Window → Workspace** permette di passare tra:

| Area di lavoro | Impiego principale | Strumenti/pannelli evidenziati |
|---|---|---|
| Classica | Lavoro quotidiano, flusso CorelDRAW familiare | Strumenti avanzati, barra contestuale, righelli, livelli |
| Social e comunicazione | Card, caroselli, post | Strumenti essenziali, livelli, allineamento |
| Stampa e prestampa | Manifesti, volantini, PDF | Righelli, barra contestuale, proprietà e informazioni |
| Disegno vettoriale | Loghi, curve, illustrazioni | Strumenti avanzati a doppia colonna, livelli, contorni |

Selezionare un'area di lavoro **non cambia gli oggetti del documento**. Non modifica neppure preferenze di colore, luminosità o scorciatoie personali.

## Personalizzazione

Il sistema esistente offre **New Workspace…** e **Manage Workspaces…** per salvare, rinominare e cancellare le disposizioni personali. Le aree predefinite non possono essere sovrascritte. Le configurazioni personalizzate vengono salvate nelle preferenze isolate di OpenDraw AI e devono sopravvivere al riavvio.

Lo schema non pretende di riprodurre pixel per pixel l'interfaccia o le icone di CorelDRAW. Il criterio finale sarà l'equivalenza dei flussi di lavoro, verificata con test pratici e screenshot forniti con autorizzazione.

## Limiti di questa prima modifica

- Selezione delle aree di lavoro **manuale**, anche durante un progetto. L'associazione automatica di una configurazione a un tipo di documento è una funzionalità successiva, e sarà sempre disattivabile.
- L'interfaccia non è ancora interamente tradotta in italiano e gli strumenti non hanno ancora tutte le scorciatoie di CorelDRAW.
- Pannelli di prestampa mostrati non equivalgono da soli a un controllo di conformità PDF/X.
- Il comando di automazione UI `window.workspace` è già presente; OD-007 aggiunge le configurazioni, non un nuovo motore di comandi.

## Verifica automatica

```sh
cargo fmt --all -- --check
cargo test --locked -p vectorcraft-ui-egui --features opendraw workspaces::tests --lib
```

Criteri di accettazione: selettore con quattro modalità; Classica come default; cambi di workspace non distruttivi; workspace personali serializzati e ricaricati; build standard priva di modifiche ai preset upstream.
