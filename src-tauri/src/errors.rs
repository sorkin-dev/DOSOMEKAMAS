#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("OCR error: {0}")]
    Ocr(String),

    #[error("DofusDB API error: {code} \u{2014} {message}")]
    DofusDbApi { code: u32, message: String },

    #[error("Calculation error: {0}")]
    Calculation(String),

    #[error("Item parsing error: {0}")]
    Parsing(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

