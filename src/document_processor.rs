use anyhow::{Result, anyhow};
use std::path::Path;
use std::fs;

pub struct DocumentProcessor;

impl DocumentProcessor {
    pub fn new() -> Self {
        Self
    }

    pub async fn extract_text_from_file<P: AsRef<Path>>(&self, file_path: P) -> Result<String> {
        let path = file_path.as_ref();
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| anyhow!("Unable to determine file extension"))?
            .to_lowercase();

        match extension.as_str() {
            "pdf" => self.extract_pdf_text(path).await,
            "docx" => self.extract_docx_text(path).await,
            "xlsx" => self.extract_xlsx_text(path).await,
            "txt" | "md" | "rst" => {
                // Handle existing text-based formats
                Ok(fs::read_to_string(path)?)
            }
            _ => Err(anyhow!("Unsupported file format: {}", extension))
        }
    }

    async fn extract_pdf_text<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        let bytes = fs::read(path)?;
        let text = pdf_extract::extract_text_from_mem(&bytes)
            .map_err(|e| anyhow!("Failed to extract PDF text: {}", e))?;
        
        // Clean up extracted text
        let cleaned_text = self.clean_extracted_text(&text);
        Ok(cleaned_text)
    }

    async fn extract_docx_text<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        let bytes = fs::read(path)?;
        let docx = docx_rs::read_docx(&bytes)
            .map_err(|e| anyhow!("Failed to read DOCX file: {}", e))?;
        
        // Extract text from all paragraphs
        let mut text = String::new();
        for child in docx.document.children {
            match child {
                docx_rs::DocumentChild::Paragraph(para) => {
                    for run in para.children {
                        if let docx_rs::ParagraphChild::Run(run_content) = run {
                            for run_child in run_content.children {
                                if let docx_rs::RunChild::Text(text_content) = run_child {
                                    text.push_str(&text_content.text);
                                }
                            }
                        }
                    }
                    text.push('\n');
                }
                _ => {} // Skip other types for now
            }
        }
        
        let cleaned_text = self.clean_extracted_text(&text);
        Ok(cleaned_text)
    }

    async fn extract_xlsx_text<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        use calamine::{Reader, Xlsx, open_workbook};
        
        let mut workbook: Xlsx<_> = open_workbook(path)
            .map_err(|e| anyhow!("Failed to open XLSX file: {}", e))?;
        
        let mut text = String::new();
        
        // Process all worksheets
        for sheet_name in workbook.sheet_names() {
            if let Ok(range) = workbook.worksheet_range(&sheet_name) {
                text.push_str(&format!("=== {} ===\n", sheet_name));
                
                for row in range.rows() {
                    let mut row_text = Vec::new();
                    for cell in row {
                        let cell_text = match cell {
                            calamine::Data::String(s) => s.clone(),
                            calamine::Data::Float(f) => f.to_string(),
                            calamine::Data::Int(i) => i.to_string(),
                            calamine::Data::Bool(b) => b.to_string(),
                            calamine::Data::DateTime(dt) => format!("{:?}", dt),
                            calamine::Data::DateTimeIso(dt) => dt.clone(),
                            calamine::Data::DurationIso(dur) => dur.clone(),
                            calamine::Data::Error(e) => format!("ERROR: {:?}", e),
                            calamine::Data::Empty => String::new(),
                        };
                        
                        if !cell_text.trim().is_empty() {
                            row_text.push(cell_text);
                        }
                    }
                    
                    if !row_text.is_empty() {
                        text.push_str(&row_text.join(" | "));
                        text.push('\n');
                    }
                }
                text.push('\n');
            }
        }
        
        let cleaned_text = self.clean_extracted_text(&text);
        Ok(cleaned_text)
    }

    fn clean_extracted_text(&self, text: &str) -> String {
        // Remove excessive whitespace and clean up text
        text.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn is_supported_format<P: AsRef<Path>>(&self, file_path: P) -> bool {
        if let Some(extension) = file_path.as_ref().extension() {
            if let Some(ext_str) = extension.to_str() {
                match ext_str.to_lowercase().as_str() {
                    "pdf" | "docx" | "xlsx" | "txt" | "md" | "rst" => true,
                    _ => false,
                }
            } else {
                false
            }
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_supported_format() {
        let processor = DocumentProcessor::new();
        
        assert!(processor.is_supported_format("test.pdf"));
        assert!(processor.is_supported_format("test.docx"));
        assert!(processor.is_supported_format("test.xlsx"));
        assert!(processor.is_supported_format("test.txt"));
        assert!(processor.is_supported_format("test.md"));
        assert!(processor.is_supported_format("test.rst"));
        
        assert!(!processor.is_supported_format("test.doc"));
        assert!(!processor.is_supported_format("test.xls"));
        assert!(!processor.is_supported_format("test.pptx"));
        assert!(!processor.is_supported_format("test.unknown"));
    }
}