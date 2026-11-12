//! The Smear face — the whole window, a pure function of params: a
//! plum-enamel instrument with the garden itself on glass (two octaves
//! of sympathetic strings, ringing with the output and drifting with
//! FLUX/MOTION), the four garden controls as the hero row, then
//! motion, tone and output.

use stave::face::photo::hex;
use stave::face::{CapStyle, Coat, Finish, Hardware, Palette};
use stave::prelude::*;

use crate::{SmearParams, SmearTaps};

/// Smear's look: plum enamel, ivory print, a soft-gold light.
pub fn theme() -> Theme {
    let base = Theme::verdigris();
    Theme {
        name: "smear",
        finish: Finish {
            coat: Coat::Enamel,
            surface: hex(0x5B2F4E),
            plate: hex(0x4A253F),
            desk: hex(0x1E1B1F),
            ..base.finish
        },
        palette: Palette {
            ink: Color::rgba(0.96, 0.91, 0.84, 1.0),
            ink_soft: Color::rgba(0.96, 0.91, 0.84, 0.6),
            accent: Color::rgba(0.98, 0.80, 0.52, 1.0),
            screen: Color::rgba(0.05, 0.03, 0.05, 1.0),
            phosphor: Color::rgba(1.0, 0.78, 0.86, 1.0),
        },
        hardware: Hardware {
            caps: CapStyle::Soft,
            cap: hex(0xEFE4D2),
            pointer: Color::rgba(0.42, 0.17, 0.35, 1.0),
            lamp: [1.6, 0.9, 0.5],
        },
        ..base
    }
}

pub fn face(p: &SmearParams) -> Face {
    let theme = theme();
    let gap = theme.space(3);
    let row_gap = theme.space(3);

    let header = row(vec![
        Title::new("SMEAR")
            .tagline("sympathetic garden · resonator field")
            .node(),
        spacer(),
    ])
    .align(Align::Center);

    let garden_view = Display::new(560.0, 150.0, garden::draw)
        .taps(&[SmearTaps::IO_OUT])
        .reads(&[
            p.smear.bind(),
            p.bloom.bind(),
            p.decay.bind(),
            p.flux.bind(),
            p.shimmer.bind(),
            p.motion.bind(),
            p.depth.bind(),
        ])
        .animated()
        .node();
    let top = row(vec![
        column(vec![Meter::db(SmearTaps::IO_IN).label("IN").height(150.0).node()]).align(Align::Center),
        garden_view,
        column(vec![Meter::db(SmearTaps::IO_OUT).label("OUT").height(150.0).node()]).align(Align::Center),
    ])
    .gap(row_gap)
    .align(Align::Center)
    .justify(Justify::Center);

    let knobs = |ks: Vec<FaceNode>| row(ks).gap(gap).align(Align::Center);
    let garden = Plate::titled("GARDEN").around(
        row(vec![
            Knob::new(p.seed.bind()).huge().node(),
            Knob::new(p.smear.bind()).huge().node(),
            Knob::new(p.bloom.bind()).huge().node(),
            Knob::new(p.decay.bind()).huge().node(),
        ])
        .gap(theme.space(5) + 16.0)
        .align(Align::Center),
    );
    let motion = Plate::titled("MOTION").around(knobs(vec![
        Knob::new(p.flux.bind()).node(),
        Knob::new(p.shimmer.bind()).node(),
        Knob::new(p.motion.bind()).node(),
        Knob::new(p.depth.bind()).node(),
    ]));
    let tone = Plate::titled("TONE").around(knobs(vec![
        Knob::new(p.drive.bind()).node(),
        Knob::new(p.tone.bind()).node(),
    ]));
    let output = Plate::titled("OUTPUT").around(knobs(vec![
        Knob::new(p.mix.bind()).node(),
        Knob::new(p.output.bind()).node(),
    ]));

    let body = column(vec![
        header,
        top,
        garden,
        row(vec![motion, tone, output])
            .gap(row_gap)
            .align(Align::Stretch),
    ])
    .gap(row_gap)
    .align(Align::Stretch);
    Face::new(theme, Panel::default().around(body))
}

/// The garden on glass.
mod garden {
    use stave::prelude::*;

    use crate::SmearTaps;

    fn alpha(c: Color, a: f64) -> Color {
        Color::rgba(c.r, c.g, c.b, c.a * a)
    }

    /// Two octaves of strings across the screen, each a standing wave
    /// whose swing follows the output level; SMEAR bleeds each string's
    /// glow into its neighbours, DECAY slows the shimmer of the swing,
    /// FLUX and MOTION drift the tuning (the strings sway), SHIMMER
    /// lights the octave partials, DEPTH spreads the lanes apart.
    pub fn draw(f: &Frame<'_>, r: Rect, s: &mut Scene) {
        let th = f.theme;
        // The binds `face` declares, in its order.
        let (smear, bloom, decay) = (f.bound(0), f.bound(1), f.bound(2));
        let (flux, shimmer, motion, depth) = (f.bound(3), f.bound(4), f.bound(5), f.bound(6));
        let level = ((f.tap(SmearTaps::IO_OUT) + 60.0) / 60.0).clamp(0.0, 1.0);
        let t = f.time;

        screens::grid(f, r, s, 12, 4);
        const N: usize = 24;
        let pitch = (r.w - 24.0) / (N - 1) as f64;
        let span = r.h / 2.0 - 10.0;
        // Each string's own excitation: a slow pattern over the lattice,
        // bled sideways by SMEAR.
        let raw: Vec<f64> = (0..N)
            .map(|i| {
                let k = i as f64;
                let a = 0.5
                    + 0.5
                        * (k * 0.83 + t * (0.15 + 0.6 * flux)).sin()
                        * (k * 0.37 - t * 0.21).cos();
                a * a
            })
            .collect();
        let glow: Vec<f64> = (0..N)
            .map(|i| {
                let l = raw[i.saturating_sub(1)];
                let rr = raw[(i + 1).min(N - 1)];
                (raw[i] * (1.0 - 0.6 * smear) + 0.3 * smear * (l + rr)).min(1.0)
            })
            .collect();
        let energy = 0.2 + 0.8 * level;
        for (i, g) in glow.iter().enumerate() {
            let x0 = r.x + 12.0 + i as f64 * pitch;
            let sway = (motion * 3.0 + depth * 2.0) * (t * (0.3 + flux) + i as f64 * 1.7).sin();
            let amp = span * energy * (0.25 + 0.75 * g) * (0.4 + 0.6 * bloom);
            let phase = t * (2.0 + 6.0 * (1.0 - decay)) + i as f64 * 0.9;
            let swing = phase.sin();
            let n = 40;
            let pts: Vec<_> = (0..=n)
                .map(|j| {
                    let u = j as f64 / n as f64;
                    let y = r.y + 8.0 + u * (r.h - 16.0);
                    let mode = (core::f64::consts::PI * u).sin();
                    let oct =
                        shimmer * 0.5 * (core::f64::consts::TAU * u).sin() * (phase * 2.0).cos();
                    pt(
                        x0 + sway * mode + (amp / span) * 9.0 * (mode * swing + oct),
                        y,
                    )
                })
                .collect();
            let a = 0.25 + 0.75 * g * energy;
            s.stroke(
                Path::polyline(&pts),
                Stroke::round(alpha(th.palette.phosphor, 0.12 * a), 6.0),
            );
            s.stroke(
                Path::polyline(&pts),
                Stroke::round(alpha(th.palette.phosphor, 0.35 + 0.6 * a), 1.2),
            );
            // The string's node: a bead of light at the antinode.
            let c = pts[n / 2];
            s.solid(
                Path::circle(c, 2.0 + 2.5 * g * energy),
                alpha(th.palette.accent, 0.3 + 0.6 * a),
            );
        }
    }
}
