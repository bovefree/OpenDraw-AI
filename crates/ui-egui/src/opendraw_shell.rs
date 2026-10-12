//! OpenDraw AI Classic desktop shell. Never compiled into ordinary VectorCraft.
//!
//! This is an original interface using the project's existing document engine:
//! operational familiarity rather than imitation of copyrighted icons or artwork.

use egui::{Sense, Stroke, Ui, vec2};
use serde_json::{Value, json};
use vectorcraft_color::Paint;

use crate::theme::Tokens;
use crate::{VectorcraftApp, menus, widgets};

type MenuRoot = (&'static str, Vec<menus::Item>);

fn cmd(label: &'static str, id: &'static str) -> menus::Item {
    menus::Item::Cmd(label, id, Value::Null)
}

fn panel(label: &'static str, id: &'static str) -> menus::Item {
    menus::Item::Cmd(label, "window.panel", json!({"panel":id}))
}

/// Reorganize existing **working** commands using a familiar print-design
/// workflow. All application actions still use their existing engine IDs.
/// Unimplemented tables remain explicitly disabled rather than pretending to
/// offer an editable table tool.
pub fn classic_menu_tree(mut roots: Vec<MenuRoot>) -> Vec<MenuRoot> {
    let mut take = |name: &str| {
        roots.iter().position(|r| r.0 == name).map(|index| roots.remove(index).1).unwrap_or_default()
    };
    let app = take("VectorCraft");
    let file = take("File");
    let mut edit = take("Edit");
    let mut object = take("Object");
    let select = take("Select");
    let type_menu = take("Type");
    let view = take("View");
    let window = take("Window");
    // Select is part of the Edit workflow, not a separate Illustrator menu.
    edit.push(menus::Item::Sep);
    edit.push(menus::Item::Sub("Selezione", select));
    object.push(menus::Item::Sep);
    object.push(menus::Item::Sub(
        "PowerClip",
        vec![
            cmd("Crea contenitore", "object.clippingMask.make"),
            cmd("Modifica contenuto", "object.clippingMask.editContents"),
            cmd("Modifica cornice", "object.clippingMask.editMask"),
            cmd("Estrai contenuto", "object.clippingMask.release"),
        ],
    ));
    vec![
        ("OpenDraw AI", app),
        ("File", file),
        ("Modifica", edit),
        ("Visualizza", view),
        (
            "Layout",
            vec![
                cmd("Impostazione documento…", "file.documentSetup"),
                panel("Pagine e tavole", "artboards"),
                cmd("Mostra righelli", "view.rulers"),
                cmd("Adatta tutte le pagine", "view.fitAll"),
            ],
        ),
        ("Oggetto", object),
        (
            "Effetti",
            vec![
                cmd("Espandi aspetto", "effect.expandAppearance"),
                cmd("Crea mezzatinta vettoriale", "object.vectorHalftone"),
                cmd("Appiattisci trasparenza…", "ui.flattenTransparencyDialog"),
                panel("Aspetto", "appearance"),
            ],
        ),
        (
            "Bitmap",
            vec![
                cmd("Rasterizza oggetti…", "object.rasterize"),
                cmd("Ritaglia immagine…", "ui.cropImage"),
                panel("Vettorizza immagine", "imageTrace"),
                panel("Collegamenti immagini", "links"),
            ],
        ),
        ("Testo", type_menu),
        ("Tabella", vec![menus::Item::Todo("Strumenti tabella non ancora disponibili", "")]),
        (
            "Strumenti",
            vec![
                cmd("Scorciatoie tastiera…", "edit.keyboardShortcuts"),
                cmd("Preferenze…", "edit.preferences"),
                cmd("Impostazioni colore…", "edit.colorSettings"),
                panel("Allineamento", "align"),
            ],
        ),
        ("Finestra", window),
        (
            "Aiuto",
            vec![
                cmd("Cerca comando…", "help.commandPalette"),
                cmd("Informazioni su OpenDraw AI", "help.about"),
            ],
        ),
    ]
}

fn run_button(app: &mut VectorcraftApp, ui: &mut Ui, label: &str, id: &str, params: Value) {
    let enabled = menus::enabled(app, id);
    if ui.add_enabled(enabled, egui::Button::new(label).min_size(vec2(61.0, 25.0))).clicked()
        && let Err(error) = app.run(id, params)
    {
        app.ui.status = error;
    }
}

/// Familiar, action-backed standard bar directly under the menu bar, above
/// the contextual property bar. It is not decorative: every control runs a
/// currently registered command.
pub fn command_bar(app: &mut VectorcraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    egui::Panel::top("opendraw_standard_bar")
        .exact_size(40.0)
        .frame(egui::Frame::NONE.fill(t.panel).inner_margin(egui::Margin::symmetric(8, 4)).stroke(Stroke::new(1.0, t.border)))
        .show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                run_button(app, ui, "Nuovo", "file.newDialog", json!({}));
                run_button(app, ui, "Apri", "file.open", json!({}));
                run_button(app, ui, "Salva", "file.save", json!({}));
                ui.separator();
                run_button(app, ui, "Annulla", "edit.undo", json!({}));
                run_button(app, ui, "Ripeti", "edit.redo", json!({}));
                ui.separator();
                run_button(app, ui, "Pagina", "view.fitArtboard", json!({}));
                run_button(app, ui, "Tutto", "view.fitAll", json!({}));
                ui.separator();
                run_button(app, ui, "Oggetti", "window.panel", json!({"panel":"layers"}));
                run_button(app, ui, "Proprietà", "window.panel", json!({"panel":"properties"}));
                ui.separator();
                ui.label(egui::RichText::new("OpenDraw AI  ·  Classica").color(t.text_dim).size(11.0));
            });
        });
}

/// Colour strip on the outer edge of the window, beside the objects dock.
/// Left click sets fill; right click sets outline. Swatches are *document
/// swatches*, so named/spot/process colours retain their meaning.
pub fn color_strip(app: &mut VectorcraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let swatches: Vec<(String, Paint)> = app
        .session
        .active()
        .map(|st| st.doc.swatches.iter().filter(|s| matches!(s.paint, Paint::None | Paint::Solid { .. })).map(|s| (s.name.clone(), s.paint.clone())).collect())
        .unwrap_or_default();
    egui::Panel::right("opendraw_color_strip")
        .exact_size(39.0)
        .frame(egui::Frame::NONE.fill(t.panel).inner_margin(egui::Margin::symmetric(4, 5)).stroke(Stroke::new(1.0, t.border)))
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new("C").size(12.0).strong()).on_hover_text("Tavolozza: clic sinistro = riempimento, clic destro = contorno");
            });
            ui.separator();
            egui::ScrollArea::vertical().id_salt("opendraw_color_scroll").auto_shrink([false, false]).show(ui, |ui| {
                for (name, paint) in swatches {
                    let (rect, response) = ui.allocate_exact_size(vec2(27.0, 24.0), Sense::click());
                    widgets::swatch_tile(ui, rect.shrink(1.0), &paint, false, response.hovered());
                    let response = response.on_hover_text(format!("{name}\nSinistro: riempimento · Destro: contorno"));
                    let command = if response.secondary_clicked() {
                        Some("paint.setStroke")
                    } else if response.clicked() {
                        Some("paint.setFill")
                    } else {
                        None
                    };
                    if let Some(command) = command
                        && let Err(error) = app.run(command, json!({"swatch":name}))
                    {
                        app.ui.status = error;
                    }
                }
            });
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classic_menus_offer_working_commands_and_expose_missing_tables() {
        let tree = menus::menu_tree_named(false);
        let names: Vec<_> = tree.iter().map(|(name, _)| *name).collect();
        // Running the transformation twice is not supported; upstream menu
        // tree is transformed exactly once by menu_tree_named when enabled.
        assert!(names.contains(&"Oggetto"));
        assert!(names.contains(&"Layout"));
        assert!(names.contains(&"Bitmap"));
        assert!(names.contains(&"Tabella"));
    }

    #[test]
    fn classic_shell_has_real_document_commands() {
        let mut s = vectorcraft_engine::Session::new();
        s.execute("file.new", &json!({"width":210,"height":297})).unwrap();
        let id = s.execute("shape.rectangle", &json!({"x":10,"y":10,"width":20,"height":20})).unwrap()["id"].clone();
        s.execute("select.set", &json!({"ids":[id]})).unwrap();
        for cmd in ["paint.setFill", "paint.setStroke"] {
            let result = s.execute(cmd, &json!({"swatch":"Red"}));
            assert!(result.is_ok(), "{cmd}: {result:?}");
        }
    }
}
