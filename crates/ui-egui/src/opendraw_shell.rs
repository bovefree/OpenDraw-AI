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
    let mut take = |name: &str| roots.iter().position(|r| r.0 == name).map(|index| roots.remove(index).1).unwrap_or_default();
    let _upstream_app_menu = take("VectorCraft");
    let file = take("File");
    let mut edit = take("Edit");
    let mut object = take("Object");
    let effect = take("Effect");
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
        ("OpenDraw AI", vec![cmd("Informazioni su OpenDraw AI", "help.about")]),
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
        ("Effetti", effect),
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
        ("Aiuto", vec![cmd("Cerca comando…", "help.commandPalette"), cmd("Informazioni su OpenDraw AI", "help.about")]),
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

/// Native OpenDraw property bar, modelled on the space and workflow in the
/// CorelDRAW 2026 reference. Fields write to editable vectors through the
/// existing document engine; nothing is a fake display-only input.
pub fn property_bar(app: &mut VectorcraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let units = app.session.general_unit();
    let selected = app.session.active().map_or(0, |st| st.selection.objects.len());
    let bounds = app.selection_box().map(|b| (b.center(), b.rect.width(), b.rect.height()));
    let page = app.session.active().and_then(|st| {
        let index = app.view().map_or(0, |v| v.artboard);
        st.doc.artboards.get(index).map(|a| (a.rect.width(), a.rect.height()))
    });

    egui::Panel::top("opendraw_classic_properties")
        .exact_size(43.0)
        .frame(egui::Frame::NONE.fill(t.panel).inner_margin(egui::Margin::symmetric(8, 5)).stroke(Stroke::new(1.0, t.border)))
        .show(ui, |ui| {
            egui::ScrollArea::horizontal().auto_shrink([false, true]).show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    if selected > 0 {
                        ui.label(egui::RichText::new(if selected == 1 { "Oggetto" } else { "Oggetti" }).size(12.0).strong());
                        if let Some((center, width, height)) = bounds {
                            for (key, label, value) in [("x", "X", center.x), ("y", "Y", center.y), ("width", "L", width), ("height", "A", height)] {
                                ui.label(egui::RichText::new(label).size(11.0).color(t.text_dim));
                                if let Some(next) = widgets::num_field(ui, ("opendraw-geometry", key), Some(value), units, 82.0) {
                                    let proportional = app.session.prefs.constrain_proportions;
                                    if let Err(error) = app.run("object.setBounds", json!({key: next, "reference": 4, "proportional": proportional}))
                                    {
                                        app.ui.status = error;
                                    }
                                }
                            }
                            crate::panels::transform::constrain_link(app, ui);
                        }
                        ui.separator();
                        if ui.button("Ruota…").clicked() {
                            crate::menus::invoke(app, "object.rotate", json!({}));
                        }
                        ui.separator();
                        ui.label("Riempimento");
                        crate::panels::paint_chip(app, ui, false, 21.0, true);
                        ui.label("Contorno");
                        crate::panels::paint_chip(app, ui, true, 21.0, true);
                        if ui.button("Oggetti").clicked() {
                            app.ui.dock_tab = crate::state::DockTab::Layers;
                        }
                    } else {
                        ui.label(egui::RichText::new("Pagina").size(12.0).strong());
                        if let Some((width, height)) = page {
                            ui.label("L");
                            ui.add_enabled(false, egui::Label::new(format!("{:.1} mm", width / vectorcraft_doc::Unit::Millimeters.points())));
                            ui.label("A");
                            ui.add_enabled(false, egui::Label::new(format!("{:.1} mm", height / vectorcraft_doc::Unit::Millimeters.points())));
                        }
                        if ui.button("Imposta pagina…").clicked() {
                            crate::menus::invoke(app, "file.documentSetup", json!({}));
                        }
                        ui.separator();
                        ui.label("Unità");
                        egui::ComboBox::from_id_salt("opendraw-units").width(110.0).selected_text(units.suffix()).show_ui(ui, |ui| {
                            for unit in [
                                vectorcraft_doc::Unit::Millimeters,
                                vectorcraft_doc::Unit::Centimeters,
                                vectorcraft_doc::Unit::Points,
                                vectorcraft_doc::Unit::Inches,
                                vectorcraft_doc::Unit::Pixels,
                            ] {
                                if ui.selectable_label(units == unit, unit.label()).clicked() {
                                    if let Err(error) = app.run("document.setUnits", json!({"units":unit.label()})) {
                                        app.ui.status = error;
                                    }
                                }
                            }
                        });
                        ui.separator();
                        if ui.button("Oggetti").clicked() {
                            app.ui.dock_tab = crate::state::DockTab::Layers;
                        }
                        if ui.button("Proprietà").clicked() {
                            app.ui.dock_tab = crate::state::DockTab::Properties;
                        }
                    }
                });
            });
        });
}

/// Multi-page navigator directly above the status line, matching the
/// placement in the reference screenshots. Each tab addresses a real artboard.
pub fn page_bar(app: &mut VectorcraftApp, ui: &mut Ui) {
    let pages = app.session.active().map_or(0, |st| st.doc.artboards.len());
    if pages == 0 {
        return;
    }
    let current = app.view().map_or(0, |v| v.artboard).min(pages - 1);
    let t = Tokens::get(ui.ctx());
    egui::Panel::bottom("opendraw_page_navigator")
        .exact_size(31.0)
        .frame(egui::Frame::NONE.fill(t.panel).inner_margin(egui::Margin::symmetric(6, 3)).stroke(Stroke::new(1.0, t.border)))
        .show(ui, |ui| {
            egui::ScrollArea::horizontal().auto_shrink([false, true]).show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    if ui.small_button("+").on_hover_text("Aggiungi pagina").clicked() {
                        match app.run("artboard.new", json!({})) {
                            Ok(_) => {
                                let _ = app.run("view.goToArtboard", json!({"index":pages}));
                            }
                            Err(error) => app.ui.status = error,
                        }
                    }
                    ui.separator();
                    if ui.add_enabled(current > 0, egui::Button::new("◀")).clicked() {
                        let _ = app.run("view.goToArtboard", json!({"index":"previous"}));
                    }
                    ui.label(format!("{} di {}", current + 1, pages));
                    if ui.add_enabled(current + 1 < pages, egui::Button::new("▶")).clicked() {
                        let _ = app.run("view.goToArtboard", json!({"index":"next"}));
                    }
                    ui.separator();
                    for index in 0..pages {
                        if ui.selectable_label(index == current, format!("Pagina {}", index + 1)).clicked() {
                            let _ = app.run("view.goToArtboard", json!({"index":index}));
                        }
                    }
                });
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
        .map(|st| {
            st.doc
                .swatches
                .iter()
                .filter(|s| matches!(s.paint, Paint::None | Paint::Solid { .. }))
                .map(|s| (s.name.clone(), s.paint.clone()))
                .collect()
        })
        .unwrap_or_default();
    egui::Panel::right("opendraw_color_strip")
        .exact_size(39.0)
        .frame(egui::Frame::NONE.fill(t.panel).inner_margin(egui::Margin::symmetric(4, 5)).stroke(Stroke::new(1.0, t.border)))
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new("C").size(12.0).strong())
                    .on_hover_text("Tavolozza: clic sinistro = riempimento, clic destro = contorno");
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
                    let params = if paint.is_none() { json!({"none":true}) } else { json!({"swatch":name}) };
                    if let Some(command) = command
                        && let Err(error) = app.run(command, params)
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
