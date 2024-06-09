//! HTML reports

// Useful re-export for HTML synthesis
pub use horrorshow::{box_html, helper::doctype, html, RenderBox};

/// HTML Report
pub trait HtmlReport {
    /// Renders self to plain HTML, generating a whole entity.
    fn to_html(&self) -> String;
    /// Renders self as an HTML node.
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_>;
}
