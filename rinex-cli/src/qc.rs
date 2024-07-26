//! Analysis report
use log::{error, info, warn};

use tl::{
    parse as parse_html, 
    ParserOptions as HtmlParserOptions, 
    Node as HtmlNode,
    VDom as HtmlVDom,
};

use std::{
    fs::{read_to_string, File},
    io::Write,
};

use crate::cli::{Cli, Context};

use rinex_qc::prelude::{QcConfig, QcExtraPage, QcReport, Render};

/// Quality check report
pub enum Report {
    /// New report generation/synthesis
    Pending(QcReport),
    /// Report iteration (preserved past run)
    Iteration(String),
}

impl Report {
    /// Create a new report
    pub fn new(cli: &Cli, ctx: &Context) -> Self {
        let cfg = QcConfig::default();
        info!("using default QC configuration: {:#?}", cfg);

        let report_path = ctx.workspace.root.join("index.html");
        let hash_path = ctx.workspace.root.join(".hash");
        if !cli.force_report_synthesis() && report_path.exists() && hash_path.exists() {
            // determine whether we can preserve previous report or not
            if let Ok(content) = read_to_string(hash_path) {
                if let Ok(prev_hash) = content.parse::<u64>() {
                    if prev_hash == cli.hash() {
                        if let Ok(content) = read_to_string(report_path) {
                            info!("preserving previous report");
                            Self::Iteration(content.to_string())
                        } else {
                            error!("failed to parse previous report");
                            warn!("forcing new report synthesis");
                            Self::Pending(QcReport::new(&ctx.data, cfg))
                        }
                    } else {
                        info!("generating new report");
                        Self::Pending(QcReport::new(&ctx.data, cfg))
                    }
                } else {
                    error!("failed to parse hashed value");
                    warn!("forcing new report synthesis");
                    Self::Pending(QcReport::new(&ctx.data, cfg))
                }
            } else {
                // new report
                info!("report synthesis");
                Self::Pending(QcReport::new(&ctx.data, cfg))
            }
        } else {
            // new report
            info!("report synthesis");
            Self::Pending(QcReport::new(&ctx.data, cfg))
        }
    }
    /// Customize report with extra page
    pub fn customize(&mut self, page: QcExtraPage) {
        match self {
            Self::Pending(report) => report.add_chapter(page),
            Self::Iteration(ref mut content) => {
                // Render new html content
                let new_content = page
                    .content
                    .render()
                    .into_string();
                // Parse previous content
                let opts = HtmlParserOptions::new()
                    .track_ids()
                    .track_classes();
                match parse_html(&content.clone(), opts) {
                    Ok(mut vdom) => {
                        // Preserve base, rewrite custom chapter
                        if let Ok(new_vdom) = parse_html(&new_content, Default::default()) {
                            if let Some(new_node) = new_vdom.nodes().get(0) {
                                if let Some(new_tag) = new_node.as_tag() {
                                    for node in vdom.nodes_mut() {
                                        if let Some(tag) = node.as_tag() {
                                            if let Some(tag_id) = tag.attributes().id() {
                                                if *tag_id == *page.html_id {
                                                    // overwrite
                                                    *node = HtmlNode::Tag(new_tag.clone());
                                                }
                                            }
                                        }
                                    }
                                    // replace old content
                                    *content = vdom.outer_html();
                                } else {
                                    error!("{} new chapter renders unexpected html", page.html_id);
                                }
                            } else {
                                error!("{} new chapter renders empty html", page.html_id);
                            }
                        } else {
                            error!("{} new chapter renders invalid html", page.html_id);
                        }
                    },
                    Err(e) => {
                        panic!("previous report is not valid html: {}", e);
                    },
                }
            },
        }
    }
    /// Render as html
    fn render(&self) -> String {
        match self {
            Self::Iteration(report) => report.to_string(),
            Self::Pending(report) => report.render().into_string(),
        }
    }
    /// Generate (dump) report
    pub fn generate(&self, cli: &Cli, ctx: &Context) -> std::io::Result<()> {
        let html = self.render();
        let path = ctx.workspace.root.join("index.html");

        let mut fd = File::create(&path)?;
        write!(fd, "{}", html)?;
        info!("{} report generated", path.display());

        // store past settings
        if let Ok(mut fd) = File::create(ctx.workspace.root.join(".hash")) {
            let _ = write!(fd, "{}", cli.hash());
        }

        Ok(())
    }
}
