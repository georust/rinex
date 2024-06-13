impl RenderHtml for QcSamplingAnalysis {
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {
            tr {
                th {
                    : "Dominant Sample rate"
                }
                @ if let Some(rate) = self.dominant_sample_rate {
                    td {
                        : format!("{} ({:.3} Hz)", rate, 1.0 / rate.to_unit(Unit::Second))
                    }
                } else {
                    th {
                        : "Undetermined"
                    }
                }
            }
            tr {
                th {
                    : "Gap analysis"
                }

                @ if self.gaps.is_empty() {
                    th {
                        : "No gaps detected"
                    }
                } else {
                    tr {
                        td {
                            @ for (epoch, dt) in &self.gaps {
                                p {
                                    : format!("Start : {}, Duration: {}", epoch, dt)
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
