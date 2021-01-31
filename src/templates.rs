use askama::Template;

#[derive(Template)]
#[template(path = "paste.html")]
pub(crate) struct PasteTemplate {
    pub(crate) code: String,
}

#[derive(Template)]
#[template(path = "index.html")]
pub(crate) struct IndexTemplate {
    /// Indicates if the user searched for a file that doesn't exist
    pub(crate) not_found: bool,
    /// The domain name being used for this instance
    pub(crate) domain: String,
    /// How many days to keep files for. 0 means forever
    pub(crate) delete_after: u32,
}
