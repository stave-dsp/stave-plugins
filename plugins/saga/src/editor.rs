//! The saga face — a pure function of params, laid out like the pedal:
//! the PROGRAM window says what this program is and what FEEDBACK /
//! REGEN / X mean right now, the seven pedal knobs sit under it, TWEAK
//! breaks the one-knob menu out, and two stomp switches (bypass, DIVE)
//! frame the clock and the tail along the toe.

use stave_core::face::prelude::*;
use stave_core::face::{step_of, CapStyle, Finish};
use stave_core::params::BindParam;
use stave_core::ui::photo::hex;
use stave_core::ui::scene::{Family, Font, TextAlign};

use crate::params::{SagaParams, MODES};

/// The mode parameter's id (the program window reads it). Checked
/// against the table in the tests below.
const MODE_ID: u32 = 3;

/// What each program does, and what its three changeable knobs mean.
const LEGENDS: [(&str, &str); 5] = [
    ("REVERSE DELAY → REVERSE REVERB", "FDBK repeats · REGEN reverse verb · X how reversed"),
    ("REVERSE DELAY → OCTAVE-UP REVERB", "FDBK repeats · REGEN octave up · X decay"),
    ("REVERSE DELAY → OCTAVE-DOWN REVERB", "FDBK repeats · REGEN octave down · X decay"),
    ("REVERSE REVERB → FORWARD REVERB", "FDBK oct down · REGEN oct up · X decay · TIME stretch"),
    ("PITCH DELAY → PITCH DELAY", "FDBK delay one · REGEN delay two · X interval order"),
];

/// Weathered copper hammertone, patina-green light, ivory caps.
fn theme() -> Theme {
    Theme {
        name: "saga",
        surface: hex(0x5A3322),
        finish: Finish::Hammertone,
        plate: hex(0x472818),
        desk: hex(0x1C1B1A),
        ink: Color::rgba(0.95, 0.90, 0.80, 1.0),
        ink_soft: Color::rgba(0.95, 0.90, 0.80, 0.58),
        accent: Color::rgba(0.50, 0.88, 0.76, 1.0),
        lamp: [0.45, 1.6, 1.05],
        caps: CapStyle::Soft,
        cap: hex(0xEDE3CC),
        pointer: Color::rgba(0.35, 0.18, 0.11, 1.0),
        screen: Color::rgba(0.03, 0.06, 0.05, 1.0),
        phosphor: Color::rgba(0.55, 0.95, 0.80, 1.0),
        radius: 16.0,
        ..Theme::graphite()
    }
}

/// The program window: the program's numeral, what it chains, and
/// what the changeable knobs do in it.
fn program(f: &Frame<'_>, r: Rect, s: &mut Scene) {
    let th = f.theme;
    let ph = th.phosphor;
    let soft = Color::rgba(ph.r, ph.g, ph.b, 0.55);
    let i = step_of(f.host.normalized(MODE_ID), MODES.len() as u32) as usize;
    let (title, keys) = LEGENDS[i.min(LEGENDS.len() - 1)];
    let left = Font::new(Family::Sans, 650).tracking(0.1).align(TextAlign::Left);
    let mono = Font::new(Family::Mono, 500).align(TextAlign::Left);
    let x = r.x + 18.0;
    s.text_in(pt(x, r.y + 22.0), "PROGRAM", 7.5, soft, left);
    s.text_in(pt(x, r.y + 60.0), MODES[i], 34.0, ph, Font::new(Family::Serif, 600).align(TextAlign::Left));
    let tx = r.x + 92.0;
    s.text_in(pt(tx, r.y + 44.0), title, 11.0, ph, left);
    s.text_in(pt(tx, r.y + 64.0), keys, 9.0, soft, mono);
    let rule = Color::rgba(ph.r, ph.g, ph.b, 0.12);
    s.stroke(Path::new().move_to(pt(tx - 14.0, r.y + 16.0)).line_to(pt(tx - 14.0, r.y + r.h - 16.0)), Stroke::new(rule, 1.0));
}

pub fn face(p: &SagaParams) -> Face {
    let theme = theme();
    let gap = theme.space(3);
    let wide = theme.space(4);

    let header = row(vec![
        Title::new("SAGA").tagline("reverse delays and reverbs, on a clock you can pull").node(),
        spacer(),
        Presets::default().node(),
    ])
    .gap(wide)
    .align(Align::Center);

    let window = Display::new(430.0, 84.0, program).reads(&[p.mode.bind()]).node();
    let programs = Plate::titled("PROGRAM").around(
        row(vec![Selector::new(p.mode.bind()).node(), window, Selector::new(p.sync.bind()).node()])
            .gap(wide)
            .align(Align::Center),
    );

    let echo = Plate::titled("ECHO").around(
        row(vec![
            Knob::new(p.time.bind()).large().node(),
            Knob::new(p.feedback.bind()).large().node(),
            Knob::new(p.regen.bind()).large().node(),
            Knob::new(p.x.bind()).large().node(),
        ])
        .gap(wide)
        .align(Align::End),
    );
    let color = Plate::titled("COLOR").around(
        row(vec![
            Knob::new(p.mod_depth.bind()).large().node(),
            Knob::new(p.tone.bind()).large().node(),
            Knob::new(p.mix.bind()).large().node(),
        ])
        .gap(wide)
        .align(Align::End),
    );

    let tweak = Plate::titled("TWEAK").around(
        row(vec![
            Knob::new(p.width.bind()).node(),
            Knob::new(p.mod_rate.bind()).node(),
            Toggle::new(p.mod_shape.bind()).label("SHAPE").node(),
            Knob::new(p.shift.bind()).node(),
            Knob::new(p.ramp.bind()).node(),
        ])
        .gap(gap)
        .align(Align::Center),
    );
    let out = Plate::titled("OUT").around(Knob::new(p.output.bind()).centered().node());

    let toe = row(vec![
        Footswitch::bypass(p.bypass.bind()).node(),
        spacer(),
        Trace::new("clock", 250.0, 64.0, (0.4, 2.1)).label("CLOCK").node(),
        Trace::new("tail", 250.0, 64.0, (0.0, 1.0)).label("TAIL").node(),
        spacer(),
        Footswitch::new(p.dive.bind()).label("DIVE / RISE").node(),
    ])
    .gap(wide)
    .align(Align::End);

    let body = column(vec![
        header,
        programs,
        row(vec![echo, color]).gap(gap).align(Align::Stretch),
        row(vec![tweak, out]).gap(gap).align(Align::Stretch),
        toe,
    ])
    .gap(theme.space(3))
    .align(Align::Stretch);
    Face::new(theme, Panel { margin: 18.0, pad: 24.0, ..Panel::default() }.around(body))
}

#[cfg(test)]
mod tests {
    use stave_core::params::AnyParam;

    #[test]
    fn the_program_window_reads_the_mode_parameter() {
        let p = crate::params::SagaParams::new();
        assert_eq!(p.mode.meta().id.0, super::MODE_ID);
    }
}
