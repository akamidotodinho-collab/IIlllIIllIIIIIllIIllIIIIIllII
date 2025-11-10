use chrono::{NaiveDate, Datelike, Duration};
use regex::Regex;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DateSearchQuery {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub query_type: DateQueryType,
}

#[derive(Debug, Clone)]
pub enum DateQueryType {
    ExactDate,       // "04/10/2025"
    Month,           // "outubro 2025" ou "outubro"
    DayAndMonth,     // "4 de outubro"
    TextualDate,     // "dia 4 de outubro de 2025"
}

pub struct DateSearchParser {
    month_map_ptbr: HashMap<String, u32>,
}

impl DateSearchParser {
    pub fn new() -> Self {
        let mut month_map_ptbr = HashMap::new();
        
        // Meses completos
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
        
        // Abreviações
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

        DateSearchParser { month_map_ptbr }
    }

    /// Detecta se a query é uma busca PURAMENTE por data (sem texto adicional)
    /// Retorna None se a query contém palavras além de componentes de data
    pub fn parse(&self, query: &str) -> Option<DateSearchQuery> {
        let query_lower = query.to_lowercase().trim().to_string();
        
        log::debug!("🔍 Analisando query de data: '{}'", query_lower);

        // IMPORTANTE: Só tratar como date-only se a query for PURAMENTE data
        // Queries mistas como "rastreabilidade outubro" devem ir para FTS5
        
        // 1. Data completa numérica: "04/10/2025", "04-10-2025", "2025-10-04"
        if let Some(result) = self.parse_numeric_date(&query_lower) {
            // Verificar se a query tem APENAS a data (sem palavras extras)
            if self.is_pure_date_query(&query_lower, &["dia", "de", "em", "do", "da"]) {
                log::info!("✅ Detectada data numérica pura: {:?}", result);
                return Some(result);
            }
        }

        // 2. Texto natural: "dia 4 de outubro de 2025", "4 de outubro"
        if let Some(result) = self.parse_textual_date(&query_lower) {
            // Verificar se não tem palavras além de "dia", "de", números e mês
            if self.is_pure_date_query(&query_lower, &["dia", "de", "em", "do", "da"]) {
                log::info!("✅ Detectada data textual pura: {:?}", result);
                return Some(result);
            }
        }

        // 3. Mês e ano: "outubro 2025", "outubro de 2025"
        if let Some(result) = self.parse_month_year(&query_lower) {
            // Verificar se tem APENAS mês e ano
            if self.is_pure_date_query(&query_lower, &["de", "em", "do", "da"]) {
                log::info!("✅ Detectado mês/ano puro: {:?}", result);
                return Some(result);
            }
        }

        // 4. Apenas mês: "outubro", "maio"
        // APENAS se a query for SOMENTE o nome do mês (sem outras palavras)
        if let Some(result) = self.parse_month_only(&query_lower) {
            // Verificar se a query tem APENAS o nome do mês
            let tokens: Vec<&str> = query_lower.split_whitespace().collect();
            if tokens.len() == 1 {
                log::info!("✅ Detectado mês puro: {:?}", result);
                return Some(result);
            }
        }

        log::debug!("⚠️ Query não é busca PURA por data (pode ter texto adicional)");
        None
    }

    /// Verifica se a query é composta APENAS por componentes de data
    /// (números, meses, conectores como "de", "dia", etc)
    fn is_pure_date_query(&self, query: &str, allowed_connectors: &[&str]) -> bool {
        // Remover números e meses conhecidos
        let mut clean_query = query.to_string();
        
        // Remover datas numéricas
        clean_query = Regex::new(r"\d{1,4}").unwrap().replace_all(&clean_query, "").to_string();
        
        // Remover meses
        for month_name in self.month_map_ptbr.keys() {
            let pattern = format!(r"\b{}\b", regex::escape(month_name));
            if let Ok(regex) = Regex::new(&pattern) {
                clean_query = regex.replace_all(&clean_query, "").to_string();
            }
        }
        
        // Remover conectores permitidos
        for connector in allowed_connectors {
            let pattern = format!(r"\b{}\b", regex::escape(connector));
            if let Ok(regex) = Regex::new(&pattern) {
                clean_query = regex.replace_all(&clean_query, "").to_string();
            }
        }
        
        // Remover separadores (/, -, _)
        clean_query = Regex::new(r"[/\-_\s]+").unwrap().replace_all(&clean_query, " ").to_string();
        
        // Se sobrou alguma palavra, não é pure date query
        let remaining = clean_query.trim();
        let is_pure = remaining.is_empty();
        
        if !is_pure {
            log::debug!("⚠️ Query tem palavras além de data: '{}'", remaining);
        }
        
        is_pure
    }

    /// Parse: "04/10/2025", "04-10-2025", "2025-10-04", etc
    fn parse_numeric_date(&self, query: &str) -> Option<DateSearchQuery> {
        let patterns = vec![
            // ISO 8601: YYYY-MM-DD
            (r"(\d{4})-(\d{2})-(\d{2})", vec![0, 1, 2]),
            // BR: DD/MM/YYYY
            (r"(\d{2})/(\d{2})/(\d{4})", vec![2, 1, 0]),
            // BR: DD-MM-YYYY
            (r"(\d{2})-(\d{2})-(\d{4})", vec![2, 1, 0]),
            // Compacto: YYYYMMDD
            (r"(\d{4})(\d{2})(\d{2})", vec![0, 1, 2]),
            // Compacto BR: DDMMYYYY
            (r"(\d{2})(\d{2})(\d{4})", vec![2, 1, 0]),
        ];

        for (pattern_str, order) in patterns {
            if let Ok(regex) = Regex::new(pattern_str) {
                if let Some(captures) = regex.captures(query) {
                    let parts: Vec<&str> = (1..=3)
                        .map(|i| captures.get(i).map(|m| m.as_str()).unwrap_or(""))
                        .collect();

                    if parts.len() == 3 {
                        let year_str = parts[order[0]];
                        let month_str = parts[order[1]];
                        let day_str = parts[order[2]];

                        if let (Ok(year), Ok(month), Ok(day)) = (
                            year_str.parse::<i32>(),
                            month_str.parse::<u32>(),
                            day_str.parse::<u32>(),
                        ) {
                            if month >= 1 && month <= 12 && day >= 1 && day <= 31 {
                                if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
                                    return Some(DateSearchQuery {
                                        start_date: date,
                                        end_date: date,
                                        query_type: DateQueryType::ExactDate,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// Parse: "dia 4 de outubro de 2025", "4 de outubro", "carga dia 4 de outubro"
    fn parse_textual_date(&self, query: &str) -> Option<DateSearchQuery> {
        // Regex para: "dia? <numero> de <mes> de? <ano>?"
        // Exemplos: "dia 4 de outubro de 2025", "4 de outubro", "4 de out"
        let pattern = r"(?:dia\s+)?(\d{1,2})\s+de\s+([a-zç]+)(?:\s+de\s+)?(\d{4})?";
        
        if let Ok(regex) = Regex::new(pattern) {
            if let Some(captures) = regex.captures(query) {
                let day_str = captures.get(1).map(|m| m.as_str()).unwrap_or("");
                let month_str = captures.get(2).map(|m| m.as_str()).unwrap_or("");
                let year_str = captures.get(3).map(|m| m.as_str());

                if let Ok(day) = day_str.parse::<u32>() {
                    if let Some(&month) = self.month_map_ptbr.get(month_str) {
                        let current_year = chrono::Local::now().year();
                        let year = if let Some(y_str) = year_str {
                            y_str.parse::<i32>().unwrap_or(current_year)
                        } else {
                            current_year
                        };

                        if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
                            return Some(DateSearchQuery {
                                start_date: date,
                                end_date: date,
                                query_type: DateQueryType::TextualDate,
                            });
                        }
                    }
                }
            }
        }

        None
    }

    /// Parse: "outubro 2025", "outubro de 2025"
    fn parse_month_year(&self, query: &str) -> Option<DateSearchQuery> {
        // Regex para: "<mes> de? <ano>"
        // Exemplos: "outubro 2025", "out de 2025"
        let pattern = r"([a-zç]+)(?:\s+de\s+|\s+)(\d{4})";
        
        if let Ok(regex) = Regex::new(pattern) {
            if let Some(captures) = regex.captures(query) {
                let month_str = captures.get(1).map(|m| m.as_str()).unwrap_or("");
                let year_str = captures.get(2).map(|m| m.as_str()).unwrap_or("");

                if let Some(&month) = self.month_map_ptbr.get(month_str) {
                    if let Ok(year) = year_str.parse::<i32>() {
                        // Retornar o mês inteiro (primeiro ao último dia)
                        if let Some(start_date) = NaiveDate::from_ymd_opt(year, month, 1) {
                            // Calcular último dia do mês
                            let next_month = if month == 12 {
                                NaiveDate::from_ymd_opt(year + 1, 1, 1)
                            } else {
                                NaiveDate::from_ymd_opt(year, month + 1, 1)
                            };

                            if let Some(next_month_date) = next_month {
                                let end_date = next_month_date - Duration::days(1);
                                
                                return Some(DateSearchQuery {
                                    start_date,
                                    end_date,
                                    query_type: DateQueryType::Month,
                                });
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// Parse: "outubro", "documentos de outubro"
    /// IMPORTANTE: Usa word boundaries para evitar falsos positivos
    /// (ex: "setor" não deve detectar "set")
    fn parse_month_only(&self, query: &str) -> Option<DateSearchQuery> {
        // Tentar meses COMPLETOS primeiro (mais específicos)
        let full_month_names = vec![
            "janeiro", "fevereiro", "março", "marco", "abril", "maio", "junho",
            "julho", "agosto", "setembro", "outubro", "novembro", "dezembro"
        ];

        for month_name in full_month_names {
            // Usar word boundary regex para match exato
            let pattern = format!(r"\b{}\b", regex::escape(month_name));
            if let Ok(regex) = Regex::new(&pattern) {
                if regex.is_match(query) {
                    if let Some(&month_num) = self.month_map_ptbr.get(month_name) {
                        let current_year = chrono::Local::now().year();
                        
                        if let Some(start_date) = NaiveDate::from_ymd_opt(current_year, month_num, 1) {
                            let next_month = if month_num == 12 {
                                NaiveDate::from_ymd_opt(current_year + 1, 1, 1)
                            } else {
                                NaiveDate::from_ymd_opt(current_year, month_num + 1, 1)
                            };

                            if let Some(next_month_date) = next_month {
                                let end_date = next_month_date - Duration::days(1);
                                
                                return Some(DateSearchQuery {
                                    start_date,
                                    end_date,
                                    query_type: DateQueryType::Month,
                                });
                            }
                        }
                    }
                }
            }
        }

        // NÃO USAR ABREVIAÇÕES AQUI - muito propenso a falsos positivos
        // Abreviações devem vir em contexto de data completa (ex: "4 de out")
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numeric_date() {
        let parser = DateSearchParser::new();
        
        // DD/MM/YYYY
        let result = parser.parse("04/10/2025").unwrap();
        assert_eq!(result.start_date.format("%Y-%m-%d").to_string(), "2025-10-04");
        
        // YYYY-MM-DD
        let result = parser.parse("2025-10-04").unwrap();
        assert_eq!(result.start_date.format("%Y-%m-%d").to_string(), "2025-10-04");
    }

    #[test]
    fn test_textual_date() {
        let parser = DateSearchParser::new();
        
        let result = parser.parse("dia 4 de outubro de 2025").unwrap();
        assert_eq!(result.start_date.format("%Y-%m-%d").to_string(), "2025-10-04");
        
        let result = parser.parse("4 de outubro de 2025").unwrap();
        assert_eq!(result.start_date.format("%Y-%m-%d").to_string(), "2025-10-04");
    }

    #[test]
    fn test_month_year() {
        let parser = DateSearchParser::new();
        
        let result = parser.parse("outubro 2025").unwrap();
        assert_eq!(result.start_date.format("%Y-%m-%d").to_string(), "2025-10-01");
        assert_eq!(result.end_date.format("%Y-%m-%d").to_string(), "2025-10-31");
    }

    #[test]
    fn test_month_only() {
        let parser = DateSearchParser::new();
        
        let result = parser.parse("documentos de outubro").unwrap();
        assert_eq!(result.start_date.month(), 10);
        assert_eq!(result.end_date.month(), 10);
    }

    #[test]
    fn test_non_date_query() {
        let parser = DateSearchParser::new();
        
        assert!(parser.parse("rastreabilidade carga").is_none());
        assert!(parser.parse("pdf documento").is_none());
    }

    #[test]
    fn test_false_positives_regression() {
        let parser = DateSearchParser::new();
        
        // CRÍTICO: Palavras que contêm abreviações de mês NÃO devem ser interpretadas como datas
        assert!(parser.parse("setor fiscal").is_none(), "setor não deve detectar 'set'");
        assert!(parser.parse("manual marcacao").is_none(), "manual não deve detectar 'mar'");
        assert!(parser.parse("maioria dos casos").is_none(), "maioria não deve detectar 'mai'");
        assert!(parser.parse("juntos no projeto").is_none(), "juntos não deve detectar 'jun'");
        assert!(parser.parse("agenda de trabalho").is_none(), "agenda não deve detectar 'ago'");
    }

    #[test]
    fn test_mixed_text_and_date_queries() {
        let parser = DateSearchParser::new();
        
        // CRÍTICO: Queries mistas (texto + data) NÃO devem ser interpretadas como busca por data
        // Devem ir para FTS5 para filtrar por texto + data
        assert!(parser.parse("rastreabilidade outubro").is_none(), "texto + mês deve ir para FTS5");
        assert!(parser.parse("carga dia 4 de outubro").is_none(), "palavras extras devem bloquear date search");
        assert!(parser.parse("documentos outubro").is_none(), "documentos + mês deve ir para FTS5");
        assert!(parser.parse("pdf de outubro").is_none(), "pdf + mês deve ir para FTS5");
        
        // Queries PURAS de data devem ser detectadas
        assert!(parser.parse("outubro").is_some(), "mês sozinho deve ser detectado");
        assert!(parser.parse("04/10/2025").is_some(), "data numérica deve ser detectada");
        assert!(parser.parse("4 de outubro de 2025").is_some(), "data textual pura deve ser detectada");
        assert!(parser.parse("outubro 2025").is_some(), "mês/ano deve ser detectado");
    }
}
