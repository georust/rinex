//! HTML rendition
pub use horrorshow::{box_html, helper::doctype, html, RenderBox};

/// HTML Rendering
pub trait RenderHtml {
    fn georust_logo_url(&self) -> &'static str {
        "https://raw.githubusercontent.com/georust/meta/master/logo/logo.png"
    }
    /// Renders self to plain HTML, generating a whole entity.
    fn to_html(&self) -> String {
        format!(
        "{}",
            html! {
                : doctype::HTML;
                html {
                    head {
                        meta(charset="utf-8");
                        meta(name="viewport", content="width=device-width, initial-scale=1");
                        link(rel="stylesheet", href="https:////cdn.jsdelivr.net/npm/bulma@0.9.4/css/bulma.min.css");
                        link(rel="stylesheet", href="https://www.w3schools.com/w3css/4/w3.css");
                        script(defer="true", src="https://use.fontawesome.com/releases/v5.3.1/js/all.js");
                        link(rel="icon", src=self.georust_logo_url(), style="width:35px;height:35px;");
                    }
                    body {
                        : self.to_inline_html()
                    }
                }
            }
        )
    }
    /// Renders self as an HTML node.
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_>;
}
