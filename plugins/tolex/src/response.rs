//! The response screen — tolex's own picture: the cabinet's computed
//! magnitude curve, live from the knobs, drawn over the reference
//! population's p10–p90 territory (aggregate statistics baked in
//! curve.rs). Every knob is legible here: damping blooms the low
//! resonance, position walks the tilt and the teeth, doping calms
//! them, mass slides the cliff. The curve is the SAME math as the
//! sound (the engine's display twins) and a law renders audio to keep
//! the picture honest (tests/curve_properties.rs).

extern crate alloc;
use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec::Vec;
use std::sync::LazyLock;

use stave_core::face::prelude::*;
use stave_core::face::step_of;
use stave_core::params::{BindParam, FloatParam};
use stave_ui::scene::TextAlign;

use crate::curve::{self, Sheet};
use crate::params::TolexParams;

static P: LazyLock<Arc<TolexParams>> = LazyLock::new(TolexParams::new);

const N_PTS: usize = 140;
const F_LO: f64 = 45.0;
const F_HI: f64 = 16_000.0;
const DB_TOP: f64 = 14.0;
const DB_BOT: f64 = -36.0;
const PAD_L: f64 = 8.0;
const PAD_R: f64 = 8.0;
const PAD_T: f64 = 10.0;
const PAD_B: f64 = 16.0;

fn alpha(c: Color, a: f64) -> Color {
    Color::rgba(c.r, c.g, c.b, c.a * a)
}

/// The parameters the picture reads.
pub fn reads(p: &TolexParams) -> Vec<Bind> {
    [
        p.size.bind(),
        p.resonance.bind(),
        p.q.bind(),
        p.damping.bind(),
        p.back.bind(),
        p.cab.bind(),
        p.mass.bind(),
        p.breakup.bind(),
        p.doping.bind(),
        p.position.bind(),
        p.mic.bind(),
        p.material.bind(),
        p.model.bind(),
        p.speakers.bind(),
        p.cone.bind(),
    ]
    .to_vec()
}

/// The build sheet at the face's current (normalized) values.
fn sheet(f: &Frame<'_>) -> Sheet {
    let p = &**P;
    let plain = |q: &FloatParam| q.range().denormalize(f.value(q.bind())) as f32;
    let step = |b: Bind| step_of(f.value(b), f.host.steps(b.id)) as usize;
    Sheet {
        size: step(p.size.bind()),
        resonance: plain(&p.resonance),
        q: plain(&p.q),
        damping: plain(&p.damping),
        open: step(p.back.bind()) >= 1,
        cab_l: plain(&p.cab),
        mass: plain(&p.mass),
        breakup: plain(&p.breakup),
        doping: plain(&p.doping),
        position: plain(&p.position),
        mic_cm: plain(&p.mic),
        material: step(p.material.bind()),
        model: step(p.model.bind()),
        layout: step(p.speakers.bind()),
        variant: plain(&p.cone).round().max(0.0) as usize,
    }
}

pub fn draw(f: &Frame<'_>, r: Rect, s: &mut Scene) {
    let th = f.theme;
    let ty = &th.type_;
    let x_of =
        |hz: f64| r.x + PAD_L + (hz / F_LO).ln() / (F_HI / F_LO).ln() * (r.w - PAD_L - PAD_R);
    let y_of = |db: f64| {
        r.y + PAD_T
            + (DB_TOP - db.clamp(DB_BOT, DB_TOP)) / (DB_TOP - DB_BOT) * (r.h - PAD_T - PAD_B)
    };

    // Grid: dB rules every 12, decade frequency rules with labels.
    let grid = Stroke::new(alpha(th.phosphor, 0.09), 1.0);
    let label = alpha(th.phosphor, 0.45);
    let left = ty.value.align(TextAlign::Left);
    for db in [-24.0, -12.0, 0.0] {
        s.stroke(
            Path::new()
                .move_to(pt(r.x + PAD_L, y_of(db)))
                .line_to(pt(r.x + r.w - PAD_R, y_of(db))),
            grid,
        );
        s.text_in(
            pt(r.x + PAD_L + 2.0, y_of(db) - 3.0),
            (db as i32).to_string(),
            7.5,
            label,
            left,
        );
    }
    for (hz, l) in [(100.0, "100"), (1_000.0, "1k"), (10_000.0, "10k")] {
        s.stroke(
            Path::new()
                .move_to(pt(x_of(hz), r.y + PAD_T))
                .line_to(pt(x_of(hz), r.y + r.h - PAD_B)),
            grid,
        );
        s.text_in(pt(x_of(hz) + 3.0, r.y + r.h - 5.0), l, 7.5, label, left);
    }

    // The population territory: p90 rail forward, p10 rail back.
    let band_n = 72;
    let at = |i: usize| F_LO * (F_HI / F_LO).powf(i as f64 / (band_n - 1) as f64);
    let mut band = Path::new();
    for i in 0..band_n {
        let p = pt(
            x_of(at(i)),
            y_of(f64::from(curve::envelope_at(at(i) as f32, true))),
        );
        band = if i == 0 {
            band.move_to(p)
        } else {
            band.line_to(p)
        };
    }
    for i in (0..band_n).rev() {
        band = band.line_to(pt(
            x_of(at(i)),
            y_of(f64::from(curve::envelope_at(at(i) as f32, false))),
        ));
    }
    s.solid(band.close(), alpha(th.accent, 0.10));

    // Two curves, same math as the sound: the RESTING sheet as a ghost,
    // the LIVE curve at the motor's operating point on top.
    let sheet = sheet(f);
    let excursion = f.probe("excursion") as f32;
    let mut hz = [0.0f32; N_PTS];
    for (i, slot) in hz.iter_mut().enumerate() {
        *slot = (F_LO * (F_HI / F_LO).powf(i as f64 / (N_PTS - 1) as f64)) as f32;
    }
    let to_pts = |db: &[f32; N_PTS]| -> Vec<_> {
        hz.iter()
            .zip(db.iter())
            .map(|(h, d)| pt(x_of(f64::from(*h)), y_of(f64::from(*d))))
            .collect()
    };
    let mut db = [0.0f32; N_PTS];
    if excursion > 0.01 {
        curve::response_into(&sheet, 0.0, &hz, &mut db);
        s.stroke(
            Path::polyline(&to_pts(&db)),
            Stroke::new(alpha(th.ink, 0.35), 1.0),
        );
    }
    curve::response_into(&sheet, excursion, &hz, &mut db);
    screens::glow_line(f, s, &to_pts(&db));
}
