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
                        meta(charset="UTF-8");
                        meta(http-equiv="X-UA-Compatible", content="IE=edge");
                        meta(name="viewport", content="width=device-width, initial-scale=1");
                        link(rel="icon", type="image/x-icon", href="https://raw.githubusercontent.com/georust/meta/master/logo/logo.png");
                        script(defer="true", src="https://use.fontawesome.com/releases/v5.3.1/js/all.js");
                        link(rel="stylesheet", href="https://cdn.jsdelivr.net/npm/bulma@1.0.0/css/bulma.min.css");
                        link(rel="stylesheet", href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.2/css/all.min.css");
                    }
                    title {
                        : "RINEX QC"
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
