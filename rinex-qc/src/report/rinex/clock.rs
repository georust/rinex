use itertools::Itertools;
use maud::{html, Markup, Render};
use qc_traits::processing::{Filter, FilterItem, MaskOperand, Preprocessing};
use rinex::prelude::{ClockProfileType, Constellation, Rinex, TimeScale, DOMES, SV};
use std::collections::HashMap;

use crate::report::shared::SamplingReport;
use crate::report::Error;

use crate::plot::{MarkerSymbol, Mode, Plot, Visible};

/// [ClockPage] per [Constellation]
struct ConstellPage {
    /// satellites
    satellites: Vec<SV>,
    offset_plot: Plot,
    drift_plot: Plot,
}

impl ConstellPage {
    fn new(rinex: &Rinex) -> Self {
        let satellites = rinex.sv().collect::<Vec<_>>();

        Self {
            offset_plot: {
                let mut plot =
                    Plot::timedomain_plot("clock_offset", "Clock Offset", "Offset [s]", true);
                for (sv_index, sv) in satellites.iter().enumerate() {
                    let label = sv.to_string();
                    let plot_x = rinex
                        .precise_sv_clock()
                        .filter_map(|(t, svnn, _, _)| if *sv == svnn { Some(t) } else { None })
                        .collect::<Vec<_>>();
                    let plot_y = rinex
                        .precise_sv_clock()
                        .filter_map(
                            |(_, svnn, _, prof)| {
                                if *sv == svnn {
                                    Some(prof.bias)
                                } else {
                                    None
                                }
                            },
                        )
                        .collect::<Vec<_>>();
                    let trace = Plot::timedomain_chart(
                        &label,
                        Mode::Markers,
                        MarkerSymbol::Cross,
                        &plot_x,
                        plot_y,
                        true,
                    )
                    .visible({
                        if sv_index == 0 {
                            Visible::True
                        } else {
                            Visible::LegendOnly
                        }
                    });
                    plot.add_trace(trace);
                }
                plot
            },
            drift_plot: {
                let mut plot =
                    Plot::timedomain_plot("clock_drift", "Clock Drift", "Drift [s/s]", true);
                for sv in &satellites {
                    let label = sv.to_string();
                    let plot_x = rinex
                        .precise_sv_clock()
                        .filter_map(|(t, svnn, _, prof)| {
                            let _ = prof.drift?;
                            if *sv == svnn {
                                Some(t)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();
                    let plot_y = rinex
                        .precise_sv_clock()
                        .filter_map(|(_, svnn, _, prof)| {
                            let drift = prof.drift?;
                            if *sv == svnn {
                                Some(drift)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();
                    let trace = Plot::timedomain_chart(
                        &label,
                        Mode::Markers,
                        MarkerSymbol::Cross,
                        &plot_x,
                        plot_y,
                        true,
                    );
                    plot.add_trace(trace);
                }
                plot
            },
            satellites,
        }
    }
}

impl Render for ConstellPage {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tr {
                        th class="is-info" {
                            button aria-label="Embedded Clocks described in this file." data-balloon-pos="right" {
                                "Satellites"
                            }
                        }
                        td {
                            (self.satellites.iter().join(", "))
                        }
                    }
                    tr {
                        th class="is-info" {
                            button aria-label="Offset to Constellation" data-balloon-pos="right" {
                                "Clock offset"
                            }
                        }
                        td {
                            (self.offset_plot.render())
                        }
                    }
                    tr {
                        th class="is-info" {
                            button aria-label="Drift with respect to Constellation" data-balloon-pos="right" {
                                "Clock drift"
                            }
                        }
                        td {
                            (self.drift_plot.render())
                        }
                    }
                }
            }
        }
    }
}

pub struct ClkReport {
    site: Option<String>,
    domes: Option<DOMES>,
    clk_name: Option<String>,
    sampling: SamplingReport,
    ref_clock: Option<String>,
    codes: Vec<ClockProfileType>,
    igs_clock_name: Option<String>,
    timescale: Option<TimeScale>,
    constellations: HashMap<Constellation, ConstellPage>,
}

impl ClkReport {
    pub fn html_inline_menu_bar(&self) -> Markup {
        html! {
            a id="menu:clk" {
                span class="icon" {
                    i class="fa-solid fa-clock" {}
                }
                "High Precision Clock (RINEX)"
            }
            ul class="menu-list" style="display:none" {
                @for constell in self.constellations.keys().sorted() {
                    li {
                        a id=(&format!("menu:clk:{}", constell)) class="menu:subtab" style="margin-left:29px" {
                            span class="icon" {
                                i class="fa-solid fa-satellite" {};
                            }
                            (constell.to_string())
                        }
                    }
                }
            }
        }
    }
    pub fn new(rnx: &Rinex) -> Result<Self, Error> {
        let clk_header = rnx.header.clock.as_ref().ok_or(Error::MissingClockHeader)?;
        Ok(Self {
            site: clk_header.site.clone(),
            domes: clk_header.domes.clone(),
            codes: clk_header.codes.clone(),
            igs_clock_name: clk_header.igs.clone(),
            clk_name: clk_header.full_name.clone(),
            ref_clock: clk_header.ref_clock.clone(),
            timescale: clk_header.timescale.clone(),
            sampling: SamplingReport::from_rinex(rnx),
            constellations: {
                let mut pages = HashMap::<Constellation, ConstellPage>::new();
                for constellation in rnx.constellation() {
                    let filter = Filter::mask(
                        MaskOperand::Equals,
                        FilterItem::ConstellationItem(vec![constellation]),
                    );
                    let focused = rnx.filter(&filter);
                    pages.insert(constellation, ConstellPage::new(&focused));
                }
                pages
            },
        })
    }
}

impl Render for ClkReport {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tbody {
                        @if let Some(clk_name) = &self.clk_name {
                            tr {
                                th class="is-info" {
                                    "Agency"
                                }
                                td {
                                    (clk_name)
                                }
                            }
                        }
                        @if let Some(site) = &self.site {
                            tr {
                                th class="is-info" {
                                    "Clock Site"
                                }
                                td {
                                    (site)
                                }
                            }
                        }
                        @if let Some(domes) = &self.domes {
                            tr {
                                th class="is-info" {
                                    "DOMES #ID"
                                }
                                td {
                                    (domes.to_string())
                                }
                            }
                        }
                        tr {
                            th class="is-info" {
                                "Clock Profiles"
                            }
                            td {
                                (self.codes.iter().join(", "))
                            }
                        }
                        @if let Some(ref_clk) = &self.ref_clock {
                            tr {
                                th class="is-info" {
                                    "Reference Clock"
                                }
                                td {
                                    (ref_clk)
                                }
                            }
                        }
                        @if let Some(igs_name) = &self.igs_clock_name {
                            tr {
                                th class="is-info" {
                                    "IGS Clock Name"
                                }
                                td {
                                    (igs_name)
                                }
                            }
                        }
                        @if let Some(timescale) = self.timescale {
                            tr {
                                th class="is-info" {
                                    button aria-label="Timescale in which Clock states are expressed.
        In PPP context, this should be identical to your Observation RINEX for optimal precision." data-balloon-pos="right" {
                                        "Timescale"
                                    }
                                }
                                td {
                                    (timescale.to_string())
                                }
                            }
                        }
                    }
                }
            }
            div class="table-container" {
                (self.sampling.render())
            }
            @for constell in self.constellations.keys().sorted() {
                @if let Some(page) = self.constellations.get(&constell) {
                    div class="table-container" {
                        table class="table is-bordered" {
                            tr {
                                th class="is-info" {
                                    (constell.to_string())
                                }
                                td {
                                    (page.render())
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
