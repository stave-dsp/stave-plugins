use stave::dsp::db_to_amp;
use stave::dsp::{tau_coeff64, Koren, PushPull, Stage};
use stave::dsp::units::{db, seconds};

use crate::controls::DynamicSettings;

use super::dynamic_headroom::{
    AdaptiveHarmonicController, FrequencySelectivePowerSupply, TransientHeadroomController,
};

const POWER_PAIR_INPUT_GAIN_DB: f32 = 3.0;

#[derive(Clone, Copy, Debug, Default)]
pub struct PowerAmplifierMeasurements {
    pub transient_amount: f32,
    pub plate_draw: f32,
    pub low_frequency_reservoir_voltage: f32,
    pub middle_frequency_reservoir_voltage: f32,
    pub high_frequency_reservoir_voltage: f32,
}

/// Halo's bias walk: sustained plate draw cools the pair by up to
/// `volts` (all of it at `full_at_draw` × idle) on an RC — a declared
/// effect of the voicing, applied as a bias offset on the pair's
/// physical bias.
#[derive(Clone)]
struct BiasWalk {
    volts: f64,
    full_at_draw: f64,
    coeff: f64,
    position: f64,
}

impl BiasWalk {
    const SECONDS: f64 = 0.110;

    fn new() -> Self {
        Self {
            volts: 0.0,
            full_at_draw: 5.0,
            coeff: 0.0,
            position: 0.0,
        }
    }

    /// Clear every stage's running state, settings kept.
    pub fn reset(&mut self) {
        self.position = 0.0;
    }

    fn prepare(&mut self, sample_rate: f32) {
        self.coeff = tau_coeff64(seconds(Self::SECONDS), f64::from(sample_rate));
        self.position = 0.0;
    }

    /// Volts colder for this plate draw.
    #[inline]
    fn tick(&mut self, draw: f32) -> f64 {
        let target = ((f64::from(draw) - 1.0) / (self.full_at_draw - 1.0)).clamp(0.0, 1.0);
        self.position += self.coeff * (target - self.position);
        self.position * self.volts
    }
}

#[derive(Clone)]
pub struct CleanPowerAmplifier {
    power_pair: PushPull,
    bias_walk: BiasWalk,
    power_supply: FrequencySelectivePowerSupply,
    transient_headroom_controller: TransientHeadroomController,
    harmonic_controller: AdaptiveHarmonicController,
    input_gain: f32,
    measurements: PowerAmplifierMeasurements,
}

impl CleanPowerAmplifier {
    pub fn new() -> Self {
        Self {
            power_pair: PushPull::new(),
            bias_walk: BiasWalk::new(),
            power_supply: FrequencySelectivePowerSupply::new(),
            transient_headroom_controller: TransientHeadroomController::new(),
            harmonic_controller: AdaptiveHarmonicController::new(),
            input_gain: db_to_amp(POWER_PAIR_INPUT_GAIN_DB),
            measurements: PowerAmplifierMeasurements::default(),
        }
    }

    /// Clear every stage's running state, settings kept.
    pub fn reset(&mut self) {
        self.power_pair.reset();
        // The pair's per-sample inputs (bias walk, supply) back to idle.
        self.power_pair.set_bias_offset(0.0);
        self.power_pair.set_supply_scale(1.0);
        self.bias_walk.reset();
        self.power_supply.reset();
        self.transient_headroom_controller.reset();
        self.harmonic_controller.reset();
        self.measurements = PowerAmplifierMeasurements::default();
    }

    pub fn prepare(&mut self, sample_rate: f32, power_pair_cold_bias_volts: f64) {
        self.power_pair.prepare(sample_rate);
        self.power_pair.set_tube(Koren::T6SN7);
        self.power_pair.set_class_ab(power_pair_cold_bias_volts);
        self.power_pair.set_imbalance(0.025);
        self.bias_walk.prepare(sample_rate);
        self.power_supply.prepare(sample_rate);
        self.transient_headroom_controller.prepare(sample_rate);
        self.harmonic_controller.prepare(sample_rate);
        self.measurements = PowerAmplifierMeasurements::default();
    }

    pub fn apply_settings(&mut self, settings: DynamicSettings) {
        self.power_pair.set_drive(db(settings.power_pair_drive_db));
        self.power_pair
            .set_feedback(db(settings.negative_feedback_db));
        self.bias_walk.volts = f64::from(settings.bloom_amount) * 0.45;
        self.power_supply.apply_settings(settings);
        self.harmonic_controller.apply_settings(settings);
    }

    #[inline]
    pub fn process_sample(&mut self, input: f32) -> f32 {
        let power_pair_input = input * self.input_gain;
        let transient_amount = self
            .transient_headroom_controller
            .process_sample(power_pair_input);
        let physical_output = self.power_pair.tick(power_pair_input);
        let plate_draw = self.power_pair.draw();
        self.power_pair
            .set_bias_offset(self.bias_walk.tick(plate_draw));
        let selected_supply_voltage = self
            .power_supply
            .process_sample(power_pair_input, plate_draw);
        self.power_pair.set_supply_scale(selected_supply_voltage);
        let small_signal_gain = db_to_amp(self.power_pair.small_signal_gain().get());
        let output = self.harmonic_controller.process_sample(
            power_pair_input,
            physical_output,
            small_signal_gain,
            transient_amount,
        );
        let (low_voltage, middle_voltage, high_voltage) = self.power_supply.voltages();
        self.measurements = PowerAmplifierMeasurements {
            transient_amount,
            plate_draw,
            low_frequency_reservoir_voltage: low_voltage,
            middle_frequency_reservoir_voltage: middle_voltage,
            high_frequency_reservoir_voltage: high_voltage,
        };
        output
    }

    /// Small-signal gain at the IDLE supply: the calibration point. The
    /// live gain follows the sagging supply; calibrating makeup against
    /// that would step the level whenever a dynamics knob moved mid-note.
    pub fn idle_small_signal_gain_db(&mut self) -> f32 {
        let supply_scale = 10f32.powf(self.power_pair.ceiling().get() / 20.0);
        self.power_pair.set_supply_scale(1.0);
        let gain_db = self.power_pair.small_signal_gain().get();
        self.power_pair.set_supply_scale(supply_scale);
        POWER_PAIR_INPUT_GAIN_DB + gain_db
    }

    pub fn measurements(&self) -> PowerAmplifierMeasurements {
        self.measurements
    }
}
