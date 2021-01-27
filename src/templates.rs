use askama::Template;

#[derive(Template)]
#[template(path = "paste.html")]
pub struct PasteTemplate {
    pub code: String,
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    /// Indicates if the user searched for a file that doesn't exist
    pub not_found: bool,
    /// The domain name being used for this instance
    pub domain: String,
    /// How many days to keep files for. 0 means forever
    pub delete_after: u32,
}
