use chrono::{NaiveDate, Datelike};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DateSource {
    Filename,
    Content,
    Fallback,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateDetectionResult {
    pub value: NaiveDate,
    pub source: DateSource,
    pub confidence: f32,
}

pub struct DateExtractor {
    month_map_ptbr: HashMap<String, u32>,
}

impl DateExtractor {
    pub fn new() -> Self {
        let mut month_map_ptbr = HashMap::new();
        month_map_ptbr.insert("janeiro".to_string(), 1);
        month_map_ptbr.insert("fevereiro".to_string(), 2);
        month_map_ptbr.insert("março".to_string(), 3);
        month_map_ptbr.insert("marco".to_string(), 3);
        month_map_ptbr.insert("abril".to_string(), 4);
        month_map_ptbr.insert("maio".to_string(), 5);
        month_map_ptbr.insert("junho".to_string(), 6);
        month_map_ptbr.insert("julho".to_string(), 7);
        month_map_ptbr.insert("agosto".to_string(), 8);
        month_map_ptbr.insert("setembro".to_string(), 9);
        month_map_ptbr.insert("outubro".to_string(), 10);
        month_map_ptbr.insert("novembro".to_string(), 11);
        month_map_ptbr.insert("dezembro".to_string(), 12);
        
        month_map_ptbr.insert("jan".to_string(), 1);
        month_map_ptbr.insert("fev".to_string(), 2);
        month_map_ptbr.insert("mar".to_string(), 3);
        month_map_ptbr.insert("abr".to_string(), 4);
        month_map_ptbr.insert("mai".to_string(), 5);
        month_map_ptbr.insert("jun".to_string(), 6);
        month_map_ptbr.insert("jul".to_string(), 7);
        month_map_ptbr.insert("ago".to_string(), 8);
        month_map_ptbr.insert("set".to_string(), 9);
        month_map_ptbr.insert("out".to_string(), 10);
        month_map_ptbr.insert("nov".to_string(), 11);
        month_map_ptbr.insert("dez".to_string(), 12);

        DateExtractor { month_map_ptbr }
    }

    pub fn extract_date_from_filename(&self, filename: &str) -> Option<DateDetectionResult> {
        log::debug!("🔍 Extraindo data do filename: {}", filename);

        let patterns = vec![
            // YYYY-MM-DD (ISO 8601) - mais confiável
            (r"(\d{4})-(\d{2})-(\d{2})", vec![0, 1, 2], 0.95),
            // DD-MM-YYYY
            (r"(\d{2})-(\d{2})-(\d{4})", vec![2, 1, 0], 0.95),
            // DD/MM/YYYY
            (r"(\d{2})/(\d{2})/(\d{4})", vec![2, 1, 0], 0.95),
            // DD_MM_YYYY
            (r"(\d{2})_(\d{2})_(\d{4})", vec![2, 1, 0], 0.95),
            // YYYYMMDD (compacto)
            (r"(\d{4})(\d{2})(\d{2})", vec![0, 1, 2], 0.90),
            // DDMMYYYY (compacto)
            (r"(\d{2})(\d{2})(\d{4})", vec![2, 1, 0], 0.85),
            // DD-MM-YY (ano com 2 dígitos)
            (r"(\d{2})-(\d{2})-(\d{2})", vec![2, 1, 0], 0.80),
            // DD/MM/YY
            (r"(\d{2})/(\d{2})/(\d{2})", vec![2, 1, 0], 0.80),
            // DD_MM_YY
            (r"(\d{2})_(\d{2})_(\d{2})", vec![2, 1, 0], 0.80),
            // DDMMYY (compacto)
            (r"(\d{2})(\d{2})(\d{2})", vec![2, 1, 0], 0.75),
        ];

        for (pattern_str, order, base_confidence) in patterns {
            if let Ok(regex) = Regex::new(pattern_str) {
                if let Some(captures) = regex.captures(filename) {
                    let parts: Vec<&str> = (1..=3)
                        .map(|i| captures.get(i).map(|m| m.as_str()).unwrap_or(""))
                        .collect();

                    if parts.len() == 3 {
                        let year_str = parts[order[0]];
                        let month_str = parts[order[1]];
                        let day_str = parts[order[2]];

                        if let (Ok(mut year), Ok(month), Ok(day)) = (
                            year_str.parse::<i32>(),
                            month_str.parse::<u32>(),
                            day_str.parse::<u32>(),
                        ) {
                            if year < 100 {
                                year += if year > 50 { 1900 } else { 2000 };
                            }

                            if month >= 1 && month <= 12 && day >= 1 && day <= 31 {
                                if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
                                    log::info!(
                                        "✅ Data extraída do filename: {} (confidence: {:.2})",
                                        date.format("%Y-%m-%d"),
                                        base_confidence
                                    );
                                    return Some(DateDetectionResult {
                                        value: date,
                                        source: DateSource::Filename,
                                        confidence: base_confidence,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        log::debug!("⚠️ Nenhuma data encontrada no filename");
        None
    }

    pub fn extract_date_from_content_ptbr(&self, text: &str) -> Option<DateDetectionResult> {
        log::debug!("🔍 Extraindo data do conteúdo (PT-BR)");

        let text_lower = text.to_lowercase();

        let current_year = chrono::Utc::now().year();
        let year_range = (current_year - 10)..=(current_year + 1);

        let patterns = vec![
            (
                r"(\d{1,2})\s+de\s+([a-záàâãéèêíïóôõöúçñ]+)\s+de\s+(\d{4})",
                vec![2, 1, 0],
                0.85,
            ),
            (
                r"(\d{1,2})\s+([a-záàâãéèêíïóôõöúçñ]+)\s+(\d{4})",
                vec![2, 1, 0],
                0.80,
            ),
            (
                r"([a-záàâãéèêíïóôõöúçñ]+)\s+de\s+(\d{4})",
                vec![1, 0, 999],
                0.70,
            ),
            (r"(\d{2})/(\d{2})/(\d{4})", vec![2, 1, 0], 0.75),
            (r"(\d{2})-(\d{2})-(\d{4})", vec![2, 1, 0], 0.75),
        ];

        for (pattern_str, order, base_confidence) in patterns {
            if let Ok(regex) = Regex::new(pattern_str) {
                if let Some(captures) = regex.captures(&text_lower) {
                    let year_idx = order[0];
                    let month_idx = order[1];
                    let day_idx = order[2];

                    let year = if year_idx < 999 {
                        captures.get(year_idx + 1).and_then(|m| m.as_str().parse::<i32>().ok())
                    } else {
                        None
                    };

                    let month_str = if month_idx < 999 {
                        captures.get(month_idx + 1).map(|m| m.as_str())
                    } else {
                        None
                    };

                    let day = if day_idx < 999 {
                        captures.get(day_idx + 1).and_then(|m| m.as_str().parse::<u32>().ok())
                    } else {
                        None
                    };

                    if let Some(year_val) = year {
                        if year_range.contains(&year_val) {
                            let month_val = month_str.and_then(|m_str| {
                                m_str.parse::<u32>().ok().or_else(|| {
                                    self.month_map_ptbr
                                        .get(m_str.trim())
                                        .copied()
                                })
                            });

                            if let Some(month) = month_val {
                                if month >= 1 && month <= 12 {
                                    let day_val = day.unwrap_or(1);
                                    
                                    if day_val >= 1 && day_val <= 31 {
                                        if let Some(date) = NaiveDate::from_ymd_opt(year_val, month, day_val) {
                                            log::info!(
                                                "✅ Data extraída do conteúdo: {} (confidence: {:.2})",
                                                date.format("%Y-%m-%d"),
                                                base_confidence
                                            );
                                            return Some(DateDetectionResult {
                                                value: date,
                                                source: DateSource::Content,
                                                confidence: base_confidence,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        log::debug!("⚠️ Nenhuma data PT-BR encontrada no conteúdo");
        None
    }

    pub fn extract_date_auto(&self, filename: &str, content: &str) -> DateDetectionResult {
        log::info!("🔍 Iniciando extração automática de data");

        if let Some(result) = self.extract_date_from_filename(filename) {
            log::info!("✅ Data extraída do filename com confidence {:.2}", result.confidence);
            return result;
        }

        if let Some(result) = self.extract_date_from_content_ptbr(content) {
            log::info!("✅ Data extraída do conteúdo com confidence {:.2}", result.confidence);
            return result;
        }

        let today = chrono::Utc::now().date_naive();
        log::warn!("⚠️ Usando data atual como fallback: {}", today.format("%Y-%m-%d"));
        
        DateDetectionResult {
            value: today,
            source: DateSource::Fallback,
            confidence: 0.1,
        }
    }
}

pub fn generate_folder_slug(date: &NaiveDate) -> String {
    format!("{}/{:02}", date.year(), date.month())
}

pub fn generate_folder_slug_named(date: &NaiveDate) -> String {
    let month_names = [
        "Janeiro", "Fevereiro", "Março", "Abril", "Maio", "Junho",
        "Julho", "Agosto", "Setembro", "Outubro", "Novembro", "Dezembro",
    ];
    
    let month_name = month_names.get((date.month() - 1) as usize).unwrap_or(&"Desconhecido");
    format!("{}/{}", date.year(), month_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_from_filename_iso() {
        let extractor = DateExtractor::new();
        let result = extractor.extract_date_from_filename("Rastreabilidade_2025-10-04.pdf");
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.value.year(), 2025);
        assert_eq!(result.value.month(), 10);
        assert_eq!(result.value.day(), 4);
        assert!(result.confidence > 0.9);
    }

    #[test]
    fn test_extract_from_filename_br() {
        let extractor = DateExtractor::new();
        let result = extractor.extract_date_from_filename("Rastreabilidade_04-10-2025.pdf");
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.value.year(), 2025);
        assert_eq!(result.value.month(), 10);
        assert_eq!(result.value.day(), 4);
    }

    #[test]
    fn test_extract_from_filename_compact() {
        let extractor = DateExtractor::new();
        let result = extractor.extract_date_from_filename("Carga_20251004.pdf");
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.value.year(), 2025);
        assert_eq!(result.value.month(), 10);
        assert_eq!(result.value.day(), 4);
    }

    #[test]
    fn test_extract_from_filename_short_year() {
        let extractor = DateExtractor::new();
        let result = extractor.extract_date_from_filename("04_10_25.pdf");
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.value.year(), 2025);
        assert_eq!(result.value.month(), 10);
        assert_eq!(result.value.day(), 4);
    }

    #[test]
    fn test_extract_from_content_ptbr_full() {
        let extractor = DateExtractor::new();
        let content = "Emitido em 4 de outubro de 2025";
        let result = extractor.extract_date_from_content_ptbr(content);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.value.year(), 2025);
        assert_eq!(result.value.month(), 10);
        assert_eq!(result.value.day(), 4);
    }

    #[test]
    fn test_extract_from_content_ptbr_month_only() {
        let extractor = DateExtractor::new();
        let content = "Relatório de outubro de 2025";
        let result = extractor.extract_date_from_content_ptbr(content);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.value.year(), 2025);
        assert_eq!(result.value.month(), 10);
        assert_eq!(result.value.day(), 1);
    }

    #[test]
    fn test_folder_slug_numeric() {
        let date = NaiveDate::from_ymd_opt(2025, 10, 4).unwrap();
        let slug = generate_folder_slug(&date);
        assert_eq!(slug, "2025/10");
    }

    #[test]
    fn test_folder_slug_named() {
        let date = NaiveDate::from_ymd_opt(2025, 10, 4).unwrap();
        let slug = generate_folder_slug_named(&date);
        assert_eq!(slug, "2025/Outubro");
    }

    #[test]
    fn test_extract_auto_priority() {
        let extractor = DateExtractor::new();
        let result = extractor.extract_date_auto(
            "Nota_2025-10-04.pdf",
            "Emitido em 15 de novembro de 2024"
        );
        assert_eq!(result.value.year(), 2025);
        assert_eq!(result.value.month(), 10);
        assert_eq!(result.value.day(), 4);
        assert_eq!(result.source, DateSource::Filename);
    }

    #[test]
    fn test_extract_auto_fallback_to_content() {
        let extractor = DateExtractor::new();
        let result = extractor.extract_date_auto(
            "Nota_sem_data.pdf",
            "Emitido em 4 de outubro de 2025"
        );
        assert_eq!(result.value.year(), 2025);
        assert_eq!(result.value.month(), 10);
        assert_eq!(result.value.day(), 4);
        assert_eq!(result.source, DateSource::Content);
    }

    #[test]
    fn test_extract_auto_fallback_to_today() {
        let extractor = DateExtractor::new();
        let result = extractor.extract_date_auto(
            "Nota_sem_data.pdf",
            "Texto sem nenhuma data válida"
        );
        assert_eq!(result.source, DateSource::Fallback);
        assert!(result.confidence < 0.2);
    }
}
