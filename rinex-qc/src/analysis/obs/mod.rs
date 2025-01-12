use rinex::{
    hardware::{Antenna, Receiver},
    prelude::{Observable, Rinex},
};

//mod frequency;
pub(crate) mod base;
pub(crate) mod rover;
//mod constellation;

pub(crate) mod clock;

pub use base::QcBasesObservationsAnalysis;
pub use rover::QcRoversObservationAnalysis;
//use constellation::ConstellationPage;
use clock::ClockAnalysis;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Physics {
    SSI,
    Doppler,
    Phase,
    PseudoRange,
}

impl Physics {
    pub fn from_observable(observable: &Observable) -> Self {
        if observable.is_phase_range_observable() {
            Self::Phase
        } else if observable.is_doppler_observable() {
            Self::Doppler
        } else if observable.is_ssi_observable() {
            Self::SSI
        } else {
            Self::PseudoRange
        }
    }
    pub fn plot_title(&self) -> String {
        match self {
            Self::SSI => "SSI".to_string(),
            Self::Phase => "Phase".to_string(),
            Self::Doppler => "Doppler".to_string(),
            Self::PseudoRange => "Pseudo Range".to_string(),
        }
    }
    pub fn y_label(&self) -> String {
        match self {
            Self::SSI => "Power [dB]".to_string(),
            Self::Phase => "Carrier Cycles".to_string(),
            Self::Doppler => "Doppler Shifts".to_string(),
            Self::PseudoRange => "Pseudo Range [m]".to_string(),
        }
    }
}

pub struct QcObservationsAnalysis {
    /// ID of this analysis
    pub id: String,
    /// Possible information about [Receiver]
    pub receiver: Option<Receiver>,
    /// Possible information about receiver [Antenna]
    pub antenna: Option<Antenna>,
    /// Possible receiver clock analysis
    pub clock_analysis: Option<ClockAnalysis>,
}

impl QcObservationsAnalysis {
    pub fn is_null(&self) -> bool {
        self.clock_analysis.is_none() && self.receiver.is_none() && self.antenna.is_none()
    }

    /// Builds new [Report] from this [Rinex]
    pub fn new(id: &str, rinex: &Rinex) -> Self {
        let mut clock_analysis = Option::<ClockAnalysis>::None;

        let record = rinex.record.as_obs().unwrap();

        for (k, v) in record.iter() {
            if let Some(clk) = v.clock {
                if let Some(clock_analysis) = &mut clock_analysis {
                    clock_analysis.new_measurement(k.epoch, &clk);
                } else {
                    let mut analysis = ClockAnalysis::default();
                    analysis.new_measurement(k.epoch, &clk);
                    clock_analysis = Some(analysis);
                }
            }
        }

        Self {
            id: id.to_string(),
            clock_analysis,
            receiver: if let Some(rcvr) = &rinex.header.rcvr {
                Some(rcvr.clone())
            } else {
                None
            },
            antenna: if let Some(ant) = &rinex.header.rcvr_antenna {
                Some(ant.clone())
            } else {
                None
            },
        }
    }
}
