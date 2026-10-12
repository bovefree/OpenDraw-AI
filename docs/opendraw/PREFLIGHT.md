# OD-010 — Preflight preliminare per la stampa

**Problema concreto:** utenti e tipografie segnalano difficoltà con profili colore, abbondanze e PDF che si aprono ma non corrispondono alle aspettative.

Fonti:
- CorelDRAW 2026, opzioni PDF e colore: https://help.coreldraw.com/CorelDRAW/540111192/Documentation-Windows/CorelDRAW-en/CorelDRAW-PDF-color-management-options.html
- CorelDRAW 2026, preparazione dei file per la stampa: https://help.coreldraw.com/CorelDRAW/540111192/Documentation-Windows/CorelDRAW-en/CorelDRAW-Prepare-PDF-files-for-a-print-provider.html
- CorelDRAW 2025, correzione documentata su bleed spostato nell'esportazione: https://www.coreldraw.com/en/support/updates/cdgs2025/update.html
- CorelDRAW community, profili FOGRA mancanti nei preset dell'utente: https://community.coreldraw.com/talk/coreldraw-graphics-suite-2025/f/coreldraw-graphics-suite-2025-for-windows/68122/publish-to-pdf-file

## Comando inserito

Solo nell'app `--features opendraw`:

```json
{"id":"document.preflight","params":{"profile":"poster140x200"}}
```

Disponibile tramite motore di comandi, quindi anche agli agenti e alle automazioni che interrogano il motore. Due profili:
- `genericPrint`: almeno 3 mm di abbondanza.
- `poster140x200`: dimensione 1400 × 2000 mm e almeno 5 mm di abbondanza.
- In entrambi: verifica modo CMYK e risoluzione effetti raster almeno 300 ppi; `minBleedMm` può personalizzare la soglia da 0 a 50.

Restituisce `preliminaryPass`, elenco `findings` con codici e gravità, misure per pagina, `pdfCertified:false` e l'elenco dei controlli **non effettuati**.

## Limiti imprescindibili

Un report senza segnalazioni NON certifica la conformità PDF/X-4 né la stampabilità finale.

Da implementare separatamente con lettura del PDF esportato: presenza di OutputIntent ICC, versioni PDF/X, reale rapporto BleedBox/TrimBox, font incorporati, trasparenze/sovrastampe, colore separazioni e PPI effettivi delle foto inserite.

Il parametro `rasterEffectsPpi` riguarda **solo gli effetti raster**, non indica la risoluzione delle immagini importate. Il report non deve confondere queste misure.

## Collaudo

```sh
cargo fmt --all -- --check
cargo test --locked -p vectorcraft-engine --features opendraw --lib preflight::tests
cargo test --locked -p vectorcraft-engine --lib tests_newdoc
```

I file CDR/PDF privati non devono essere pubblicati nel repository. I test usano documenti sintetici.
