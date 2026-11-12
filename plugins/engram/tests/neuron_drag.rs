//! The NEURONS window's nodes are grabbed where they are drawn: a press
//! on a neuron's drawn position grabs that neuron (and only it), and a
//! drag carries its x/y with the pointer, on the real face's layout.

use std::collections::HashMap;

use stave_core::face::{Host, Mods, Runtime};
use stave_core::params::AnyParam;
use stave_core::ui::layout::Rect;
use stave_core::ui::scene::{pt, Pt};
use stave_core::Plugin;
use stave_engram_vst3::Engram;

/// The face's geometry, as the window draws it (vfd/neurons.rs).
const LIP: f64 = 5.0;
const SIDE: f64 = 10.0;
const FIELD_TOP: f64 = 32.0;
const NEURONS_W: f64 = 340.0;
const SCREEN_H: f64 = 224.0;

struct Table {
    v: HashMap<u32, f64>,
}

impl Host for Table {
    fn normalized(&self, id: u32) -> f64 {
        *self.v.get(&id).unwrap_or(&0.0)
    }
    fn default_normalized(&self, _id: u32) -> f64 {
        0.0
    }
    fn steps(&self, _id: u32) -> u32 {
        0
    }
    fn step_name(&self, _id: u32, _i: u32) -> Option<String> {
        None
    }
    fn value_text(&self, _id: u32) -> String {
        String::new()
    }
    fn probe(&self, _name: &str) -> f64 {
        0.0
    }
    fn begin(&mut self, _id: u32) {}
    fn set(&mut self, id: u32, n: f64) {
        self.v.insert(id, n.clamp(0.0, 1.0));
    }
    fn end(&mut self, _id: u32) {}
}

#[test]
fn a_neuron_is_grabbed_where_it_is_drawn_and_follows_the_pointer() {
    let plugin = Engram::new();
    let table = plugin.param_table();
    let params: Vec<&dyn AnyParam> = (0..table.count()).filter_map(|i| table.by_index(i)).collect();
    let id = |name: &str| params.iter().find(|p| p.meta().name == name).unwrap().meta().id.0;
    let mut host = Table { v: params.iter().map(|p| (p.meta().id.0, p.default_normalized())).collect() };

    let mut rt = Runtime::new(plugin.editor_face().unwrap());
    let win = rt
        .rects()
        .into_iter()
        .find(|r| (r.w - (NEURONS_W + 2.0 * LIP)).abs() < 1e-6 && (r.h - (SCREEN_H + 2.0 * LIP)).abs() < 1e-6)
        .expect("the NEURONS window");
    let w = NEURONS_W - 2.0 * SIDE;
    let field = Rect { x: win.x + LIP + SIDE, y: win.y + LIP + FIELD_TOP, w, h: w / 1.9 };
    // The field draws y down.
    let drawn = |u: f64, v: f64| -> Pt { pt(field.x + u * field.w, field.y + v * field.h) };

    let (x, y) = (id("node 4 x"), id("node 4 y"));
    let (other_x, other_y) = (id("node 5 x"), id("node 5 y"));
    let (u0, v0) = (host.normalized(x), host.normalized(y));
    let others = (host.normalized(other_x), host.normalized(other_y));
    let at = drawn(u0, v0);

    assert!(rt.pointer_down(&mut host, at, Mods::default(), 1), "a press on the node grabs it");
    assert!((host.normalized(x) - u0).abs() < 1e-9 && (host.normalized(y) - v0).abs() < 1e-9, "grabbing never moves the node off the pointer");

    let to = pt(at.x + 30.0, at.y + 20.0);
    rt.pointer_drag(&mut host, to, Mods::default());
    let now = drawn(host.normalized(x), host.normalized(y));
    assert!((now.x - to.x).abs() < 1e-6 && (now.y - to.y).abs() < 1e-6, "the node is drawn under the pointer: {now:?} vs {to:?}");
    assert!(host.normalized(x) > u0 && host.normalized(y) > v0, "right and down raise x and y");
    rt.pointer_up(&mut host);
    assert_eq!((host.normalized(other_x), host.normalized(other_y)), others, "only the grabbed neuron moves");

    // Empty glass far from every node grabs nothing.
    let nodes: Vec<Pt> = (1..=16)
        .map(|i| drawn(host.normalized(id(&format!("node {i} x"))), host.normalized(id(&format!("node {i} y")))))
        .collect();
    let empty = (0..=20)
        .flat_map(|a| (0..=20).map(move |b| (a, b)))
        .map(|(a, b)| drawn(f64::from(a) / 20.0, f64::from(b) / 20.0))
        .find(|p| nodes.iter().all(|n| (n.x - p.x).hypot(n.y - p.y) > 20.0))
        .unwrap();
    let before = host.v.clone();
    assert!(!rt.pointer_down(&mut host, empty, Mods::default(), 1), "empty field grabs nothing");
    assert_eq!(host.v, before);
}
