use crate::analysis::solutions::ppp::{
    QcNavPostPPPSolutions,
    // Summary,
};

use crate::prelude::{html, Markup, QcAnalysis, QcHtmlReporting};

use maud::{PreEscaped, DOCTYPE};

impl QcHtmlReporting for QcNavPostPPPSolutions {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tr {
                        th class="is-info" {
                            "Summary"
                        }
                        // td {
                        //     (self.summary.render())
                        // }
                    }
                }
            }
        }
    }
}

// impl QcHtmlReporting for Summary {
//     fn render(&self) -> Markup {
//         html! {}
//     }
// }
