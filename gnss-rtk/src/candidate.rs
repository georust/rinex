use gnss::prelude::{SNR, SV};
use hifitime::{Epoch, Unit, Duration};
use nyx_space::cosmic::SPEED_OF_LIGHT;
use crate::{RTKConfig, Vector3D, Error};

/// Pseudo Range observations, against
/// given carrier frequency
#[derive(Debug, Default, Clone)]
pub struct PseudoRange {
    pub value: f64,
    pub frequency: f64,
}

/// Position solver candidate
#[derive(Debug, Clone)]
pub struct Candidate<'a> {
    /// SV
    pub sv: SV,
    /// Signal sampling instant
    pub t: Epoch,
    /// SV state (that we will resolve in the process)
    pub(crate) state: Option<Vector3D>,
    /// SV elevation (that we will resolve in the process)
    pub(crate) elevation: Option<f64>,
    /// SV azimuth (that we will resolve in the process)
    pub(crate) azimuth: Option<f64>,
    /// SV group delay
    pub(crate) tgd: Option<f64>,
    /// SV clock state (compared to GNSS timescale)
    pub(crate) clock_state: Vector3D,
    /// SV clock correction 
    pub(crate) clock_corr: Duration,
    /// SNR at sampling instant
    pub(crate) snr: SNR,
    /// Pseudo range observations at "t"
    pub(crate) pseudo_range: &'a Vec<PseudoRange>,
}

impl<'a> Candidate<'a> {
    /// Creates a new candidate, to inject in the solver pool.
    pub fn new(
        sv: SV, 
        t: Epoch, 
        clock_state: Vector3D, 
        clock_corr: Duration,
        snr: SNR,
        pseudo_range: &'a Vec<PseudoRange>,
    ) -> Result<Self, Error> {
        if pseudo_range.len() == 0 {
            Err(Error::NeedsAtLeastOnePseudoRange)
        } else {
           Ok(Self {
            sv,
            t,
            clock_state,
            clock_corr,
            snr,
            pseudo_range: &'a pseudo_range,
            tgd: None,
            state: None,
            elevation: None,
            azimuth: None,
        }) }
    }
    /*
     * Returns true if self is fully interpolated,
     * and therefore ready to resolve
     */
    pub(crate) fn interpolated(&self) -> bool {
        self.state.is_some() 
        && self.elevation.is_some() 
        && self.azimuth.is_some()
    }
    /*
     * Returns one pseudo range observation [m], disregarding its frequency
     * Infaillible, because we don't allow to build Self without at least
     * 1 PR observation
     */
    pub(crate) fn pseudo_range(&self) -> f64 {
        self.pseudo_range
            .iter()
            .map(|pr| pr.value)
            .reduce(|k, _| k)
            .unwrap()
    }  
    /* 
     * Compute and return signal transmission Epoch 
     */
    pub(crate) fn transmission_time(&self, cfg: &RTKConfig) -> Result<Epoch, Error> {
        let (t, ts) = (self.t, self.t.time_scale);
        let seconds_ts = t.to_duration().to_seconds(); 
        let dt_tx = seconds_ts - self.pseudo_range() / SPEED_OF_LIGHT;
        let mut e_tx = Epoch::from_duration(dt_tx * Unit::Second, ts); 

        if cfg.modeling.sv_clock_bias {
            debug!("{:?}: {} dt_sat  {}", t, self.sv, self.clock_corr);
            e_tx -= self.clock_corr;
        }

        if cfg.modeling.sv_total_group_delay {
            if let Some(tgd) = self.tgd {
                let tgd = tgd * Unit::Second;
                debug!("{:?}: {} tgd      {}", t, self.sv, tgd);
                e_tx -= tgd;
            }
        }
        
        /* run physical verification */
        let dt = (t - e_tx).to_seconds();
        assert!(dt > 0.0, "resolved t_tx is physically impossible");
        assert!(dt < 1.0, "|t -t_tx| < 1sec is physically impossible");

        Ok(e_tx)
    }
}
