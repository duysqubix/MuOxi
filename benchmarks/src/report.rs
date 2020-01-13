pub struct ReportBuilder {
    title: Option<String>,
    body: Option<String>,
    footnotes: Option<String>,
}

pub struct Report {
    title: String,
    body: String,
    footnotes: String,
}

impl<'a> ReportBuilder {
    pub fn new() -> Self {
        Self {
            title: None,
            body: None,
            footnotes: None,
        }
    }

    pub fn with_title(&mut self, title: &'a str) -> &mut Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_body(&mut self, body: String) -> &mut Self {
        self.body = Some(body.clone());
        self
    }

    pub fn with_footnotes(&mut self, notes: &'a str) -> &mut Self {
        self.footnotes = Some(notes.into());
        self
    }
}
