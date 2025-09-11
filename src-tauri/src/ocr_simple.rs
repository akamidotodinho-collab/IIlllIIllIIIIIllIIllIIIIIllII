// Simplified OCR system that actually works
// This is a practical implementation focused on reliability over advanced features

use std::path::Path;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use regex::Regex;
use tokio::process::Command;
use tempfile::NamedTempFile;
use std::io::Write;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SimpleOCRResult {
    pub extracted_text: String,
    pub document_type: String,
    pub extracted_fields: HashMap<String, String>,
    pub confidence_score: f32,
    pub processing_method: String,
    pub processing_time_ms: u128,
    pub error_message: Option<String>,
}

#[derive(Debug)]
pub enum SimpleOCRError {
    IOError(std::io::Error),
    ProcessingError(String),
    TesseractNotAvailable,
}

impl std::fmt::Display for SimpleOCRError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SimpleOCRError::IOError(e) => write!(f, "IO error: {}", e),
            SimpleOCRError::ProcessingError(e) => write!(f, "Processing error: {}", e),
            SimpleOCRError::TesseractNotAvailable => write!(f, "Tesseract OCR not available"),
        }
    }
}

impl std::error::Error for SimpleOCRError {}

impl From<std::io::Error> for SimpleOCRError {
    fn from(e: std::io::Error) -> Self {
        SimpleOCRError::IOError(e)
    }
}

pub struct SimpleOCRProcessor;

impl SimpleOCRProcessor {
    pub fn new() -> Result<Self, SimpleOCRError> {
        log::info!("🔧 Inicializando Simple OCR Processor");
        Ok(SimpleOCRProcessor)
    }

    // Processar imagem usando tesseract via comando do sistema (mais confiável)
    pub async fn process_image<P: AsRef<Path>>(&self, image_path: P) -> Result<SimpleOCRResult, SimpleOCRError> {
        let start_time = std::time::Instant::now();
        let image_path = image_path.as_ref();
        
        log::info!("🔍 Processando imagem: {:?}", image_path);
        
        // Verificar se tesseract está disponível
        if !self.is_tesseract_available().await {
            return Ok(SimpleOCRResult {
                extracted_text: String::new(),
                document_type: "unknown".to_string(),
                extracted_fields: HashMap::new(),
                confidence_score: 0.0,
                processing_method: "tesseract_unavailable".to_string(),
                processing_time_ms: start_time.elapsed().as_millis(),
                error_message: Some("Tesseract OCR não disponível no sistema".to_string()),
            });
        }

        // Executar tesseract via comando do sistema
        let output = Command::new("tesseract")
            .arg(image_path.to_str().unwrap())
            .arg("stdout")
            .arg("-l")
            .arg("por+eng")
            .output()
            .await?;

        let text = if output.status.success() {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            log::warn!("⚠️ Tesseract error: {}", error_msg);
            
            return Ok(SimpleOCRResult {
                extracted_text: String::new(),
                document_type: "unknown".to_string(),
                extracted_fields: HashMap::new(),
                confidence_score: 0.0,
                processing_method: "tesseract_failed".to_string(),
                processing_time_ms: start_time.elapsed().as_millis(),
                error_message: Some(format!("Tesseract failed: {}", error_msg)),
            });
        };

        // Analisar texto extraído usando heurísticas
        let document_type = self.classify_document_type(&text);
        let extracted_fields = self.extract_fields(&text);
        let confidence_score = self.calculate_confidence(&text, &extracted_fields);

        log::info!("✅ OCR concluído: {} caracteres extraídos", text.len());

        Ok(SimpleOCRResult {
            extracted_text: text,
            document_type,
            extracted_fields,
            confidence_score,
            processing_method: "tesseract_system".to_string(),
            processing_time_ms: start_time.elapsed().as_millis(),
            error_message: None,
        })
    }

    // Processar PDF (texto simples apenas)
    pub async fn process_pdf<P: AsRef<Path>>(&self, pdf_path: P) -> Result<SimpleOCRResult, SimpleOCRError> {
        let start_time = std::time::Instant::now();
        let pdf_path = pdf_path.as_ref();
        
        log::info!("📄 Processando PDF: {:?}", pdf_path);

        // Tentar extrair texto do PDF
        let text = match pdf_extract::extract_text(pdf_path) {
            Ok(text) => text.trim().to_string(),
            Err(e) => {
                log::warn!("⚠️ Erro ao extrair texto do PDF: {:?}", e);
                return Ok(SimpleOCRResult {
                    extracted_text: String::new(),
                    document_type: "unknown".to_string(),
                    extracted_fields: HashMap::new(),
                    confidence_score: 0.0,
                    processing_method: "pdf_extraction_failed".to_string(),
                    processing_time_ms: start_time.elapsed().as_millis(),
                    error_message: Some(format!("PDF text extraction failed: {}", e)),
                });
            }
        };

        if text.is_empty() {
            return Ok(SimpleOCRResult {
                extracted_text: String::new(),
                document_type: "scanned_pdf".to_string(),
                extracted_fields: HashMap::new(),
                confidence_score: 0.0,
                processing_method: "pdf_empty_text".to_string(),
                processing_time_ms: start_time.elapsed().as_millis(),
                error_message: Some("PDF parece ser escaneado - use conversão para imagem para OCR completo".to_string()),
            });
        }

        // Analisar texto extraído
        let document_type = self.classify_document_type(&text);
        let extracted_fields = self.extract_fields(&text);
        let confidence_score = self.calculate_confidence(&text, &extracted_fields);

        log::info!("✅ PDF processado: {} caracteres extraídos", text.len());

        Ok(SimpleOCRResult {
            extracted_text: text,
            document_type,
            extracted_fields,
            confidence_score,
            processing_method: "pdf_text_extraction".to_string(),
            processing_time_ms: start_time.elapsed().as_millis(),
            error_message: None,
        })
    }

    // Verificar se tesseract está disponível
    async fn is_tesseract_available(&self) -> bool {
        match Command::new("tesseract").arg("--version").output().await {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    // HEURÍSTICA: Classificar tipo de documento
    fn classify_document_type(&self, text: &str) -> String {
        let text_lower = text.to_lowercase();
        
        if text_lower.contains("nota fiscal") || text_lower.contains("nf-e") || text_lower.contains("icms") {
            "nota_fiscal".to_string()
        } else if text_lower.contains("contrato") || text_lower.contains("clausula") || text_lower.contains("cláusula") {
            "contrato".to_string()
        } else if text_lower.contains("recibo") || text_lower.contains("comprovante") || text_lower.contains("pagamento") {
            "recibo".to_string()
        } else if text_lower.contains("funcionário") || text_lower.contains("salário") || text_lower.contains("admissão") {
            "documento_rh".to_string()
        } else if text_lower.contains("processo") || text_lower.contains("tribunal") || text_lower.contains("advogado") {
            "documento_juridico".to_string()
        } else if text_lower.contains("relatório") || text_lower.contains("análise") || text_lower.contains("relatorio") {
            "relatorio".to_string()
        } else {
            "documento_generico".to_string()
        }
    }

    // HEURÍSTICA: Extrair campos principais
    fn extract_fields(&self, text: &str) -> HashMap<String, String> {
        let mut fields = HashMap::new();

        // CNPJ
        if let Ok(cnpj_regex) = Regex::new(r"(?:CNPJ[:\s]*)?(\d{2}\.?\d{3}\.?\d{3}/?\d{4}-?\d{2})") {
            if let Some(captures) = cnpj_regex.captures(text) {
                if let Some(cnpj) = captures.get(1) {
                    fields.insert("cnpj".to_string(), cnpj.as_str().to_string());
                }
            }
        }

        // CPF
        if let Ok(cpf_regex) = Regex::new(r"(?:CPF[:\s]*)?(\d{3}\.?\d{3}\.?\d{3}-?\d{2})") {
            if let Some(captures) = cpf_regex.captures(text) {
                if let Some(cpf) = captures.get(1) {
                    fields.insert("cpf".to_string(), cpf.as_str().to_string());
                }
            }
        }

        // Valores monetários
        if let Ok(valor_regex) = Regex::new(r"(?:R\$|total|valor)[:\s]*([0-9,.]+)") {
            if let Some(captures) = valor_regex.captures(text) {
                if let Some(valor) = captures.get(1) {
                    fields.insert("valor_total".to_string(), valor.as_str().to_string());
                }
            }
        }

        // Datas
        if let Ok(data_regex) = Regex::new(r"(\d{2}/\d{2}/\d{4})") {
            if let Some(captures) = data_regex.captures(text) {
                if let Some(data) = captures.get(1) {
                    fields.insert("data".to_string(), data.as_str().to_string());
                }
            }
        }

        fields
    }

    // HEURÍSTICA: Calcular score de confiança
    fn calculate_confidence(&self, text: &str, fields: &HashMap<String, String>) -> f32 {
        let mut score = 0.5; // Base score

        // Aumentar baseado no texto
        if text.len() > 100 {
            score += 0.2;
        }
        if text.len() > 500 {
            score += 0.1;
        }

        // Aumentar baseado nos campos extraídos
        score += (fields.len() as f32) * 0.1;

        // Penalizar se muito pouco texto
        if text.len() < 50 {
            score -= 0.3;
        }

        score.max(0.0).min(1.0)
    }
}

// Funções públicas para uso
pub fn create_simple_ocr_processor() -> Result<SimpleOCRProcessor, SimpleOCRError> {
    SimpleOCRProcessor::new()
}

pub fn get_simple_supported_types() -> Vec<String> {
    vec![
        "Imagens (PNG, JPEG, TIFF) com Tesseract OCR".to_string(),
        "PDFs com texto extraível".to_string(),
        "Nota Fiscal".to_string(),
        "Contrato".to_string(),
        "Recibo".to_string(),
        "Documento RH".to_string(),
        "Documento Jurídico".to_string(),
        "Relatório".to_string(),
    ]
}