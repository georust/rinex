//! Analysis report
use log::{error, info, warn};

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
    pub fn new(cli: &Cli, ctx: &Context, cfg: QcConfig) -> Self {
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
                let new_content = page.content.render().into_string();
                // replace within boundaries
                let start_pat = format!(
                    "<div id=\"{}\" class=\"container is-main\" style=\"display:none\">",
                    page.html_id
                );
                let end_pat = format!(
                    "<div id=\"end:{}\" style=\"display:none\"></div>",
                    page.html_id
                );
                if let Some(start) = content.find(&start_pat) {
                    if let Some(end) = content.find(&end_pat) {
                        content.replace_range(
                            start..=end + end_pat.len(),
                            &format!("{}{}{}", start_pat, new_content, end_pat),
                        );
                    } else {
                        panic!("report customization failure");
                    }
                } else {
                    panic!("report customization failure");
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
