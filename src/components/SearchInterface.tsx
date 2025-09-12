// Interface de Busca FTS5 - ARKIVE
// Sistema de busca instantânea enterprise-grade
import React, { useState, useEffect, useRef, useMemo } from 'react';
import { Search, Filter, Clock, FileText, Zap, Loader2, CheckCircle } from 'lucide-react';
import { SearchAPI, SearchResult, SearchResponse } from '../services/documentApi';

interface SearchInterfaceProps {
  onResultSelect?: (result: SearchResult) => void;
  className?: string;
}

export default function SearchInterface({ onResultSelect, className = '' }: SearchInterfaceProps) {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<SearchResult[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const [searchStats, setSearchStats] = useState<SearchResponse | null>(null);
  const [searchHistory, setSearchHistory] = useState<string[]>([]);
  const [showHistory, setShowHistory] = useState(false);
  const [useFTS, setUseFTS] = useState(true);
  
  const searchInputRef = useRef<HTMLInputElement>(null);
  const debounceTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Stats de indexação
  const [indexStats, setIndexStats] = useState<{
    indexed_documents: number;
    total_documents: number;
    indexing_percentage: number;
    fts5_available: boolean;
  } | null>(null);

  // Carregar estatísticas de busca
  useEffect(() => {
    SearchAPI.getSearchStatistics()
      .then(setIndexStats)
      .catch(console.warn);
  }, []);

  // Busca com debounce para performance
  const performSearch = async (searchQuery: string, immediate = false) => {
    if (!searchQuery.trim()) {
      setResults([]);
      setSearchStats(null);
      return;
    }

    // Limpar timer anterior se não for busca imediata
    if (!immediate && debounceTimerRef.current) {
      clearTimeout(debounceTimerRef.current);
    }

    const executeSearch = async () => {
      setIsSearching(true);
      try {
        const response = await SearchAPI.searchDocuments(searchQuery, 20, useFTS);
        setResults(response.results);
        setSearchStats(response);
        
        // Adicionar ao histórico
        if (searchHistory.indexOf(searchQuery) === -1) {
          const newHistory = [searchQuery, ...searchHistory.slice(0, 9)]; // Max 10
          setSearchHistory(newHistory);
          localStorage.setItem('arkive_search_history', JSON.stringify(newHistory));
        }
      } catch (error) {
        console.error('Erro na busca:', error);
        setResults([]);
        setSearchStats(null);
      } finally {
        setIsSearching(false);
      }
    };

    if (immediate) {
      await executeSearch();
    } else {
      // Debounce de 300ms
      debounceTimerRef.current = window.setTimeout(executeSearch, 300);
    }
  };

  // Carregar histórico do localStorage
  useEffect(() => {
    try {
      const stored = localStorage.getItem('arkive_search_history');
      if (stored) {
        setSearchHistory(JSON.parse(stored));
      }
    } catch (error) {
      console.warn('Erro ao carregar histórico de busca:', error);
    }
  }, []);

  // Auto-busca quando query muda
  useEffect(() => {
    performSearch(query);
    return () => {
      if (debounceTimerRef.current) {
        clearTimeout(debounceTimerRef.current);
      }
    };
  }, [query, useFTS]);

  // Função segura para destacar termos encontrados (sem XSS)
  const highlightMatchesSafe = (text: string, searchTerm: string): React.ReactNode => {
    if (!searchTerm.trim() || !text) return text;
    
    // Escapar caracteres especiais do regex
    const escapedTerm = searchTerm.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    const regex = new RegExp(`(${escapedTerm})`, 'gi');
    
    // Dividir texto e criar elementos React seguros
    const parts = text.split(regex);
    
    return parts.map((part, index) => {
      // Verificar se é um match (case insensitive)
      if (regex.test(part)) {
        return (
          <mark 
            key={index} 
            className="bg-yellow-200 dark:bg-yellow-700 px-1 rounded font-medium"
          >
            {part}
          </mark>
        );
      }
      return part;
    });
  };

  // Formatar tipo de documento
  const formatDocumentType = (type: string): string => {
    const types: Record<string, string> = {
      'NotaFiscal': '🧾 Nota Fiscal',
      'Contrato': '📋 Contrato',
      'ReciboPagamento': '💳 Recibo',
      'DocumentoRH': '👥 RH',
      'DocumentoJuridico': '⚖️ Jurídico',
      'Relatorio': '📊 Relatório',
      'Generico': '📄 Documento'
    };
    return types[type] || `📄 ${type}`;
  };

  // Estatísticas de busca memorizadas
  const searchStatsDisplay = useMemo(() => {
    if (!searchStats) return null;

    const avgRelevance = searchStats.results.length > 0
      ? (searchStats.results.reduce((acc, r) => acc + r.relevance_score, 0) / searchStats.results.length).toFixed(2)
      : '0';

    return (
      <div className="flex items-center gap-4 text-sm text-gray-600 dark:text-gray-400 px-3 py-2 bg-gray-50 dark:bg-gray-800 rounded-lg">
        <span className="flex items-center gap-1">
          <CheckCircle className="w-4 h-4 text-green-500" />
          {searchStats.total_found} encontrados
        </span>
        <span className="flex items-center gap-1">
          <Clock className="w-4 h-4" />
          {searchStats.search_time_ms}ms
        </span>
        <span className="flex items-center gap-1">
          <Zap className="w-4 h-4" />
          Score médio: {avgRelevance}
        </span>
        {useFTS && (
          <span className="flex items-center gap-1 text-blue-600 dark:text-blue-400">
            <Filter className="w-4 h-4" />
            FTS5
          </span>
        )}
      </div>
    );
  }, [searchStats, useFTS]);

  return (
    <div className={`space-y-4 ${className}`}>
      {/* Header com estatísticas */}
      {indexStats && (
        <div className="flex items-center justify-between p-3 bg-blue-50 dark:bg-blue-900/20 rounded-lg border border-blue-200 dark:border-blue-800">
          <div className="flex items-center gap-3">
            <Search className="w-5 h-5 text-blue-600" />
            <div>
              <p className="text-sm font-medium text-blue-900 dark:text-blue-100">
                Busca Instantânea Ativa
              </p>
              <p className="text-xs text-blue-700 dark:text-blue-300">
                {indexStats.indexed_documents}/{indexStats.total_documents} documentos indexados
                ({indexStats.indexing_percentage}%)
              </p>
            </div>
          </div>
          
          {indexStats.fts5_available && (
            <label className="flex items-center gap-2 text-sm">
              <input
                type="checkbox"
                checked={useFTS}
                onChange={(e) => setUseFTS(e.target.checked)}
                className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
              />
              <span className="text-blue-700 dark:text-blue-300">FTS5</span>
            </label>
          )}
        </div>
      )}

      {/* Input de busca */}
      <div className="relative">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-5 h-5 text-gray-400" />
          <input
            ref={searchInputRef}
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onFocus={() => setShowHistory(true)}
            onBlur={() => setTimeout(() => setShowHistory(false), 200)}
            placeholder="Buscar documentos por texto, CNPJ, valores, datas..."
            className="w-full pl-10 pr-10 py-3 border border-gray-300 dark:border-gray-600 rounded-lg 
                     bg-white dark:bg-gray-800 text-gray-900 dark:text-white
                     focus:ring-2 focus:ring-blue-500 focus:border-transparent
                     placeholder-gray-500 dark:placeholder-gray-400"
          />
          {isSearching && (
            <Loader2 className="absolute right-3 top-1/2 transform -translate-y-1/2 w-5 h-5 text-blue-500 animate-spin" />
          )}
        </div>

        {/* Histórico de busca */}
        {showHistory && searchHistory.length > 0 && (
          <div className="absolute top-full left-0 right-0 mt-1 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg z-10">
            <div className="p-2">
              <p className="text-xs font-medium text-gray-500 dark:text-gray-400 mb-2">BUSCAS RECENTES</p>
              {searchHistory.slice(0, 5).map((term, index) => (
                <button
                  key={index}
                  onClick={() => {
                    setQuery(term);
                    setShowHistory(false);
                    performSearch(term, true);
                  }}
                  className="w-full text-left px-2 py-1.5 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded"
                >
                  <Clock className="w-3 h-3 inline-block mr-2 text-gray-400" />
                  {term}
                </button>
              ))}
            </div>
          </div>
        )}
      </div>

      {/* Estatísticas da busca atual */}
      {searchStatsDisplay}

      {/* Resultados */}
      {results.length > 0 && (
        <div className="space-y-2">
          <h3 className="font-medium text-gray-900 dark:text-white">
            Resultados da Busca
          </h3>
          
          <div className="space-y-2 max-h-96 overflow-y-auto">
            {results.map((result) => (
              <div
                key={result.document_id}
                onClick={() => onResultSelect?.(result)}
                className="p-4 border border-gray-200 dark:border-gray-700 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-800 cursor-pointer transition-colors"
              >
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <div className="flex items-center gap-2 mb-1">
                      <FileText className="w-4 h-4 text-gray-500" />
                      <h4 className="font-medium text-gray-900 dark:text-white">
                        {result.document_name}
                      </h4>
                      <span className="text-xs px-2 py-1 bg-gray-100 dark:bg-gray-700 rounded">
                        {formatDocumentType(result.document_type)}
                      </span>
                    </div>
                    
                    {result.matched_content && (
                      <p className="text-sm text-gray-600 dark:text-gray-400 mt-2">
                        {highlightMatchesSafe(result.matched_content, query)}
                      </p>
                    )}
                    
                    <div className="flex items-center gap-4 mt-2 text-xs text-gray-500">
                      <span>{result.created_at}</span>
                      <span>Score: {result.relevance_score.toFixed(2)}</span>
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Estado vazio */}
      {query.trim() && !isSearching && results.length === 0 && (
        <div className="text-center py-8 text-gray-500 dark:text-gray-400">
          <Search className="w-12 h-12 mx-auto mb-4 opacity-50" />
          <p>Nenhum documento encontrado para "{query}"</p>
          <p className="text-sm mt-1">Tente termos diferentes ou verifique a ortografia</p>
        </div>
      )}

      {/* Placeholder inicial */}
      {!query.trim() && (
        <div className="text-center py-8 text-gray-500 dark:text-gray-400">
          <Search className="w-12 h-12 mx-auto mb-4 opacity-50" />
          <p>Digite para buscar seus documentos</p>
          <p className="text-sm mt-1">Busque por nome, conteúdo, CNPJ, valores ou datas</p>
        </div>
      )}
    </div>
  );
}