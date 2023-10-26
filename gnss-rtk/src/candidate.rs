/// Position solver candidate
#[derive(Debug, Default, Clone)]
pub struct Candidate {
    /// SV
    sv: SV,
    /// Signal sampling instant
    t: Epoch,
    /// SV state (that we will resolve in the process)
    state: Option<Vector3D>,
    /// SV elevation (that we will resolve in the process)
    elevation: Option<f64>,
    /// SV azimuth (that we will resolve in the process)
    azimuth: f64,
    /// SV group delay
    tgd: Option<f64>,
    /// SV clock state (compared to GNSS timescale)
    clock_state: Vector3D,
    /// SV clock correction 
    clock_corr: Duration,
    /// SV state: position (ECEF)
    state: (Vector3D),
    /// SNR at sampling instant
    snr: Snr,
    /// Signal observations and sampling Epoch
    observations: Vec<(Observable, f64)>,
}

impl Candidate {
    pub(crate) fn interpolated(&self) -> bool {
        self.state.is_some() 
        && self.elevation.is_some() 
        && self.azimuth.is_some()
    }
    pub fn sv(s: &self, sv: SV) -> Self {
        let mut s = self.clone();
        s.sv = sv;
        s
    }
    pub fn clock_bias(s: &self, clock_bias: (f64, f64, f64)) -> Self {
        let mut s = self.clone();
        s.clock_bias = clock_bias;
        s
    }
    pub fn tgd(&self, tgd: f64) -> Self {
        let mut s = self.clone();
        s.tgd = tgd;
        s
    }
    /*
     * Returns L1 pseudo range observation [m]
     */
    fn l1_pseudorange(&self) -> Option<f64> {
        self.observations
            .filter_map(|(observable, value)| {
                if observable.is_pseudorange() && observable.is_l1() {
                    Some(value)
                } else {
                    None
                }
            })
            .reduce(|k, _| k)
    }  
    /* 
     * Compute and return signal transmission Epoch 
     */
    pub(crate) fn transmission_time(&self) -> Result<(Epoch, , Error> {
        let l1_pr = self.l1_pseudorange()
            .ok_or(Error::MissingL1PseudoRange(self.sv))?;
        let seconds_ts = self.t_sampling.to_duration().to_seconds(); 
        let dt_tx = seconds_ts - pr / C;
        let mut e_tx = Epoch::from_duration(dt_tx * Unit::Second, self.t_sampling.time_scale); 

        if self.cfg.sv_clock_bias {
            dt_sat = Ephemeris::sv_clock_corr(sv, clock_bias, t, toe);
            debug!("{:?}: {} dt_sat  {}", t, sv, dt_sat);
            e_tx -= dt_sat;
        }

        if self.cfg.sv_total_group_delay {
            if let Some(tgd) = self.tgd {
                debug!("{:?}: {} tgd      {}", t, sv, tgd);
                e_tx -= tgd;
            }
        }
        
        /* run physical verification */
        let dt = (t - e_tx).to_seconds();
        assert!(dt > 0.0, "resolved t_tx is physically impossible");
        assert!(dt < 1.0, "|t -t_tx| < 1sec is physically impossible");
    }
}

