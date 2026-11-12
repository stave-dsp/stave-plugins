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

use stave::face::photo::hex;
use stave::face::{Finish, Palette};
use stave::prelude::*;

use crate::engine::ChimeTaps;
use crate::params::ChimeParams;

/// The editor-local page set (VOICE / MOD) — view state, never a param.
const PAGES: u32 = 1;

/// Chime's finish: the cream powder coat, warmed toward ivory, with a
/// bell-bronze accent.
fn theme() -> Theme {
    let base = Theme::cream();
    Theme {
        name: "chime",
        finish: Finish {
            surface: hex(0xEFE6D2),
            plate: hex(0xE4D8BE),
            ..base.finish
        },
        palette: Palette {
            accent: Color::rgba(0.62, 0.38, 0.16, 1.0),
            phosphor: Color::rgba(1.0, 0.74, 0.42, 1.0),
            ..base.palette
        },
        ..base
    }
}

/// The ADSR picture, from the four binds the display declares
/// (attack, decay, sustain, release — `reads` in `face`).
fn envelope(f: &Frame<'_>, r: Rect, s: &mut Scene) {
    screens::envelope(f, r, s, f.bound(0), f.bound(1), f.bound(2), f.bound(3));
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
    const LABELS: [&str; 4] = ["1", "2", "3", "4"];
    let routes = Plate::titled("MODULATION").around(
        column(p.routes.iter().enumerate().map(|(i, r)| r.row(ChimeTaps::MODS[i], LABELS[i])).collect())
            .gap(gap)
            .align(Align::Center),
    );
    let pages = pages(PAGES, vec![voice_page, routes]);
    let body = column(vec![header, pages]).gap(theme.space(4)).align(Align::Stretch);
    Face::new(theme, Panel::default().around(body))
}
