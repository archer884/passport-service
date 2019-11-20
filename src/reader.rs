use leptess::{leptonica, tesseract::TessApi};
use std::path::Path;

pub struct Reader {
    api: TessApi,
}

impl Reader {
    pub fn new() -> Self {
        Self {
            api: TessApi::new(Some("./resource"), "mrz").unwrap(),
        }
    }

    pub fn read(&mut self, path: impl AsRef<Path>) -> Option<String> {
        let pixels = leptonica::pix_read(path.as_ref())?;
        self.api.set_image(&pixels);

        // We're "expecting" this error because the language
        // we're using should not support non-ASCII characters.
        Some(self.api.get_utf8_text().expect("Invalid UTF-8"))
    }
}
