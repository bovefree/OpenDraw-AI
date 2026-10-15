//! Drop Shadow tool: drag the shadow's direction and distance on selected art, or click to apply
//! the standard 7 pt offset. Blur, opacity, blend mode and colour remain editable as a live effect.

use serde_json::{Value, json};
use vectorcraft_doc::NodeId;
use vectorcraft_doc::hit::hit_test;
use vectorcraft_geom::{Point, Vec2};

use super::paint_owner;
use crate::{Action, Cursor, Mods, Overlay, PointerEvent, PointerKind, Tool, ToolContext, json_ids};

const EFFECT: &str = "stylize.dropShadow";
const DEFAULT_OFFSET: Vec2 = Vec2::new(7.0, 7.0);

#[derive(Clone, Debug)]
struct Gesture {
    start: Point,
    end: Point,
    ids: Vec<NodeId>,
    began: bool,
}

#[derive(Default)]
pub struct DropShadowTool {
    gesture: Option<Gesture>,
}

impl DropShadowTool {
    fn params(delta: Vec2) -> Value {
        json!({
            "effect": EFFECT,
            "params": {
                "mode": "multiply",
                "opacity": 75.0,
                "x": delta.x,
                "y": delta.y,
                "blur": 5.0,
                "color": "#000000"
            }
        })
    }

    fn apply(ids: &[NodeId], delta: Vec2) -> Value {
        let mut p = Self::params(delta);
        p["ids"] = json_ids(ids);
        p
    }

    fn delta(start: Point, end: Point, shift: bool) -> Vec2 {
        let d = end - start;
        if shift { vectorcraft_geom::constrain_angle(d, 45.0) } else { d }
    }
}

impl Tool for DropShadowTool {
    fn id(&self) -> &'static str {
        "dropShadow"
    }

    fn busy(&self) -> bool {
        self.gesture.as_ref().is_some_and(|g| g.began)
    }

    fn pointer(&mut self, cx: &ToolContext, ev: &PointerEvent) -> Vec<Action> {
        match ev.kind {
            PointerKind::Down => {
                let ids = if !cx.selection.objects.is_empty() {
                    cx.selection.objects.clone()
                } else {
                    let Some(hit) = hit_test(cx.doc, ev.pos, cx.hit_options()) else { return vec![] };
                    let id = paint_owner(cx.doc, hit.leaf);
                    vec![id]
                };
                let mut out = Vec::new();
                if cx.selection.objects.is_empty() {
                    out.push(Action::Exec("select.set".into(), json!({"ids": json_ids(&ids)})));
                }
                self.gesture = Some(Gesture { start: ev.pos, end: ev.pos, ids, began: false });
                out
            }
            PointerKind::Drag => {
                let Some(g) = &mut self.gesture else { return vec![] };
                g.end = ev.pos;
                if !g.began {
                    if ev.pos.distance(g.start) < cx.tol(2.0) {
                        return vec![];
                    }
                    g.began = true;
                    return vec![
                        Action::Begin("Drop Shadow".into()),
                        Action::Preview("effect.apply".into(), Self::apply(&g.ids, Self::delta(g.start, g.end, ev.mods.shift))),
                    ];
                }
                vec![Action::Preview("effect.apply".into(), Self::apply(&g.ids, Self::delta(g.start, g.end, ev.mods.shift)))]
            }
            PointerKind::Up => {
                let Some(mut g) = self.gesture.take() else { return vec![] };
                g.end = ev.pos;
                if g.began {
                    vec![Action::Preview("effect.apply".into(), Self::apply(&g.ids, Self::delta(g.start, g.end, ev.mods.shift))), Action::Commit]
                } else {
                    vec![Action::Exec("effect.apply".into(), Self::apply(&g.ids, DEFAULT_OFFSET))]
                }
            }
            PointerKind::DoubleClick | PointerKind::Move => vec![],
        }
    }

    fn overlays(&self, cx: &ToolContext) -> Vec<Overlay> {
        let Some(g) = self.gesture.as_ref().filter(|g| g.began) else { return vec![] };
        let delta = g.end - g.start;
        let tip = g.start + delta;
        vec![
            Overlay::Line { a: g.start, b: tip, color: [0x9b, 0x59, 0xb6], dashed: false },
            Overlay::Measure { p: tip + Vec2::new(cx.tol(10.0), cx.tol(10.0)), text: format!("Drop Shadow\n{}", cx.offset_label(delta.x, delta.y)) },
        ]
    }

    fn cursor(&self, _cx: &ToolContext, _p: Point, _mods: Mods) -> Cursor {
        Cursor::Crosshair
    }

    fn options(&self) -> Value {
        self.gesture.as_ref().map_or(Value::Null, |g| {
            let d = g.end - g.start;
            json!({"x": d.x, "y": d.y, "blur": 5.0, "opacity": 75.0})
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::*;
    use vectorcraft_doc::Selection;

    #[test]
    fn drag_applies_an_editable_shadow_with_the_drag_offset() {
        let (d, id) = doc_with_rect();
        let mut selection = Selection::default();
        selection.add(id);
        let paint = paint();
        let cx = cx(&d, &selection, &paint);
        let mut tool = DropShadowTool::default();
        assert!(tool.pointer(&cx, &PointerEvent::new(PointerKind::Down, 120.0, 120.0)).is_empty());
        let drag = tool.pointer(&cx, &PointerEvent::new(PointerKind::Drag, 145.0, 132.0));
        assert_eq!(drag[0], Action::Begin("Drop Shadow".into()));
        assert_eq!(drag[1], Action::Preview("effect.apply".into(), DropShadowTool::apply(&[id], Vec2::new(25.0, 12.0))));
        let up = tool.pointer(&cx, &PointerEvent::new(PointerKind::Up, 145.0, 132.0));
        assert!(matches!(up.last(), Some(Action::Commit)));
        assert_eq!(up[0], Action::Preview("effect.apply".into(), DropShadowTool::apply(&[id], Vec2::new(25.0, 12.0))));
    }

    #[test]
    fn click_applies_the_default_and_empty_selection_can_pick_art() {
        let (d, id) = doc_with_rect();
        let paint = paint();
        let selection = Selection::default();
        let cx = cx(&d, &selection, &paint);
        let mut tool = DropShadowTool::default();
        let down = tool.pointer(&cx, &PointerEvent::new(PointerKind::Down, 150.0, 150.0));
        assert_eq!(down, vec![Action::Exec("select.set".into(), json!({"ids": [id.0]}))]);
        let up = tool.pointer(&cx, &PointerEvent::new(PointerKind::Up, 150.0, 150.0));
        assert_eq!(up, vec![Action::Exec("effect.apply".into(), DropShadowTool::apply(&[id], DEFAULT_OFFSET))]);
    }
}
