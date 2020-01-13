use chrono;
use std::fs::File;
use std::io;
use std::io::{BufWriter, Write};

#[derive(Debug, Clone)]
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

impl Report {
    pub fn write_report<'a>(&self, path: &'a str) -> io::Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(&file);
        let report = self.present();
        writer.write_all(report.as_bytes())?;
        Ok(())
    }
    fn present(&self) -> String {
        let today = chrono::offset::Local::now();

        let s = format!(
            "{:?}\n{}\n------------\n{}\n*{}\n\n",
            today, self.title, self.body, self.footnotes
        );
        s
    }
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

    pub fn with_body(&mut self, body: &'a str) -> &mut Self {
        self.body = Some(body.into());
        self
    }

    pub fn with_footnotes(&mut self, notes: &'a str) -> &mut Self {
        self.footnotes = Some(notes.into());
        self
    }

    pub fn build_report(&self) -> Report {
        Report {
            title: self.title.clone().unwrap_or("".into()),
            body: self.body.clone().unwrap_or("".into()),
            footnotes: self.clone().footnotes.unwrap_or("".into()),
        }
    }
}
