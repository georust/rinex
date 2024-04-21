//! RINEX QC reports in HTML
use horrorshow::RenderBox;

pub trait HtmlReport {
    /// Renders self to plain HTML.
    /// Generates a whole HTML entity.
    fn to_html(&self) -> String;
    /// Renders self as an HTML node to embed within external HTML.
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_>;
}
