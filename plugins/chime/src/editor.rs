//! The chime face — the whole window, a pure function of params, on
//! a warm ivory powder coat (a bell's case, not a rack's).
//!
//! ```text
//! header   CHIME · VOICE/MOD pager · bypass
//! VOICE    ENVELOPE (picture + A D S R) · VOICE (shape, output)
//!          LFO (rate, rise, shape, sync, trig)
//! MOD      four route slots, one row each: live source trace ·
//!          source · destination · amount · via · mode · lag · quant
//! ```

extern crate alloc;
use alloc::vec;

use stave_core::face::prelude::*;
use stave_core::params::{BindParam, ParamEnum, RouteParams};

use crate::params::ChimeParams;

/// The editor-local page set (VOICE / MOD) — view state, never a param.
const PAGES: u32 = 1;

/// Chime's finish: the cream powder coat, warmed toward ivory, with a
/// bell-bronze accent.
fn theme() -> Theme {
    use stave_ui::photo::hex;
    Theme {
        name: "chime",
        surface: hex(0xEFE6D2),
        plate: hex(0xE4D8BE),
        accent: Color::rgba(0.62, 0.38, 0.16, 1.0),
        phosphor: Color::rgba(1.0, 0.74, 0.42, 1.0),
        ..Theme::cream()
    }
}

/// The ADSR picture, from the four binds the display declares
/// (attack, decay, sustain, release — `reads` in `face`).
fn envelope(f: &Frame<'_>, r: Rect, s: &mut Scene) {
    screens::envelope(f, r, s, f.bound(0), f.bound(1), f.bound(2), f.bound(3));
}

/// One route slot as a channel strip: the source's live trace (the
/// slot's number printed on it), then every one of its seven params.
pub(crate) fn route_row<S: ParamEnum, T: ParamEnum>(
    r: &RouteParams<S, T>,
    probe: &'static str,
    label: &'static str,
) -> FaceNode {
    row(vec![
        Trace::new(probe, 64.0, 44.0, (-1.0, 1.0)).label(label).node(),
        Stepper::new(r.src.bind()).label("SOURCE").width(120.0).node(),
        Stepper::new(r.tgt.bind()).label("DEST").width(120.0).node(),
        Knob::new(r.depth.bind()).small().centered().label("AMOUNT").node(),
        Stepper::new(r.via.bind()).label("VIA").width(120.0).node(),
        Toggle::new(r.mode.bind()).label("MODE").node(),
        Knob::new(r.lag.bind()).small().label("LAG").node(),
        Stepper::new(r.quant.bind()).label("QUANT").width(96.0).node(),
    ])
    .gap(10.0)
    .align(Align::Center)
}

pub fn face(p: &ChimeParams) -> Face {
    let theme = theme();
    let gap = theme.space(3);

    let header = row(vec![
        Title::new("CHIME").tagline("polyphonic sine · saw synth").node(),
        Pager::new(PAGES, &["VOICE", "MOD"]).node(),
        Footswitch::bypass(p.bypass.bind()).compact().node(),
    ])
    .gap(theme.space(4))
    .align(Align::Center);

    // ---- VOICE ---------------------------------------------------
    let env = Plate::titled("ENVELOPE").around(
        column(vec![
            Display::new(500.0, 112.0, envelope)
                .reads(&[p.attack.bind(), p.decay.bind(), p.sustain.bind(), p.release.bind()])
                .node(),
            row(vec![
                Knob::new(p.attack.bind()).large().node(),
                Knob::new(p.decay.bind()).large().node(),
                Knob::new(p.sustain.bind()).large().node(),
                Knob::new(p.release.bind()).large().node(),
            ])
            .gap(theme.space(5)),
        ])
        .gap(gap)
        .align(Align::Center),
    );
    let voice = Plate::titled("VOICE").around(
        column(vec![
            Toggle::new(p.shape.bind()).label("SHAPE").node(),
            Knob::new(p.output.bind()).huge().centered().node(),
        ])
        .gap(gap)
        .align(Align::Center),
    );
    let lfo = Plate::titled("LFO").around(
        row(vec![
            Knob::new(p.lfo_rate.bind()).label("RATE").large().node(),
            Knob::new(p.lfo_rise.bind()).label("RISE").large().node(),
            Selector::new(p.lfo_shape.bind()).label("SHAPE").node(),
            Selector::new(p.lfo_sync.bind()).label("SYNC").node(),
            Toggle::new(p.lfo_trig.bind()).label("TRIG").node(),
        ])
        .gap(theme.space(5))
        .align(Align::Center),
    );
    let voice_page = column(vec![row(vec![env, voice]).gap(gap).align(Align::Stretch), lfo])
        .gap(gap)
        .align(Align::Stretch);

    // ---- MOD -----------------------------------------------------
    const PROBES: [&str; 4] = ["mod 1", "mod 2", "mod 3", "mod 4"];
    const LABELS: [&str; 4] = ["1", "2", "3", "4"];
    let routes = Plate::titled("MODULATION").around(
        column(p.routes.iter().enumerate().map(|(i, r)| route_row(r, PROBES[i], LABELS[i])).collect())
            .gap(gap)
            .align(Align::Center),
    );
    let pages = pages(PAGES, vec![voice_page, routes]);
    let body = column(vec![header, pages]).gap(theme.space(4)).align(Align::Stretch);
    Face::new(theme, Panel::default().around(body))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every page fits the window budget.
    #[test]
    fn the_face_fits() {
        let rt = stave_core::face::Runtime::new(face(&ChimeParams::new()));
        let (w, h) = rt.size();
        assert!(w <= 1000.0 && h <= 720.0, "chime face is {w}×{h}");
    }
}
