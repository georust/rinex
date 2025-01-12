use maud::Render;

pub struct QcExtraHtmlPage {
    /// HTML id
    pub html_id: String,
    /// Menu for HTML navigation
    pub menu: Box<dyn Render>,
    /// Extra HTML content
    pub content: Box<dyn Render>,
}
