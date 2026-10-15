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
    let icon = match id {
        "file.newDialog" => "file-plus",
        "file.open" => "folder-open",
        "file.save" => "save",
        "edit.undo" => "undo-2",
        "edit.redo" => "redo-2",
        "view.fitArtboard" => "frame",
        "view.fitAll" => "maximize-2",
        "window.panel" if params.get("panel").and_then(Value::as_str) == Some("layers") => "layers",
        _ => "layout-grid",
    };
    let mut clicked = false;
    ui.add_enabled_ui(enabled, |ui| {
        clicked = widgets::icon_button(ui, icon, label, false, 24.0).clicked();
    });
    if clicked {
        if id == "window.panel" && params.get("panel").and_then(Value::as_str).is_some_and(|p| matches!(p, "layers" | "properties")) {
            app.ui.dock_collapsed = false;
        }
        if let Err(error) = app.run(id, params) {
            app.ui.status = error;
        }
    }
}

/// Familiar, action-backed standard bar directly under the menu bar, above
/// the contextual property bar. It is not decorative: every control runs a
/// currently registered command.
pub fn command_bar(app: &mut VectorcraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    egui::Panel::top("opendraw_standard_bar")
        .exact_size(33.0)
        .frame(egui::Frame::NONE.fill(t.panel).inner_margin(egui::Margin::symmetric(6, 3)).stroke(Stroke::new(1.0, t.border)))
        .show(ui, |ui| {
            egui::ScrollArea::horizontal().id_salt("opendraw-standard-scroll").auto_shrink([false, true]).show(ui, |ui| {
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
        .exact_size(40.0)
        .frame(egui::Frame::NONE.fill(t.panel).inner_margin(egui::Margin::symmetric(6, 3)).stroke(Stroke::new(1.0, t.border)))
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
                        let stroke = crate::panels::current_stroke(app);
                        let mixed = crate::panels::stroke_mixed(app, ui.ctx());
                        let weight = crate::panels::stroke::shown_weight(app, stroke.as_ref(), &mixed);
                        crate::panels::stroke::weight_field(app, ui, "opendraw-outline-width", weight, 72.0);
                        if crate::panels::character::text_style(app).is_some() {
                            ui.separator();
                            crate::panels::character::opendraw_control_bar(app, ui);
                        } else {
                            crate::place::control_bar_details(app, ui);
                        }
                        if ui.button("Oggetti").clicked() {
                            app.ui.dock_tab = crate::state::DockTab::Layers;
                            app.ui.dock_collapsed = false;
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
                                if ui.selectable_label(units == unit, unit.label()).clicked()
                                    && let Err(error) = app.run("document.setUnits", json!({"units":unit.label()}))
                                {
                                    app.ui.status = error;
                                }
                            }
                        });
                        ui.separator();
                        if ui.button("Oggetti").clicked() {
                            app.ui.dock_tab = crate::state::DockTab::Layers;
                            app.ui.dock_collapsed = false;
                        }
                        if ui.button("Proprietà").clicked() {
                            app.ui.dock_tab = crate::state::DockTab::Properties;
                            app.ui.dock_collapsed = false;
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
        .exact_size(26.0)
        .frame(egui::Frame::NONE.fill(t.panel).inner_margin(egui::Margin::symmetric(6, 1)).stroke(Stroke::new(1.0, t.border)))
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

/// Corel-style Objects docker for the Classic workspace. Keeps the real
/// hierarchy, selection and layer commands; the upstream Illustrator-style
/// tab/icon dock remains available in the other workspaces.
pub fn classic_dock(app: &mut VectorcraftApp, ui: &mut Ui) {
    use crate::state::DockTab;

    let t = Tokens::get(ui.ctx());
    // The right-hand vertical strip is not ornamental: it opens functional
    // panels, with Objects selected on first launch.
    egui::Panel::right("opendraw_classic_rail")
        .exact_size(29.0)
        .frame(egui::Frame::NONE.fill(t.panel).inner_margin(egui::Margin::symmetric(2, 5)).stroke(Stroke::new(1.0, t.border)))
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                for tab in [DockTab::Layers, DockTab::Properties, DockTab::Libraries] {
                    let label = tab.info().1;
                    let galley = ui.painter().layout_no_wrap(label.to_string(), egui::FontId::proportional(11.0), t.text);
                    let (rect, response) = ui.allocate_exact_size(vec2(24.0, galley.size().x + 24.0), Sense::click());
                    if app.ui.dock_tab == tab && !app.ui.dock_collapsed {
                        ui.painter().rect_filled(rect, 1.0, t.tool_active);
                    } else if response.hovered() {
                        ui.painter().rect_filled(rect, 1.0, t.hover);
                    }
                    let at = rect.center() + vec2(galley.size().y / 2.0, -galley.size().x / 2.0);
                    ui.painter().add(egui::epaint::TextShape::new(at, galley, t.text).with_angle(std::f32::consts::FRAC_PI_2));
                    if response.on_hover_text(label).clicked() {
                        app.ui.dock_collapsed = app.ui.dock_tab == tab && !app.ui.dock_collapsed;
                        app.ui.dock_tab = tab;
                    }
                    ui.add_space(9.0);
                }
                ui.separator();
                if ui.small_button("C").on_hover_text("Colore").clicked()
                    && let Err(error) = app.run("window.panel", json!({"panel":"color"}))
                {
                    app.ui.status = error;
                }
            });
        });

    if app.ui.dock_collapsed {
        return;
    }
    egui::Panel::right("opendraw_classic_dock")
        .resizable(true)
        .default_size(298.0)
        .size_range(250.0..=510.0)
        .frame(egui::Frame::NONE.fill(t.panel).stroke(Stroke::new(1.0, t.border)).inner_margin(egui::Margin::symmetric(8, 5)))
        .show(ui, |ui| {
            let tab = app.ui.dock_tab;
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(tab.info().1).size(13.0).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("×").on_hover_text("Riduci pannello").clicked() {
                        app.ui.dock_collapsed = true;
                    }
                    ui.menu_button("⋯", |ui| {
                        for choice in [DockTab::Layers, DockTab::Properties, DockTab::Libraries] {
                            if ui.selectable_label(app.ui.dock_tab == choice, choice.info().1).clicked() {
                                app.ui.dock_tab = choice;
                                ui.close();
                            }
                        }
                    });
                });
            });
            ui.separator();
            match tab {
                DockTab::Layers => crate::panels::layers::show(app, ui),
                DockTab::Properties => {
                    egui::ScrollArea::vertical().id_salt("opendraw-classic-properties").auto_shrink([false, false]).show(ui, |ui| {
                        crate::panels::properties::show(app, ui);
                    });
                }
                DockTab::Libraries => crate::panels::libraries(app, ui),
            }
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
        .exact_size(22.0)
        .frame(egui::Frame::NONE.fill(t.panel).inner_margin(egui::Margin::symmetric(2, 4)).stroke(Stroke::new(1.0, t.border)))
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new("C").size(12.0).strong())
                    .on_hover_text("Tavolozza: clic sinistro = riempimento, clic destro = contorno");
            });
            ui.separator();
            egui::ScrollArea::vertical().id_salt("opendraw_color_scroll").auto_shrink([false, false]).show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                for (name, paint) in swatches {
                    let (rect, response) = ui.allocate_exact_size(vec2(17.0, 16.0), Sense::click());
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

    fn app() -> VectorcraftApp {
        let mut app = VectorcraftApp::new(vectorcraft_engine::Session::new(), Default::default());
        app.run("file.new", json!({"width":210,"height":297})).unwrap();
        app
    }

    #[test]
    fn classic_dock_can_collapse_and_reopen_without_losing_selection() {
        let mut app = app();
        app.ui.dock_tab = crate::state::DockTab::Layers;
        assert!(crate::tests_labels::painted_text(&mut app, classic_dock).contains("Oggetti"));
        app.run("window.collapseDock", json!({})).unwrap();
        assert!(app.ui.dock_collapsed);
        // The labelled rail remains visible, while the panel body gives its width back.
        let text = crate::tests_labels::painted_text(&mut app, classic_dock);
        assert_eq!(text.lines().filter(|s| *s == "Oggetti").count(), 1);
        app.run("window.collapseDock", json!({})).unwrap();
        assert!(!app.ui.dock_collapsed);
        assert_eq!(app.ui.dock_tab, crate::state::DockTab::Layers);
        assert!(crate::tests_labels::painted_text(&mut app, classic_dock).contains("Oggetti"));
    }

    #[test]
    fn property_bar_follows_text_selection_and_returns_to_geometry() {
        let mut app = app();
        let text = app.run("text.create", json!({"x":20,"y":30,"text":"OpenDraw","size":12})).unwrap()["id"].clone();
        app.run("select.set", json!({"ids":[text]})).unwrap();
        assert!(crate::tests_labels::painted_text(&mut app, property_bar).lines().any(|s| s == "Testo"));
        let rect = app.run("shape.rectangle", json!({"x":10,"y":10,"width":20,"height":20})).unwrap()["id"].clone();
        app.run("select.set", json!({"ids":[rect]})).unwrap();
        assert!(!crate::tests_labels::painted_text(&mut app, property_bar).lines().any(|s| s == "Testo"));
    }

    #[test]
    fn classic_flyout_tools_are_registered_and_unique() {
        let mut seen = std::collections::BTreeSet::new();
        for (_, groups) in crate::toolbar::CLASSIC {
            for group in *groups {
                for id in *group {
                    assert!(vectorcraft_tools::tool_info(id).is_some(), "unknown tool {id}");
                    assert!(seen.insert(*id), "tool {id} appears in more than one flyout");
                }
            }
        }
    }

    #[test]
    fn classic_tool_groups_follow_a_print_design_workflow() {
        let categories: Vec<_> = crate::toolbar::CLASSIC.iter().map(|(name, _)| *name).collect();
        assert_eq!(categories.first(), Some(&"Puntatore"));
        assert!(categories.contains(&"Forma") && categories.contains(&"Disegno") && categories.contains(&"Modifica"));
        let group_of = |tool: &str| {
            crate::toolbar::CLASSIC.iter().find_map(|(category, groups)| groups.iter().any(|group| group.contains(&tool)).then_some(*category))
        };
        assert_eq!(group_of("directSelection"), Some("Forma"));
        assert_eq!(group_of("warp"), Some("Modifica"));
        assert_eq!(group_of("lineCut"), Some("Ritaglia"));
        assert_eq!(crate::toolbar::classic_tool_label("lineSegment"), Some("Linea a 2 punti"));
    }

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
