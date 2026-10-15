# OpenDraw AI — ricerca CDR e mappa funzionale CorelDRAW

Data di verifica: 2026-10-13. La ricerca ha letto in modo statico documentazione, configurazioni e metadati dell’installazione CorelDRAW Graphics Suite 2026 (moduli versione 27.2.0.135), oltre ai riferimenti pubblici dei progetti di importazione. Non usa automazione o prove operative di CorelDRAW e non copia codice, asset, palette o testi proprietari.

## Cosa mostra l'installazione

- `Config/filters.ini` dichiara il formato `.cdr` e associa il filtro `IECDR.flt` al modulo `CdrCpr.dll`; entrambi risultano versione 27.2.0.135. Le stesse componenti servono ai template `.cdrt/.cdt`. Questo documenta il confine del lettore CDR commerciale, ma non spiega il formato interno né autorizza a riutilizzarne il codice.
- `Config/component_list.json`, i cataloghi di risorse e l'albero `Programs64` mostrano aree separate per selezione, modifica nodi, forme, testo, riempimenti, contorni, trasparenza, mesh, tabelle, simboli, stili, effetti, bitmap e gestione oggetti.
- La guida italiana locale espone flussi basati su strumenti e attributi: manipolazione di nodi e maniglie Bézier, testo artistico/di paragrafo e su tracciato, riempimenti interattivi con stop e angolo, contorni, ritaglio bitmap e pagine. È una mappa della superficie funzionale, non una specifica completa di serializzazione.
- I nomi, gli identificativi e i contenuti dell’installazione non vanno copiati nell’interfaccia OpenDraw; servono soltanto per costruire una lista indipendente di capacità da progettare.

## Architettura di apertura già presente in OpenDraw

Il percorso unico è `fileio::detect → load_with → open_loaded`: rileva il tipo, decodifica il file in un `Document`, restituisce avvisi, poi conserva nel documento origine e perdite d’importazione. Se l’importazione è incompleta, l’app impedisce di sovrascrivere il file sorgente senza conferma esplicita. `document.open` viene decodificato in un thread dedicato nell’app desktop.

I formati vettoriali vengono convertiti in oggetti nativi attraverso importatori dedicati: SVG usa `vectorcraft-svg`, PDF/`.ai` usa `vectorcraft-pdf` più il supporto PostScript per i dati editabili di Illustrator, DXF usa `vectorcraft-cad`. La tabella `FORMATS`, `OPEN_EXTS`, `PLACE_EXTS`, il rilevamento, il motore, la UI di apertura e i test fuzz sono i punti da estendere per un formato aggiuntivo. CDR non è attualmente rilevato né elencato.

## Opzione tecnica valutata

Il progetto pubblico [libcdr](https://github.com/LibreOffice/libcdr) offre un parser CDR/C​​MX indipendente in licenza MPL-2.0 e un’interfaccia `CDRDocument::parse` che emette eventi di disegno tramite librevenge. Le note di rilascio documentano miglioramenti per CDR 7, riempimenti, contorni, immagini incorporate, trasparenze e sicurezza della memoria; includono anche una correzione di crash su CDR 14 con gradienti. La versione corrente dichiarata dal sorgente consultato è 0.1.10, ma le note di rilascio visibili arrivano a 0.1.9.

Questa evidenza non certifica il supporto completo di file creati da CorelDRAW 2026. Una versione alternativa di parser Rust consultata pubblicamente dichiara copertura CDR 7–X8, quindi non è una base sufficiente per promettere il formato 2026. Nessuna delle fonti consultate documenta compatibilità completa con PowerClip, effetti live, profili colore, pagine multiple e tutte le varianti di testo CDR 2026.

La build Rust del progetto oggi non ha una catena C++/libcdr. Integrare una libreria nativa aggiungerebbe dipendenze C++ e di sistema (libcdr dichiara Boost, ICU, lcms2, librevenge e zlib), packaging dei binari per Windows x64 e macOS ARM64, aggiornamento delle licenze/NOTICE e manutenzione del bridge. Non va aggiunta una dipendenza o un converter esterno alla cieca né trattato un SVG appiattito come preservazione del documento.

## Percorso di implementazione verificabile

1. **Prova di fattibilità isolata:** creare un adapter opzionale che riceve byte CDR e produce eventi/una rappresentazione neutrale, senza dipendere da CorelDRAW. Misurare sulle versioni CDR disponibili e autorizzate: document size, pagine, livelli e ordine, curve/trasformazioni, testo editabile, immagini, riempimenti, contorni, trasparenze, profili e oggetti raggruppati. Tenere i campioni fuori dal repository e registrare hash e matrice dei risultati senza caricare documenti privati in CI.
2. **Scelta del backend:** confrontare parser Rust e `libcdr` per copertura reale, manutenzione, build riproducibile Windows/Mac, dipendenze, sicurezza e licenze. Scegliere solo dopo aver provato file posseduti dall’utente e campioni sintetici costruiti da specifiche pubbliche.
3. **Import nativo, non semplice anteprima:** mappare pagine/livelli/oggetti su `vectorcraft-doc`; mantenere testi e bitmap editabili quando possibile. Convertire effetti non rappresentabili in oggetti equivalenti soltanto se il risultato è misurabile; altrimenti rasterizzare solo l’oggetto interessato e descrivere chiaramente la perdita.
4. **Avvisi e tutela:** usare il tracciamento `warnings/import_losses`; non conservare un falso collegamento di salvataggio al CDR se non esiste writer. Un documento importato va salvato come `.vectorcraft`, SVG o PDF. Mantenere il CDR sorgente intatto; evidenziare font mancanti, effetti/riempimenti appiattiti o elementi ignorati.
5. **Sicurezza del parser:** introdurre limiti su decompressione, conteggi, annidamento e immagini, fuzzare gli stream corrotti e assicurare che nessun file ostile arresti l’app. Aggiungere test sintetici per detection, pagine, livelli, testo, bitmap e avvisi di perdita.
6. **Release:** dopo la scelta del backend, compilare e testare Windows x64 per primo; macOS ARM64 deve partire soltanto dopo il successo Windows, aggiornando pacchetti, attributi file e NOTICE.

## Prova locale del campione e stato dell’importatore

Verifica locale del 2026-10-15: il documento fornito dall’utente è un archivio ZIP CDR moderno da circa 67,7 MB. Contiene `content/root.dat` con un flusso RIFF `CDRU`, `content/data/page1.dat` … `page38.dat`, 38 miniature di pagina e 96 voci complessive. Il file resta fuori dal repository e non viene usato in CI.

OpenDraw ora riconosce `.cdr` dal MIME di archivio e dalla firma del flusso root, elenca l’estensione in Open e Place e apre le miniature di tutte le pagine come tavole raster separate. Il documento è marcato come anteprima, non può essere piazzato come vettoriale, non mantiene un percorso di salvataggio CDR e avvisa che vettori, testo, livelli, riempimenti ed effetti non sono stati importati. L’origine e la perdita vengono conservate per impedire la sovrascrittura del sorgente. Il lettore ZIP limita conteggi, dimensioni decompresse, miniature e pixel totali e controlla CRC, percorsi e intestazioni.

Questa è una prima apertura visiva, non l’importazione CDR richiesta in forma modificabile. Il lettore ora analizza l’ordine di `content/dataFileList.dat`, richiede che ogni flusso elencato esista e valida indice, offset e lunghezza di ogni riferimento esterno senza decomprimere in massa i payload. Sul campione locale ha verificato 22.807 riferimenti per 145.925.729 byte, mantenendo il file fuori dal repository e dalla CI. Non interpreta ancora i payload geometrici: il passo successivo è decodificare i record oggetto e mappare almeno curve, forme, trasformazioni, livelli e stili su `vectorcraft-doc`. I test sintetici coprono detection, anteprime multipagina, protezione del sorgente, tabella dei flussi, limiti dei riferimenti, pacchetti corrotti e mutazioni casuali. Prima di dichiarare compatibilità CDR 2026 servono test locali su più campioni posseduti dall’utente e misure di fedeltà per pagine, livelli, curve, testo, immagini e riempimenti.

## Decisione sul backend

Non è giustificato aggiungere ora una dipendenza C++ non verificata: il campione recente supera la copertura dichiarata di alcuni parser Rust consultati, mentre l’integrazione `libcdr` richiederebbe un bridge e pacchetti di dipendenze per entrambi i sistemi operativi senza garanzia di compatibilità con il formato 2026. Proseguire con l’analisi clean-room dei flussi e usare questo campione localmente come oracle visivo consentito, mai come fixture o dato di CI.
