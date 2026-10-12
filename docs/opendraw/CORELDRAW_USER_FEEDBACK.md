# OD-009 — Scelta operativa basata sul feedback CorelDRAW

## Criterio

L'area di lavoro OpenDraw AI deve mantenere il metodo operativo familiare agli utenti CorelDRAW 2026. Le funzioni vengono riprodotte senza copiare marchi o asset grafici. Il feedback pubblico identifica le priorità, ma una testimonianza singola non dimostra un difetto universale.

## Fonti consultate, ottobre 2026

- Reddit: lamentele CorelDRAW 2026 su crash e affidabilità, incluse testimonianze contrastanti: https://www.reddit.com/r/coreldraw/comments/1sdhqei/coreldraw_keeps_crashing/
- Reddit: prestazioni e rallentamento con file di centinaia di oggetti, PowerClip e visualizzazione: https://www.reddit.com/r/coreldraw/comments/1nd93rw
- Reddit: utenti descrivono i vantaggi di selezione con riquadro completamente racchiuso, modifica nodi, contour, allineamento, snapping, PowerClip, personalizzazione: https://www.reddit.com/r/coreldraw/comments/h9q0xf
- Reddit: il vantaggio della selezione e della modifica dei nodi è confermato in un'altra discussione: https://www.reddit.com/r/coreldraw/comments/ga6fld
- Help CorelDRAW 2026: selezione con Pick, selezione per riquadro, modi con Alt, oggetti sovrapposti e gruppi: https://help.coreldraw.com/CorelDRAW/540111192/Documentation-Windows/CorelDRAW-en/CorelDRAW-Select-objects.html
- Corel tip: **Alt** con il Pick include anche oggetti solo in parte compresi nella selezione: https://www.coreldraw.com/en/pages/items/1500718.html
- Reddit: miglioramento desiderato per PowerClip — scegliere da pannello livelli la destinazione senza nascondere oggetti: https://www.reddit.com/r/coreldraw/comments/1omceii
- Reddit: file persi o corrotti, backup e necessità di salvataggio frequente: https://www.reddit.com/r/coreldraw/comments/1ok3wpg
- G2: recensioni CorelDRAW 2026, apprezzamento per lavoro veloce, strumenti e prestampa, con riserve su prestazioni e accessibilità: https://www.g2.com/products/coreldraw/reviews

## Ordine delle verifiche

| Priorità | Problema/valore | Implementazione/prova |
|---|---|---|
| P0 | Perdita del lavoro e crash | Test di riapertura, recovery, rollback, backup, file complessi e error injection; misurare senza promettere assenza di crash |
| P0 | Pick: selezionare solo oggetti racchiusi | **OD-009 implementa il comportamento OpenDraw**; Alt mantiene selezione per intersezione, Shift continua a gestire il toggle |
| P1 | Tempi di risposta e PowerClip | Benchmark su selezione/zoom/movimento/PowerClip di 100, 500 e 10.000 oggetti; verificare rendering e undo |
| P1 | PowerClip / clipping mask | Scelta della cornice dal pannello Oggetti, modifica dei contenuti e comportamento non distruttivo |
| P1 | Stampa | PDF/X-4, output intent ICC, bleed box diverso dal trim, font incorporati e controlli sui raster |
| P1 | Palette colore | Clic primario per riempimento e secondario per contorno, entrambi sempre raggiungibili |
| P2 | Nodi, snapping, allineamento, scorciatoie | Confronto video/clic/tempi con CorelDRAW 2026 su cinque documenti tipo |
| P2 | Automazione | Comandi AI reversibili e ripetibili senza perdita di editabilità |

## OD-009 — Dettaglio tecnico

**Con feature `opendraw`:** trascinare il riquadro con lo strumento Selezione seleziona soltanto gli oggetti con limiti geometrici completamente racchiusi. Tenere **Alt** durante il trascinamento seleziona anche quelli parzialmente intersecati. **Shift** mantiene il toggle.

**Build standard VectorCraft:** conserva l'algoritmo di selezione preesistente. L'opzione è propagata dalla feature dell'app a `vectorcraft-ui-egui` e `vectorcraft-tools`. I filtri per gruppi, isolamento, oggetti nascosti/bloccati sono riutilizzati dal motore corrente.

**Limite dichiarato:** la verifica di inclusione utilizza l'involucro geometrico, che è conservativo per curve inclinate e forme molto complesse; non equivale a un test di intersezione esatto di tutte le aree dipinte. Dovrà essere valutato in seguito con curve e clipping complessi.

## Privacy / corpus

Documenti personali (CDR, PDF e asset del cliente) non vengono inseriti nel repository pubblico né nei test CI. Per gli stress test si useranno documenti sintetici.
