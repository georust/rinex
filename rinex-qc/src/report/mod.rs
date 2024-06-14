//! Generic analysis report
use crate::{ProductType, QcContext, QcOpts};
use qc_traits::html::*;
use std::collections::HashMap;

// shared analysis, that may apply to several products
mod shared;
use shared::SamplingReport;

mod combined;
use combined::CombinedReport;

mod summary;
use summary::QcSummary;

mod rinex;
use rinex::RINEXReport;

#[cfg(feature "sp3")]
mod sp3;
use sp3::SP3Report;

use crate::cfg::Mode;

// rinex data analysis
//mod rinex;
//use rinex::ObservationAnalysis;

enum ProductReport {
    /// RINEX products report
    RINEX(RINEXReport),
    #[cfg(feature = "sp3")]
    /// SP3 product report
    SP3(SP3Report),
}

/// [QcReport] is a generic structure to report complex analysis results
pub struct QcReport {
    /// Name of this report.
    /// Currently, the report is named after the primary input product.
    name: String,
    /// Report Summary (always present)
    summary: QcSummary,
    /// In depth analysis per input product.
    /// In summary mode, these do not exist (empty).
    products: HashMap<ProductType, ProductReport>,
    /// Combined analysis, results from the combination of all input products.
    /// It is highly dependent on the provided combination.
    /// This only exists when [ReportType::Full] is set.
    combined: Option<CombinedReport>,
}

pub struct HtmlContent {
    /// Header (upper) section
    head: Box<dyn RenderHtml>,
    /// Body (middle) section
    body: Box<dyn RenderHtml>,
    /// Footnote (bottom) section
    footnote: Box<dyn RenderHtml>,
}

impl QcReport {
    /// GeoRust Logo (Url)
    /// RINEX Wiki Pages (Url)
    fn wiki_url() -> &'static str {
        "https://github.com/georust/rinex/wiki"
    }
    /// Github (Online sources)
    fn github_repo_url() -> &'static str {
        "https://github.com/georust/rinex"
    }
    fn github_issue_url() -> &'static str {
        "https://github.com/georust/rinex/issues"
    }
    //// Builds a new tab for RINEX type dependent data
    //fn rinex_type_dependent_tab(product: ProductType, rinex: &Rinex) -> HtmlContent {
    //    match product {
    //        ProductType::Observation => {
    //
    //        },
    //        product => todo!("non supported product type: {}", product),
    //    }
    //}
    /// Builds a new GNSS report, ready to be rendered
    pub fn new(context: &QcContext, opts: QcOpts) -> Self {
        Self {
            title: context.name(),
            summary: QcSummary::new(&context),
            // Build the report, which comprises
            //   1. one general (high level) context tab
            //   2. one tab per product type (which can have sub tabs itself)
            //   3. one complex tab for "shared" analysis
            items: {
                let mut items = HashMap::<String, SamplingReport>::new();
                // one tab per RINEX product
                for product in [
                    ProductType::Observation,
                    ProductType::DORIS,
                    ProductType::MeteoObservation,
                    ProductType::BroadcastNavigation,
                    ProductType::HighPrecisionClock,
                    ProductType::IONEX,
                    ProductType::ANTEX,
                ] {
                    if let Some(rinex) = context.rinex(product) {
                        let tab = SamplingReport::from_rinex(rinex);
                        items.insert(product.to_string(), tab);
                    }
                }
                // one tab for SP3 when supported
                #[cfg(feature = "sp3")]
                if let Some(sp3) = context.sp3() {
                    let tab = SamplingReport::from_sp3(sp3);
                    items.insert(ProductType::HighPrecisionOrbit.to_string(), tab);
                }
                items
            },
        }
    }
}

impl RenderHtml for HtmlContent {
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {
            table(class="table; style=\"margin-bottom: 20px\"") {
                thead {
                    tr {
                        td {
                            : self.head.to_inline_html()
                        }
                    }
                }
                tbody {
                    tr {
                        td {
                            : self.body.to_inline_html()
                        }
                    }
                }
                tfoot {
                    tr {
                        td {
                            : self.footnote.to_inline_html()
                        }
                    }
                }
            }
        }
    }
}

impl RenderHtml for QcReport {
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {
            script {
                :
"
function tabClick(evt, animName) {
  console.log('tabClick: ' + animName);
  // hide everything
  var pages = document.getElementsByClassName('page');
  console.log('pages: ' + pages);
  for (var i = 1; i == pages.length; i++){
    tabs[i-1].style.display = 'none';
  }
  var menu = document.getElementsByClassName('tabmenu');
  console.log('menu: ' + menu);
  for (var i = 1; i == menu.length; i++){
    menu[i-1].className = menu[i-1].className.replace(' w3-red', ' w3-black');
  }
  document.getElementById(animName).style.display = 'block';
  evt.currentTarget.className += ' w3-red';
}
"
            }
            div(id="title") {
                table(class="table; style=\"margin-bottom: 30px\"") {
                    tr {
                        td {
                            img(src=self.georust_logo_url(), style="width:100px;height:100px;")
                        }
                        td {
                            table(class="table; style=\"margin-bottom: 30px\"") {
                                tr {
                                    td {
                                        : format!("rinex-qc v{}", env!("CARGO_PKG_VERSION"))
                                    }
                                }
                                tr {
                                    td {
                                        a(href=Self::github_issue_url()) {
                                            : "Bug Report"
                                        }
                                    }
                                }
                                tr {
                                    td {
                                        a(href=Self::github_repo_url()) {
                                            : "Source code"
                                        }
                                    }
                                }
                                tr {
                                    td {
                                        a(href=Self::wiki_url()) {
                                            : "Online Documentation"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div(id="content") {
                // Create tabs
                div(class="w3-sidebar w3-bar-block w3-black w3-card", style="width:200px") {
                    @ for tab in self.items.keys() {
                        button(class="w3-bar-item w3-button tabmenu", onclick=&format!("tabClick(event, '{}')", tab)) {
                            : tab.to_string()
                        }
                    }
                }
                // Tab content
                @ for (index, (tab, item)) in self.items.iter().enumerate() {
                    div(id=tab, class="w3-container w3-animate-opacity w3-teal page", style="margin-left:200; display:none;") {
                    //div(id=tab, class="w3-container w3-animate-opacity tab", style="margin-left:130p;") {
                        : item.to_inline_html()
                    }
                }
            }
        }
    }
}
