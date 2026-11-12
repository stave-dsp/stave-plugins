//! The Multiband face — the whole window, a pure function of params.
//! Up top, the whole program at once: the output spectrum with the
//! crossovers, each band's threshold and its gain reduction drawn in,
//! the three GR meters and the crossover knobs. Below, one band at a
//! time (LOW / MID / HIGH pages): its compressor, its transfer curve
//! with solo and mute, its envelope and its colour.

use stave::face::photo::hex;
use stave::face::{CapStyle, Coat, Finish, Hardware, Palette};
use stave::prelude::*;

use crate::params::{BandParams, MbParams};
use crate::{views, MultibandTaps};

/// Space between a band page's plates.
const PLATE_GAP: f64 = 8.0;

/// The band page set.
const BANDS_KEY: u32 = 1;

/// Multiband's look: cool studio graphite, aluminium caps, a green
/// phosphor and amber light.
pub fn theme() -> Theme {
    let base = Theme::graphite();
    Theme {
        name: "multiband",
        finish: Finish {
            coat: Coat::Anodized,
            surface: hex(0x33383B),
            plate: hex(0x272B2E),
            desk: hex(0x1B1D1E),
            ..base.finish
        },
        palette: Palette {
            ink: Color::rgba(0.92, 0.93, 0.90, 1.0),
            ink_soft: Color::rgba(0.92, 0.93, 0.90, 0.55),
            accent: Color::rgba(0.62, 0.90, 0.48, 1.0),
            phosphor: Color::rgba(0.66, 0.92, 0.52, 1.0),
            screen: Color::rgba(0.03, 0.05, 0.04, 1.0),
        },
        hardware: Hardware {
            lamp: [0.55, 1.5, 0.35],
            caps: CapStyle::Aluminium,
            ..base.hardware
        },
        ..base
    }
}

fn knobs(ks: Vec<FaceNode>, gap: f64) -> FaceNode {
    row(ks).gap(gap).align(Align::Center)
}

/// One band's page.
fn band_page(b: &BandParams, transfer: Draw, gr: TapId, gap: f64) -> FaceNode {
    let dynamics = Plate::titled("COMPRESSOR").around(knobs(
        vec![
            Selector::new(b.style.bind()).label("STYLE").node(),
            Knob::new(b.threshold.bind()).label("THRESHOLD").node(),
            Knob::new(b.ratio.bind()).label("RATIO").node(),
            Knob::new(b.knee.bind()).label("KNEE").node(),
            Knob::new(b.makeup.bind()).label("MAKEUP").node(),
        ],
        gap,
    ));
    let curve = Plate::titled("TRANSFER · SOLO · MUTE").around(knobs(
        vec![
            Display::new(150.0, 98.0, transfer)
                .taps(&[gr])
                .reads(&[
                    b.threshold.bind(),
                    b.ratio.bind(),
                    b.knee.bind(),
                    b.makeup.bind(),
                    b.style.bind(),
                ])
                .node(),
            Footswitch::new(b.solo.bind())
                .compact()
                .label("SOLO")
                .node(),
            Footswitch::new(b.mute.bind())
                .compact()
                .label("MUTE")
                .node(),
        ],
        gap,
    ));
    let envelope = Plate::titled("DETECTOR").around(knobs(
        vec![
            Knob::new(b.attack.bind()).label("ATTACK").node(),
            Knob::new(b.release.bind()).label("RELEASE").node(),
            Knob::new(b.transient.bind()).label("TRANSIENT").node(),
            Knob::new(b.rms.bind()).label("RMS").node(),
            Knob::new(b.sidechain_hpf.bind()).label("SC HPF").node(),
        ],
        gap,
    ));
    let colour = Plate::titled("COLOUR").around(knobs(
        vec![
            Selector::new(b.saturation.bind())
                .label("SATURATION")
                .node(),
            Knob::new(b.drive.bind()).label("DRIVE").node(),
            Knob::new(b.saturation_mix.bind()).label("SAT MIX").node(),
            Knob::new(b.mix.bind()).label("MIX").node(),
        ],
        gap,
    ));
    column(vec![
        row(vec![dynamics, curve])
            .gap(PLATE_GAP)
            .align(Align::Stretch),
        row(vec![envelope, colour])
            .gap(PLATE_GAP)
            .align(Align::Stretch),
    ])
    .gap(PLATE_GAP)
    .align(Align::Stretch)
}

pub fn face(p: &MbParams) -> Face {
    let theme = theme();
    let gap = theme.space(2);
    let row_gap = theme.space(3);
    let [low, mid, high] = &p.bands;

    let header = row(vec![
        Title::new("MULTIBAND")
            .tagline("three-band dynamics · LR4 split")
            .node(),
        spacer(),
        Pager::new(BANDS_KEY, &["LOW", "MID", "HIGH"]).node(),
        spacer(),
        Footswitch::new(p.external_sidechain.bind())
            .compact()
            .label("EXT SC")
            .node(),
        Footswitch::bypass(p.bypass.bind()).compact().node(),
    ])
    .gap(theme.space(4))
    .align(Align::Center);

    let mut reads = vec![p.xover_low.bind(), p.xover_high.bind()];
    for b in &p.bands {
        reads.push(b.threshold.bind());
    }
    let spectrum = Display::new(450.0, 118.0, views::spectrum)
        .taps(&[
            MultibandTaps::SAMPLE_RATE,
            MultibandTaps::GR_LO,
            MultibandTaps::GR_MID,
            MultibandTaps::GR_HI,
        ])
        .reads(&reads)
        .animated()
        .node();
    let gr = Display::new(112.0, 118.0, views::reduction)
        .taps(&MultibandTaps::GR)
        .node();
    let crossover = Plate::titled("CROSSOVER").around(
        row(vec![
            Knob::new(p.xover_low.bind()).label("LOW / MID").node(),
            Knob::new(p.xover_high.bind()).label("MID / HIGH").node(),
        ])
        .gap(gap)
        .align(Align::Center),
    );
    let top = row(vec![spectrum, gr, crossover])
        .gap(row_gap)
        .align(Align::Center);

    let pages = stack(vec![
        on_page(
            band_page(low, views::transfer_lo, MultibandTaps::GR_LO, theme.space(0)),
            BANDS_KEY,
            0,
        ),
        on_page(
            band_page(mid, views::transfer_mid, MultibandTaps::GR_MID, theme.space(0)),
            BANDS_KEY,
            1,
        ),
        on_page(
            band_page(high, views::transfer_hi, MultibandTaps::GR_HI, theme.space(0)),
            BANDS_KEY,
            2,
        ),
    ]);

    let body = column(vec![header, top, pages])
        .gap(row_gap)
        .align(Align::Stretch);
    let panel = Panel::rack(10).handles();
    Face::new(theme, panel.around(body))
}
