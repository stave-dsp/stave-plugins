//! The opal face — the whole window, a pure function of params, on
//! a deep-violet anodized case with pearl knobs and an opal-fire
//! accent.
//!
//! ```text
//! header   OPAL · preset · VOICE/LFO/MOD pager · bypass
//! VOICE    ALGORITHM (stepper + live wiring diagram + feedback,
//!          vel→bright, output) · OPERATORS (OP 1–4 pager; each page
//!          an ADSR picture over ratio, level, A D S R)
//! LFO      LFO 1 · LFO 2
//! MOD      four route slots, one row each
//! ```
//!
//! Both pagers are editor-local view state — an op page is what you
//! are LOOKING at, not what the plugin sounds like, so it lives
//! outside params, automation, and the session entirely. The
//! algorithm stays param-bound: algorithms ARE sounds.

use stave::face::photo::hex;
use stave::face::{CapStyle, Finish, Hardware, Palette};
use stave::prelude::*;

use crate::engine::OpalTaps;
use crate::params::OpalParams;

/// The top page set (VOICE / LFO / MOD).
const PAGES: u32 = 1;
/// The operator page set (OP 1–4), nested in VOICE.
const OP_PAGES: u32 = 2;

/// Opal's finish: deep violet anodized, pearl caps, an opal-fire
/// pink accent over sea-green phosphor.
fn theme() -> Theme {
    let base = Theme::indigo();
    Theme {
        name: "opal",
        finish: Finish {
            surface: hex(0x2A2143),
            plate: hex(0x221A38),
            ..base.finish
        },
        palette: Palette {
            accent: Color::rgba(1.0, 0.56, 0.74, 1.0),
            phosphor: Color::rgba(0.55, 0.96, 0.90, 1.0),
            ..base.palette
        },
        hardware: Hardware {
            caps: CapStyle::Soft,
            cap: hex(0xE9E4EE),
            pointer: Color::rgba(0.20, 0.15, 0.36, 1.0),
            lamp: [1.6, 0.5, 1.1],
        },
        ..base
    }
}

/// An operator's ADSR picture, from the four binds its display
/// declares (attack, decay, sustain, release — `reads` in `op_page`).
fn envelope(f: &Frame<'_>, r: Rect, s: &mut Scene) {
    screens::envelope(f, r, s, f.bound(0), f.bound(1), f.bound(2), f.bound(3));
}

/// One operator page: its ADSR picture over ratio · level and the
/// four envelope knobs.
fn op_page(ratio: &dyn AnyParam, level: &dyn AnyParam, adsr: [&dyn AnyParam; 4], gap: f64) -> FaceNode {
    let binds: Vec<Bind> = adsr.iter().map(|p| p.bind()).collect();
    column(vec![
        Display::new(500.0, 104.0, envelope).reads(&binds).node(),
        row(vec![
            Knob::new(ratio.bind()).label("RATIO").large().node(),
            Knob::new(level.bind()).label("LEVEL").large().node(),
            Knob::new(adsr[0].bind()).label("ATTACK").node(),
            Knob::new(adsr[1].bind()).label("DECAY").node(),
            Knob::new(adsr[2].bind()).label("SUSTAIN").node(),
            Knob::new(adsr[3].bind()).label("RELEASE").node(),
        ])
        .gap(gap)
        .align(Align::End),
    ])
    .gap(gap)
    .align(Align::Center)
}

/// One LFO's full voice of controls.
fn lfo_plate(
    title: &'static str,
    rate: &dyn AnyParam,
    rise: &dyn AnyParam,
    shape: &dyn AnyParam,
    sync: &dyn AnyParam,
    trig: &dyn AnyParam,
    gap: f64,
) -> FaceNode {
    Plate::titled(title).around(
        row(vec![
            Knob::new(rate.bind()).label("RATE").large().node(),
            Knob::new(rise.bind()).label("RISE").large().node(),
            Selector::new(shape.bind()).label("SHAPE").node(),
            Selector::new(sync.bind()).label("SYNC").node(),
            Toggle::new(trig.bind()).label("TRIG").node(),
        ])
        .gap(gap + 8.0)
        .align(Align::Center),
    )
}

pub fn face(p: &OpalParams) -> Face {
    let theme = theme();
    let gap = theme.space(3);

    let header = row(vec![
        Title::new("OPAL").tagline("four-operator fm synth").node(),
        Presets { width: 188.0 }.node(),
        Pager::new(PAGES, &["VOICE", "LFO", "MOD"]).node(),
        Footswitch::bypass(p.bypass.bind()).compact().node(),
    ])
    .gap(theme.space(4))
    .align(Align::Center);

    // ---- VOICE ---------------------------------------------------
    let algorithm = Plate::titled("GLOBAL").around(
        column(vec![
            Stepper::new(p.algo.bind()).label("ALGORITHM").width(212.0).node(),
            Display::new(212.0, 128.0, crate::diagram::draw).reads(&[p.algo.bind()]).node(),
            row(vec![
                Knob::new(p.feedback.bind()).node(),
                Knob::new(p.vel_mod.bind()).label("VEL→BRT").node(),
                Knob::new(p.output.bind()).centered().node(),
            ])
            .gap(0.0),
        ])
        .gap(gap)
        .align(Align::Center),
    );
    let ops = [
        op_page(&p.ratio1, &p.level1, [&p.attack1, &p.decay1, &p.sustain1, &p.release1], gap),
        op_page(&p.ratio2, &p.level2, [&p.attack2, &p.decay2, &p.sustain2, &p.release2], gap),
        op_page(&p.ratio3, &p.level3, [&p.attack3, &p.decay3, &p.sustain3, &p.release3], gap),
        op_page(&p.ratio4, &p.level4, [&p.attack4, &p.decay4, &p.sustain4, &p.release4], gap),
    ];
    let operators = Plate::titled("OPERATORS").around(
        column(vec![
            Pager::new(OP_PAGES, crate::params::OPS).node(),
            pages(OP_PAGES, ops.into()),
        ])
        .gap(gap)
        .align(Align::Center),
    );
    let voice_page = row(vec![algorithm, operators]).gap(gap).align(Align::Stretch);

    // ---- LFO -----------------------------------------------------
    let lfo_page = column(vec![
        lfo_plate("LFO 1", &p.lfo_rate, &p.lfo_rise, &p.lfo_shape, &p.lfo_sync, &p.lfo_trig, gap),
        lfo_plate("LFO 2", &p.lfo2_rate, &p.lfo2_rise, &p.lfo2_shape, &p.lfo2_sync, &p.lfo2_trig, gap),
    ])
    .gap(gap)
    .align(Align::Stretch);

    // ---- MOD -----------------------------------------------------
    const LABELS: [&str; 4] = ["1", "2", "3", "4"];
    let mod_page = Plate::titled("MODULATION").around(
        column(p.routes.iter().enumerate().map(|(i, r)| r.row(OpalTaps::MODS[i], LABELS[i])).collect())
            .gap(gap)
            .align(Align::Center),
    );

    let pages = pages(PAGES, vec![voice_page, lfo_page, mod_page]);
    let body = column(vec![header, pages]).gap(theme.space(4)).align(Align::Stretch);
    Face::new(theme, Panel::default().around(body))
}
