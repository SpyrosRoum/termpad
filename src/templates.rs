use askama::Template;

#[derive(Template)]
#[template(path = "paste.html")]
pub struct PasteTemplate {
    pub code: String,
}
