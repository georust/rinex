pub struct PPPSummary {
    nb_solutions: usize,
    final_error: (f64, f64, f64),
    final_solution_ecef: (f64, f64, f64),
}

impl PPPSummary {
    fn new(solutions: &BTreeMap<Epoch, PVTSolution>) -> Self {
        let last = solutions.last().unwrap_or_else(|| panic!("failed to grab last solution"));
        Self {
            nb_solutions: solutions.len(),
            final_error: 
        }
    }
}

impl RenderHtml for PPPSummary {
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {
            table(class="table is-bordered") {
                tr {
                    th {
                        : "Number of solutions"
                    }
                    td {
                        : self.nb_solutions.to_string()
                    }
                }
                tr {
                    th(class="is-info") {
                        : "Final"
                    }
                    td {
                        table(class="table is-bordered") {
                            tr {
                                th {
                                    : "ECEF"
                                }
                            }
                            tr {
                                th {
                                    : "GEO"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub struct PPPReport {
    map_plot: Plot,
    navi_plot: Plot,
    error_plot: Plot,
    summary: PPPSummary,
    ambiguity_plot: Option<Plot>,
}

impl PPPReport {
    pub fn new(solutions: &BTreeMap<Epoch, PVTSolution>) -> Self {
        Self {
            summary: PPPSummary::new(solutions),
        }
    }
}

impl RenderHtml for PPPReport {
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {

        }
    }
}
