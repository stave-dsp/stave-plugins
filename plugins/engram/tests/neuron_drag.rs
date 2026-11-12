//! The NEURONS window's nodes are grabbed where they are drawn: a press
//! on a neuron's drawn position grabs that neuron (and only it), and a
//! drag carries its x/y with the pointer, on the real face's layout.

use stave_engram_vst3::Engram;
use stave::face::Gesture;
use stave_test::face::FaceDriver;

/// The face's geometry, as the window draws it (vfd/neurons.rs).
const LIP: f64 = 5.0;
const SIDE: f64 = 10.0;
const FIELD_TOP: f64 = 32.0;
const NEURONS_W: f64 = 340.0;

#[test]
#[cfg_attr(miri, ignore)]
fn a_neuron_is_grabbed_where_it_is_drawn_and_follows_the_pointer() {
    let mut ui = FaceDriver::of::<Engram>();
    // The NEURONS window is named by its first read: node 1's x.
    let win = ui.rect("node 1 x");
    assert!((win.w - (NEURONS_W + 2.0 * LIP)).abs() < 1e-6, "the NEURONS window: {win:?}");
    let w = NEURONS_W - 2.0 * SIDE;
    let (fx, fy, fw, fh) = (win.x + LIP + SIDE, win.y + LIP + FIELD_TOP, w, w / 1.9);
    // The field draws y down.
    let drawn = |u: f64, v: f64| (fx + u * fw, fy + v * fh);

    let (u0, v0) = (ui.normalized("node 4 x"), ui.normalized("node 4 y"));
    let others = (ui.normalized("node 5 x"), ui.normalized("node 5 y"));
    let at = drawn(u0, v0);

    match ui.grab_at(at) {
        Some((_, Gesture::Pad { x, y, .. })) => assert_eq!((x.name, y.name), ("node 4 x", "node 4 y"), "a press on the node grabs it"),
        other => panic!("a press on node 4 grabs {other:?}"),
    }
    ui.press(at);
    assert!(
        (ui.normalized("node 4 x") - u0).abs() < 1e-9 && (ui.normalized("node 4 y") - v0).abs() < 1e-9,
        "grabbing never moves the node off the pointer"
    );

    let to = (at.0 + 30.0, at.1 + 20.0);
    ui.move_to(to);
    let now = drawn(ui.normalized("node 4 x"), ui.normalized("node 4 y"));
    assert!((now.0 - to.0).abs() < 1e-6 && (now.1 - to.1).abs() < 1e-6, "the node is drawn under the pointer: {now:?} vs {to:?}");
    assert!(ui.normalized("node 4 x") > u0 && ui.normalized("node 4 y") > v0, "right and down raise x and y");
    ui.release();
    assert_eq!((ui.normalized("node 5 x"), ui.normalized("node 5 y")), others, "only the grabbed neuron moves");
    ui.assert_bracketed();

    // Empty glass far from every node grabs nothing.
    let nodes: Vec<(f64, f64)> = (1..=16)
        .map(|i| drawn(ui.normalized(format!("node {i} x").as_str()), ui.normalized(format!("node {i} y").as_str())))
        .collect();
    let empty = (0..=20)
        .flat_map(|a| (0..=20).map(move |b| (a, b)))
        .map(|(a, b)| drawn(f64::from(a) / 20.0, f64::from(b) / 20.0))
        .find(|p| nodes.iter().all(|n| (n.0 - p.0).hypot(n.1 - p.1) > 20.0))
        .unwrap();
    assert_eq!(ui.grab_at(empty), None, "empty field grabs nothing");
    let before: Vec<f64> = (1..=16).map(|i| ui.normalized(format!("node {i} x").as_str())).collect();
    ui.click(empty);
    let after: Vec<f64> = (1..=16).map(|i| ui.normalized(format!("node {i} x").as_str())).collect();
    assert_eq!(after, before);
}
