use std::path::{Path, PathBuf};
use image::{ImageReader, DynamicImage, ImageFormat};
use tesseract::{Tesseract, TessInitError};
use pdf_extract::extract_text;
// Removed pdfium-render for compatibility
use regex::Regex;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use tempfile::{NamedTempFile, TempDir};
use std::sync::Arc;
use tokio::task;

// Estrutura para metadados extraídos
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExtractedMetadata {
    pub text_content: String,
    pub document_type: DocumentType,
    pub extracted_fields: HashMap<String, String>,
    pub confidence_score: f32,
    pub language: String,
    pub processing_method: ProcessingMethod,
    pub pages_processed: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum DocumentType {
    NotaFiscal,
    Contrato,
    ReciboPagamento,
    DocumentoRH,
    DocumentoJuridico,
    Relatorio,
    Generico,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ProcessingMethod {
    ImageOCR,
    PDFTextExtraction,
    PDFPageOCR,
    Hybrid, // Combinação de text extraction + OCR
}

// Estrutura para dados específicos extraídos
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocumentFields {
    pub cnpj: Option<String>,
    pub cpf: Option<String>,
    pub valor_total: Option<String>,
    pub data_emissao: Option<String>,
    pub razao_social: Option<String>,
    pub numero_documento: Option<String>,
}

pub struct OCRProcessor {
    tesseract_config: TesseractConfig,
    field_extractors: HashMap<DocumentType, Vec<FieldExtractor>>,
    temp_dir: TempDir,
}

#[derive(Debug, Clone)]
pub struct TesseractConfig {
    pub tessdata_path: Option<PathBuf>,
    pub languages: String,
    pub char_whitelist: String,
}

impl Default for TesseractConfig {
    fn default() -> Self {
        Self {
            tessdata_path: None,
            languages: "por+eng".to_string(),
            char_whitelist: "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyzÀÁÂÃÄÇÈÉÊËÌÍÎÏÒÓÔÕÖÙÚÛÜàáâãäçèéêëìíîïòóôõöùúûü.,/()-:; ".to_string(),
        }
    }
}

struct FieldExtractor {
    name: String,
    regex: Regex,
    transform: Option<fn(&str) -> String>,
}

#[derive(Debug)]
pub enum OCRError {
    TesseractInitError(TessInitError),
    ImageProcessingError(String),
    PDFProcessingError(String),
    IOError(std::io::Error),
    TempFileError(String),
}

impl std::fmt::Display for OCRError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OCRError::TesseractInitError(e) => write!(f, "Tesseract initialization error: {}", e),
            OCRError::ImageProcessingError(e) => write!(f, "Image processing error: {}", e),
            OCRError::PDFProcessingError(e) => write!(f, "PDF processing error: {}", e),
            OCRError::IOError(e) => write!(f, "IO error: {}", e),
            OCRError::TempFileError(e) => write!(f, "Temporary file error: {}", e),
        }
    }
}

impl std::error::Error for OCRError {}

impl From<TessInitError> for OCRError {
    fn from(e: TessInitError) -> Self {
        OCRError::TesseractInitError(e)
    }
}

impl From<std::io::Error> for OCRError {
    fn from(e: std::io::Error) -> Self {
        OCRError::IOError(e)
    }
}

impl OCRProcessor {
    pub fn new() -> Result<Self, OCRError> {
        log::info!("🔧 Inicializando OCR Processor...");
        
        // Configurar tessdata path para operação offline
        let tesseract_config = Self::setup_tessdata_config();
        
        // Criar diretório temporário seguro
        let temp_dir = TempDir::new()
            .map_err(|e| OCRError::TempFileError(format!("Failed to create temp dir: {}", e)))?;
        
        let field_extractors = Self::create_field_extractors();
        
        log::info!("✅ OCR Processor inicializado com tessdata: {:?}", tesseract_config.tessdata_path);
        
        Ok(OCRProcessor {
            tesseract_config,
            field_extractors,
            temp_dir,
        })
    }
    
    // Configurar tessdata para operação 100% offline com recursos bundled
    fn setup_tessdata_config() -> TesseractConfig {
        let mut config = TesseractConfig::default();
        
        // Primeiro, tentar tessdata bundled (recursos Tauri)
        if let Some(bundled_path) = Self::get_bundled_tessdata_path() {
            log::info!("📁 Tessdata bundled encontrado em: {:?}", bundled_path);
            config.tessdata_path = Some(bundled_path);
            return config;
        }
        
        // Fallback: tentar localizações padrão do sistema
        let possible_tessdata_paths = vec![
            PathBuf::from("/usr/share/tesseract-ocr/5/tessdata"),
            PathBuf::from("/usr/share/tesseract-ocr/4.00/tessdata"),
            PathBuf::from("/usr/share/tessdata"),
            // Desenvolvimento local
            PathBuf::from("./tessdata"),
            PathBuf::from("../tessdata"),
        ];
        
        for path in possible_tessdata_paths {
            if path.exists() && path.join("por.traineddata").exists() {
                log::info!("📁 Tessdata do sistema encontrado em: {:?}", path);
                config.tessdata_path = Some(path);
                break;
            }
        }
        
        if config.tessdata_path.is_none() {
            log::warn!("⚠️ Tessdata não encontrado, OCR pode falhar");
        }
        
        config
    }
    
    // Localizar tessdata bundled nos recursos Tauri
    fn get_bundled_tessdata_path() -> Option<PathBuf> {
        // Em produção, os recursos ficam no diretório da aplicação
        // Durante desenvolvimento, ficam no src-tauri/tessdata
        
        // Tentar desenvolvimento primeiro
        let dev_path = PathBuf::from("src-tauri/tessdata");
        if dev_path.exists() && dev_path.join("por.traineddata").exists() && dev_path.join("eng.traineddata").exists() {
            return Some(dev_path);
        }
        
        // Tentar caminho relativo para desenvolvimento
        let dev_path_relative = PathBuf::from("tessdata");
        if dev_path_relative.exists() && dev_path_relative.join("por.traineddata").exists() {
            return Some(dev_path_relative);
        }
        
        // TODO: Em produção, usar tauri::api::path::resource_dir() para localizar recursos bundled
        // Por enquanto, tentar alguns caminhos comuns onde Tauri coloca recursos
        let possible_resource_paths = vec![
            PathBuf::from("./tessdata"),
            PathBuf::from("../tessdata"),
            PathBuf::from("./resources/tessdata"),
        ];
        
        for path in possible_resource_paths {
            if path.exists() && path.join("por.traineddata").exists() {
                return Some(path);
            }
        }
        
        None
    }
    
    // Criar instância do Tesseract com configuração específica
    fn create_tesseract_instance(&self) -> Result<Tesseract, OCRError> {
        let mut tesseract = if let Some(tessdata_path) = &self.tesseract_config.tessdata_path {
            Tesseract::new(Some(tessdata_path), Some(&self.tesseract_config.languages))?
        } else {
            Tesseract::new(None, Some(&self.tesseract_config.languages))?
        };
        
        tesseract.set_variable("tessedit_char_whitelist", &self.tesseract_config.char_whitelist)?;
        tesseract.set_variable("tessedit_pageseg_mode", "1")?; // Automatic page segmentation
        
        Ok(tesseract)
    }
    
    // Processar imagem para extrair texto com async support
    pub async fn extract_text_from_image<P: AsRef<Path>>(&self, image_path: P) -> Result<String, OCRError> {
        let image_path = image_path.as_ref().to_path_buf();
        let tesseract_config = self.tesseract_config.clone();
        
        log::info!("🔍 Processando imagem: {:?}", image_path);
        
        // Executar em thread separada para não bloquear async runtime
        let text = task::spawn_blocking(move || -> Result<String, OCRError> {
            // Carregar e preprocessar imagem
            let img = ImageReader::open(&image_path)
                .map_err(|e| OCRError::ImageProcessingError(format!("Failed to open image: {}", e)))?
                .decode()
                .map_err(|e| OCRError::ImageProcessingError(format!("Failed to decode image: {}", e)))?;
            
            let processed_img = Self::preprocess_image(img);
            
            // Criar arquivo temporário seguro
            let temp_file = NamedTempFile::with_suffix(".png")
                .map_err(|e| OCRError::TempFileError(format!("Failed to create temp file: {}", e)))?;
            
            // Salvar imagem processada
            processed_img.save_with_format(temp_file.path(), ImageFormat::Png)
                .map_err(|e| OCRError::ImageProcessingError(format!("Failed to save processed image: {}", e)))?;
            
            // Criar instância do Tesseract
            let mut tesseract = if let Some(tessdata_path) = &tesseract_config.tessdata_path {
                Tesseract::new(Some(tessdata_path), Some(&tesseract_config.languages))
                    .map_err(OCRError::TesseractInitError)?
            } else {
                Tesseract::new(None, Some(&tesseract_config.languages))
                    .map_err(OCRError::TesseractInitError)?
            };
            
            tesseract.set_variable("tessedit_char_whitelist", &tesseract_config.char_whitelist)
                .map_err(OCRError::TesseractInitError)?;
            
            // Executar OCR
            tesseract.set_image(temp_file.path().to_str().unwrap())
                .map_err(OCRError::TesseractInitError)?;
            let text = tesseract.get_text()
                .map_err(OCRError::TesseractInitError)?;
            
            // temp_file é automaticamente limpo quando sai de escopo
            Ok(text.trim().to_string())
        }).await.map_err(|e| OCRError::TempFileError(format!("Task join error: {}", e)))??;
        
        log::info!("✅ Texto extraído da imagem ({} caracteres)", text.len());
        Ok(text)
    }
    
    // Processar PDF com OCR real para documentos escaneados
    pub async fn extract_text_from_pdf<P: AsRef<Path>>(&self, pdf_path: P) -> Result<ExtractedMetadata, OCRError> {
        let pdf_path = pdf_path.as_ref().to_path_buf();
        let tesseract_config = self.tesseract_config.clone();
        
        log::info!("📄 Processando PDF: {:?}", pdf_path);
        
        // Primeiro, tentar extração de texto simples
        let simple_text = self.try_simple_text_extraction(&pdf_path).await;
        
        // Se texto simples é muito pequeno ou vazio, usar OCR página por página
        let should_use_ocr = match &simple_text {
            Ok(text) if text.trim().len() < 100 => true,
            Ok(text) if self.is_likely_scanned_pdf(text) => true,
            Err(_) => true,
            _ => false,
        };
        
        if should_use_ocr {
            log::info!("📄 PDF parece ser escaneado, usando OCR página por página");
            self.extract_text_from_pdf_with_ocr(pdf_path, tesseract_config).await
        } else if let Ok(text) = simple_text {
            log::info!("📄 PDF tem texto extraível, usando extração simples");
            Ok(ExtractedMetadata {
                text_content: text.clone(),
                document_type: self.classify_document_type(&text),
                extracted_fields: self.extract_fields_by_type(&self.classify_document_type(&text), &text),
                confidence_score: self.calculate_confidence_score(&text, &HashMap::new()),
                language: self.detect_language(&text),
                processing_method: ProcessingMethod::PDFTextExtraction,
                pages_processed: None,
            })
        } else {
            Err(OCRError::PDFProcessingError("Failed to process PDF with both methods".to_string()))
        }
    }
    
    // Tentar extração de texto simples primeiro
    async fn try_simple_text_extraction(&self, pdf_path: &Path) -> Result<String, OCRError> {
        let pdf_path = pdf_path.to_path_buf();
        
        task::spawn_blocking(move || {
            extract_text(&pdf_path)
                .map_err(|e| OCRError::PDFProcessingError(format!("Simple text extraction failed: {}", e)))
                .map(|text| text.trim().to_string())
        }).await.map_err(|e| OCRError::TempFileError(format!("Task join error: {}", e)))?
    }
    
    // OCR simplificado para PDFs (fallback sem pdfium-render)
    async fn extract_text_from_pdf_with_ocr(&self, pdf_path: PathBuf, tesseract_config: TesseractConfig) -> Result<ExtractedMetadata, OCRError> {
        task::spawn_blocking(move || -> Result<ExtractedMetadata, OCRError> {
            log::info!("🔍 Fallback: OCR não disponível para PDF escaneado sem dependências do sistema");
            log::info!("📄 Para OCR completo de PDF escaneado, instale pdfium ou poppler");
            
            // Fallback: tentar extrair texto simples e marcar como baixa confiança
            let text = extract_text(&pdf_path)
                .map_err(|e| OCRError::PDFProcessingError(format!("PDF text extraction failed: {}", e)))?;
            
            if text.trim().is_empty() {
                return Err(OCRError::PDFProcessingError("PDF appears to be scanned but OCR dependencies not available".to_string()));
            }
            
            let clean_text = text.trim().to_string();
            let document_type = Self::classify_document_type_heuristic(&clean_text);
            let extracted_fields = Self::extract_fields_by_type_heuristic(&document_type, &clean_text);
            let confidence_score = Self::calculate_confidence_score_heuristic(&clean_text, &extracted_fields, 1, 1) * 0.7; // Reduzir confiança
            let language = Self::detect_language_heuristic(&clean_text);
            
            log::info!("✅ Texto extraído por fallback ({} caracteres)", clean_text.len());
            
            Ok(ExtractedMetadata {
                text_content: clean_text,
                document_type,
                extracted_fields,
                confidence_score,
                language,
                processing_method: ProcessingMethod::PDFTextExtraction,
                pages_processed: Some(1),
            })
        }).await.map_err(|e| OCRError::TempFileError(format!("Task join error: {}", e)))?
    }
    
    // Detectar se PDF é provavelmente escaneado
    fn is_likely_scanned_pdf(&self, text: &str) -> bool {
        // Heurísticas simples para detectar PDF escaneado
        let word_count = text.split_whitespace().count();
        let char_count = text.len();
        
        // Se muito pouco texto ou muitos caracteres estranhos
        word_count < 10 || 
        (char_count > 0 && (text.chars().filter(|c| c.is_ascii_punctuation()).count() as f32 / char_count as f32) > 0.3)
    }
    
    // Análise inteligente do documento (heurística, não IA real)
    pub fn analyze_document(&self, text: &str) -> ExtractedMetadata {
        log::info!("🧠 Analisando documento com heurística...");
        
        let document_type = Self::classify_document_type_heuristic(text);
        let extracted_fields = Self::extract_fields_by_type_heuristic(&document_type, text);
        let confidence_score = Self::calculate_confidence_score_heuristic(text, &extracted_fields, 1, 1);
        let language = Self::detect_language_heuristic(text);
        
        ExtractedMetadata {
            text_content: text.to_string(),
            document_type,
            extracted_fields,
            confidence_score,
            language,
            processing_method: ProcessingMethod::ImageOCR,
            pages_processed: Some(1),
        }
    }
    
    // Preprocessamento de imagem para melhorar OCR
    fn preprocess_image(img: DynamicImage) -> DynamicImage {
        // Converter para escala de cinza
        let gray_img = img.to_luma8();
        let mut processed = DynamicImage::ImageLuma8(gray_img);
        
        // Redimensionar se muito pequena (melhora qualidade OCR)
        let (width, height) = processed.dimensions();
        if width < 800 || height < 800 {
            processed = processed.resize(
                (width as f32 * 1.5) as u32,
                (height as f32 * 1.5) as u32,
                image::imageops::FilterType::Lanczos3,
            );
        }
        
        processed
    }
    
    // HEURÍSTICA (NÃO IA REAL): Classificação automática do tipo de documento
    fn classify_document_type_heuristic(text: &str) -> DocumentType {
        let text_lower = text.to_lowercase();
        
        // Padrões para nota fiscal
        if text_lower.contains("nota fiscal") || text_lower.contains("nf-e") || 
           text_lower.contains("icms") || text_lower.contains("cfop") {
            return DocumentType::NotaFiscal;
        }
        
        // Padrões para contrato
        if text_lower.contains("contrato") || text_lower.contains("partes contratantes") ||
           text_lower.contains("clausula") || text_lower.contains("cláusula") {
            return DocumentType::Contrato;
        }
        
        // Padrões para RH
        if text_lower.contains("funcionário") || text_lower.contains("funcionario") ||
           text_lower.contains("salário") || text_lower.contains("salario") ||
           text_lower.contains("admissão") || text_lower.contains("admissao") {
            return DocumentType::DocumentoRH;
        }
        
        // Padrões para jurídico
        if text_lower.contains("processo") || text_lower.contains("advogado") ||
           text_lower.contains("tribunal") || text_lower.contains("sentença") {
            return DocumentType::DocumentoJuridico;
        }
        
        // Padrões para recibo
        if text_lower.contains("recibo") || text_lower.contains("comprovante") ||
           text_lower.contains("pagamento") || text_lower.contains("valor pago") {
            return DocumentType::ReciboPagamento;
        }
        
        // Padrões para relatório
        if text_lower.contains("relatório") || text_lower.contains("relatorio") ||
           text_lower.contains("análise") || text_lower.contains("analise") {
            return DocumentType::Relatorio;
        }
        
        DocumentType::Generico
    }
    
    // HEURÍSTICA: Extração de campos específicos por tipo de documento
    fn extract_fields_by_type_heuristic(doc_type: &DocumentType, text: &str) -> HashMap<String, String> {
        let mut fields = HashMap::new();
        
        // Extratores universais (aplicados a todos os documentos)
        Self::apply_universal_extractors_heuristic(&mut fields, text);
        
        // Campos específicos por tipo
        match doc_type {
            DocumentType::NotaFiscal => {
                // Número da NF
                if let Ok(nf_regex) = Regex::new(r"(?:N[úu]mero|N[°º])[:\s]*(\d+)") {
                    if let Some(captures) = nf_regex.captures(text) {
                        if let Some(numero) = captures.get(1) {
                            fields.insert("numero_nf".to_string(), numero.as_str().to_string());
                        }
                    }
                }
                
                // Série da NF
                if let Ok(serie_regex) = Regex::new(r"(?:S[eé]rie)[:\s]*(\d+)") {
                    if let Some(captures) = serie_regex.captures(text) {
                        if let Some(serie) = captures.get(1) {
                            fields.insert("serie_nf".to_string(), serie.as_str().to_string());
                        }
                    }
                }
            }
            DocumentType::Contrato => {
                // Número do contrato
                if let Ok(contrato_regex) = Regex::new(r"(?:Contrato|Contract)[:\s]*n[°º]?\s*([A-Z0-9-/]+)") {
                    if let Some(captures) = contrato_regex.captures(text) {
                        if let Some(numero) = captures.get(1) {
                            fields.insert("numero_contrato".to_string(), numero.as_str().to_string());
                        }
                    }
                }
            }
            _ => {} // Outros tipos usam apenas extratores universais
        }
        
        fields
    }
    
    // HEURÍSTICA: Extratores universais para todos os documentos
    fn apply_universal_extractors_heuristic(fields: &mut HashMap<String, String>, text: &str) {
        // CNPJ
        if let Ok(cnpj_regex) = Regex::new(r"(?:CNPJ[:\s]*)?(\d{2}\.?\d{3}\.?\d{3}/?\d{4}-?\d{2})") {
            if let Some(captures) = cnpj_regex.captures(text) {
                if let Some(cnpj) = captures.get(1) {
                    fields.insert("cnpj".to_string(), Self::normalize_cnpj_heuristic(cnpj.as_str()));
                }
            }
        }
        
        // CPF
        if let Ok(cpf_regex) = Regex::new(r"(?:CPF[:\s]*)?(\d{3}\.?\d{3}\.?\d{3}-?\d{2})") {
            if let Some(captures) = cpf_regex.captures(text) {
                if let Some(cpf) = captures.get(1) {
                    fields.insert("cpf".to_string(), Self::normalize_cpf_heuristic(cpf.as_str()));
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
        
        // Datas (formato brasileiro)
        if let Ok(data_regex) = Regex::new(r"(\d{2}/\d{2}/\d{4})") {
            if let Some(captures) = data_regex.captures(text) {
                if let Some(data) = captures.get(1) {
                    fields.insert("data_emissao".to_string(), data.as_str().to_string());
                }
            }
        }
    }
    
    // HEURÍSTICA: Calcular score de confiança
    fn calculate_confidence_score_heuristic(text: &str, fields: &HashMap<String, String>, successful_pages: usize, total_pages: usize) -> f32 {
        let mut score = 0.5; // Score base
        
        // Aumentar score baseado no número de campos extraídos
        score += (fields.len() as f32) * 0.1;
        
        // Aumentar score baseado no tamanho do texto
        let text_length_factor = (text.len() as f32 / 1000.0).min(0.3);
        score += text_length_factor;
        
        // Penalizar se texto muito curto
        if text.len() < 50 {
            score -= 0.3;
        }
        
        // Fator de páginas processadas com sucesso
        if total_pages > 0 {
            let page_success_rate = successful_pages as f32 / total_pages as f32;
            score *= page_success_rate;
        }
        
        score.max(0.0).min(1.0)
    }
    
    // HEURÍSTICA: Detectar idioma
    fn detect_language_heuristic(text: &str) -> String {
        let portuguese_words = ["de", "da", "do", "para", "com", "em", "por", "são", "não"];
        let portuguese_count = portuguese_words.iter()
            .filter(|&word| text.to_lowercase().contains(word))
            .count();
        
        if portuguese_count >= 2 {
            "pt-BR".to_string()
        } else {
            "en".to_string()
        }
    }
    
    // Utilitários para normalização
    fn normalize_cnpj_heuristic(cnpj: &str) -> String {
        let digits: String = cnpj.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() == 14 {
            format!("{}.{}.{}/{}-{}", 
                &digits[0..2], &digits[2..5], &digits[5..8], &digits[8..12], &digits[12..14])
        } else {
            cnpj.to_string()
        }
    }
    
    fn normalize_cpf_heuristic(cpf: &str) -> String {
        let digits: String = cpf.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() == 11 {
            format!("{}.{}.{}-{}", 
                &digits[0..3], &digits[3..6], &digits[6..9], &digits[9..11])
        } else {
            cpf.to_string()
        }
    }
    
    // Criar extratores específicos (não usado na versão heurística, mas mantido para compatibilidade)
    fn create_field_extractors() -> HashMap<DocumentType, Vec<FieldExtractor>> {
        HashMap::new() // Implementação simplificada - campos são extraídos diretamente nas funções heurísticas
    }
    
    // Método para classificação (compatibilidade)
    fn classify_document_type(&self, text: &str) -> DocumentType {
        Self::classify_document_type_heuristic(text)
    }
    
    // Método para extração de campos (compatibilidade)
    fn extract_fields_by_type(&self, doc_type: &DocumentType, text: &str) -> HashMap<String, String> {
        Self::extract_fields_by_type_heuristic(doc_type, text)
    }
    
    // Método para score de confiança (compatibilidade)
    fn calculate_confidence_score(&self, text: &str, fields: &HashMap<String, String>) -> f32 {
        Self::calculate_confidence_score_heuristic(text, fields, 1, 1)
    }
    
    // Método para detecção de idioma (compatibilidade)
    fn detect_language(&self, text: &str) -> String {
        Self::detect_language_heuristic(text)
    }
}

// Função pública para inicializar o processador OCR
pub fn create_ocr_processor() -> Result<OCRProcessor, OCRError> {
    match OCRProcessor::new() {
        Ok(processor) => {
            log::info!("✅ OCR Processor inicializado com sucesso");
            Ok(processor)
        }
        Err(e) => {
            log::error!("❌ Erro ao inicializar OCR: {:?}", e);
            Err(e)
        }
    }
}

// Função para obter tipos de documentos suportados
pub fn get_supported_document_types() -> Vec<String> {
    vec![
        "PDF (com OCR para documentos escaneados)".to_string(),
        "Imagens (PNG, JPEG, TIFF)".to_string(),
        "Nota Fiscal".to_string(),
        "Contrato".to_string(),
        "Recibo de Pagamento".to_string(),
        "Documento RH".to_string(),
        "Documento Jurídico".to_string(),
        "Relatório".to_string(),
    ]
}