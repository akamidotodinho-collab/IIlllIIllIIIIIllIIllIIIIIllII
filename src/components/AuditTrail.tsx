import React, { useState, useEffect, useCallback } from 'react';
import { 
  Shield, Clock, User, FileText, Download, Filter, CheckCircle, AlertTriangle, 
  ChevronLeft, ChevronRight, RefreshCw, Settings, FileDown, Calendar,
  Search, X, Eye, ChevronDown, Hash, AlertCircle
} from 'lucide-react';
import { 
  auditApi, 
  AuditLog, 
  AuditChainStatus, 
  AuditFilters, 
  PaginatedAuditLogs,
  ExportOptions,
  ComplianceReport
} from '../services/auditApi';
import DateRangePicker from './DateRangePicker';
import AuditLogDrawer from './AuditLogDrawer';

export default function AuditTrail() {
  // Estados principais
  const [auditData, setAuditData] = useState<PaginatedAuditLogs>({
    logs: [],
    total: 0,
    page: 1,
    totalPages: 1,
    hasNext: false,
    hasPrevious: false
  });
  const [chainStatus, setChainStatus] = useState<AuditChainStatus | null>(null);
  const [loading, setLoading] = useState(false);
  const [refreshing, setRefreshing] = useState(false);
  const [exporting, setExporting] = useState(false);
  
  // Estados de filtros
  const [filters, setFilters] = useState<AuditFilters>({
    action: '',
    resourceType: '',
    userId: '',
    resourceId: '',
    startDate: '',
    endDate: '',
    page: 1,
    limit: 50
  });
  const [showFilters, setShowFilters] = useState(false);
  
  // Estados para seletores
  const [users, setUsers] = useState<Array<{id: string, username: string, email: string}>>([]);
  const [documents, setDocuments] = useState<Array<{id: string, name: string, type: string}>>([]);
  
  // Estados do drawer
  const [selectedLog, setSelectedLog] = useState<AuditLog | null>(null);
  const [drawerOpen, setDrawerOpen] = useState(false);
  
  // Estados de busca rápida
  const [quickSearch, setQuickSearch] = useState('');
  const [searchTimeout, setSearchTimeout] = useState<NodeJS.Timeout | null>(null);

  // Buscar logs de auditoria
  const fetchAuditLogs = useCallback(async (newFilters?: Partial<AuditFilters>) => {
    setLoading(true);
    try {
      const currentFilters = { ...filters, ...newFilters };
      const data = await auditApi.getAuditLogs(currentFilters);
      setAuditData(data);
      
      // Atualizar filtros se necessário
      if (newFilters) {
        setFilters(currentFilters);
      }
    } catch (error) {
      console.error('Erro ao buscar logs de auditoria:', error);
      // Em caso de erro, manter dados anteriores mas mostrar erro
    } finally {
      setLoading(false);
    }
  }, [filters]);

  // Verificar integridade da cadeia
  const verifyChain = useCallback(async () => {
    try {
      const status = await auditApi.verifyAuditChain();
      setChainStatus(status);
    } catch (error) {
      console.error('Erro ao verificar cadeia:', error);
    }
  }, []);
  
  // Carregar dados iniciais
  const loadInitialData = useCallback(async () => {
    setLoading(true);
    try {
      const [usersData, documentsData] = await Promise.all([
        auditApi.getUsers(),
        auditApi.getDocuments()
      ]);
      setUsers(usersData);
      setDocuments(documentsData);
    } catch (error) {
      console.error('Erro ao carregar dados iniciais:', error);
    } finally {
      setLoading(false);
    }
  }, []);
  
  // Refresh manual
  const handleRefresh = useCallback(async () => {
    setRefreshing(true);
    try {
      await Promise.all([
        fetchAuditLogs(),
        verifyChain()
      ]);
    } catch (error) {
      console.error('Erro ao atualizar dados:', error);
    } finally {
      setRefreshing(false);
    }
  }, [fetchAuditLogs, verifyChain]);

  // Export logs para compliance
  const exportLogs = useCallback(async (format: 'json' | 'csv' | 'pdf' = 'json') => {
    setExporting(true);
    try {
      const exportOptions: ExportOptions = {
        format,
        includeSignature: true,
        includeMeta: true,
        filters: filters
      };
      
      const report = await auditApi.exportAuditLogs(exportOptions);
      
      // Criar arquivo baseado no formato
      let content: string;
      let mimeType: string;
      let fileName: string;
      
      const timestamp = new Date().toISOString().split('T')[0];
      
      switch (format) {
        case 'csv':
          content = generateCSVFromLogs(auditData.logs);
          mimeType = 'text/csv';
          fileName = `ARKIVE_audit_trail_${timestamp}.csv`;
          break;
        case 'pdf':
          // Para PDF, simular ou usar uma biblioteca como jsPDF
          content = generateComplianceReport(report);
          mimeType = 'application/pdf';
          fileName = `ARKIVE_compliance_report_${timestamp}.pdf`;
          break;
        case 'json':
        default:
          content = JSON.stringify({
            report,
            logs: auditData.logs
          }, null, 2);
          mimeType = 'application/json';
          fileName = `ARKIVE_audit_trail_${timestamp}.json`;
          break;
      }
      
      // Download do arquivo
      const blob = new Blob([content], { type: mimeType });
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = fileName;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      URL.revokeObjectURL(url);
      
    } catch (error) {
      console.error('Erro ao exportar logs:', error);
    } finally {
      setExporting(false);
    }
  }, [auditData.logs, filters]);
  
  // Gerar CSV dos logs
  const generateCSVFromLogs = (logs: AuditLog[]): string => {
    const headers = [
      'ID', 'Data/Hora', 'Usuário', 'Ação', 'Tipo Recurso', 
      'Nome Recurso', 'IP', 'Hash Arquivo', 'Hash Cadeia', 'Sucesso', 'Metadados'
    ];
    
    const rows = logs.map(log => [
      log.id,
      log.timestamp,
      log.username,
      log.action,
      log.resource_type,
      log.resource_name || '',
      log.ip_address || '',
      log.file_hash || '',
      log.current_hash,
      log.is_success ? 'Sim' : 'Não',
      log.metadata
    ]);

    return [headers, ...rows]
      .map(row => row.map(cell => `"${cell}"`).join(','))
      .join('\n');
  };
  
  // Gerar relatório de compliance
  const generateComplianceReport = (report: ComplianceReport): string => {
    return JSON.stringify({
      title: 'ARKIVE - Relatório de Compliance de Auditoria',
      generated_at: report.exportDate,
      summary: {
        total_logs: report.totalLogs,
        chain_integrity: report.chainIntegrity,
        verification_signature: report.signature,
        file_hash: report.fileHash
      },
      filters_applied: report.filters,
      note: 'Este relatório foi gerado automaticamente e contém assinatura criptográfica para verificação de integridade.'
    }, null, 2);
  };

  // Busca com debounce
  const handleQuickSearch = useCallback((term: string) => {
    setQuickSearch(term);
    
    if (searchTimeout) {
      clearTimeout(searchTimeout);
    }
    
    const timeout = setTimeout(() => {
      fetchAuditLogs({ 
        ...filters, 
        page: 1 // Reset página na busca
      });
    }, 500);
    
    setSearchTimeout(timeout);
  }, [filters, fetchAuditLogs, searchTimeout]);
  
  // Paginação
  const handlePageChange = useCallback((newPage: number) => {
    fetchAuditLogs({ page: newPage });
  }, [fetchAuditLogs]);
  
  // Abrir drawer de detalhes
  const openLogDetails = useCallback((log: AuditLog) => {
    setSelectedLog(log);
    setDrawerOpen(true);
  }, []);
  
  // Fechar drawer
  const closeDrawer = useCallback(() => {
    setDrawerOpen(false);
    setTimeout(() => setSelectedLog(null), 300); // Delay para animação
  }, []);
  
  // Aplicar filtros
  const applyFilters = useCallback(() => {
    fetchAuditLogs({ ...filters, page: 1 });
    setShowFilters(false);
  }, [fetchAuditLogs, filters]);
  
  // Limpar filtros
  const clearFilters = useCallback(() => {
    const clearedFilters: AuditFilters = {
      action: '',
      resourceType: '',
      userId: '',
      resourceId: '',
      startDate: '',
      endDate: '',
      page: 1,
      limit: 50
    };
    setFilters(clearedFilters);
    fetchAuditLogs(clearedFilters);
  }, [fetchAuditLogs]);
  
  // Effects
  useEffect(() => {
    loadInitialData();
  }, [loadInitialData]);
  
  useEffect(() => {
    fetchAuditLogs();
    verifyChain();
  }, []);
  
  useEffect(() => {
    return () => {
      if (searchTimeout) {
        clearTimeout(searchTimeout);
      }
    };
  }, [searchTimeout]);

  // Ícone da ação
  const getActionIcon = (action: string) => {
    switch (action.toUpperCase()) {
      case 'LOGIN':
        return <User className="w-4 h-4 text-green-500" />;
      case 'LOGIN_FAILED':
        return <User className="w-4 h-4 text-red-500" />;
      case 'UPLOAD':
        return <FileText className="w-4 h-4 text-blue-500" />;
      case 'DOWNLOAD':
        return <Download className="w-4 h-4 text-purple-500" />;
      case 'VIEW':
        return <Eye className="w-4 h-4 text-gray-500" />;
      case 'EXPORT_AUDIT':
        return <Shield className="w-4 h-4 text-orange-500" />;
      default:
        return <Clock className="w-4 h-4 text-gray-400" />;
    }
  };

  // Cor do badge de sucesso
  const getSuccessColor = (isSuccess: boolean) => {
    return isSuccess 
      ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200'
      : 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200';
  };

  return (
    <div className="space-y-6">
      {/* Header da Trilha de Auditoria */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-3">
            <Shield className="w-6 h-6 text-blue-500" />
            <div>
              <h2 className="text-xl font-bold text-gray-900 dark:text-white">
                Trilha de Auditoria Legal
              </h2>
              <p className="text-sm text-gray-600 dark:text-gray-400">
                {auditData.total.toLocaleString()} registros encontrados
              </p>
            </div>
          </div>
          
          <div className="flex items-center gap-2">
            <button
              onClick={handleRefresh}
              disabled={refreshing}
              className="flex items-center gap-2 px-3 py-2 text-gray-600 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
              title="Atualizar"
            >
              <RefreshCw className={`w-4 h-4 ${refreshing ? 'animate-spin' : ''}`} />
            </button>
            
            <button
              onClick={() => setShowFilters(!showFilters)}
              className={`flex items-center gap-2 px-4 py-2 rounded-lg transition-colors ${
                showFilters 
                  ? 'bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-200' 
                  : 'bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600'
              }`}
            >
              <Filter className="w-4 h-4" />
              Filtros
              {Object.values(filters).some(v => v && v !== '' && v !== 1 && v !== 50) && (
                <span className="w-2 h-2 bg-blue-500 rounded-full"></span>
              )}
            </button>
            
            <div className="relative">
              <button
                onClick={() => exportLogs('json')}
                disabled={auditData.logs.length === 0 || exporting}
                className="flex items-center gap-2 px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
              >
                {exporting ? (
                  <RefreshCw className="w-4 h-4 animate-spin" />
                ) : (
                  <Download className="w-4 h-4" />
                )}
                Export
              </button>
            </div>
          </div>
        </div>

        {/* Status da Cadeia */}
        {chainStatus && (
          <div className={`p-4 rounded-lg border ${
            chainStatus.is_valid 
              ? 'bg-green-50 border-green-200 dark:bg-green-900/20 dark:border-green-800' 
              : 'bg-red-50 border-red-200 dark:bg-red-900/20 dark:border-red-800'
          }`}>
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                {chainStatus.is_valid ? (
                  <CheckCircle className="w-5 h-5 text-green-500" />
                ) : (
                  <AlertTriangle className="w-5 h-5 text-red-500" />
                )}
                <span className={`font-semibold ${
                  chainStatus.is_valid ? 'text-green-800 dark:text-green-200' : 'text-red-800 dark:text-red-200'
                }`}>
                  {chainStatus.is_valid ? 'Cadeia Criptográfica Íntegra' : 'Cadeia Comprometida'}
                </span>
              </div>
              
              {!chainStatus.is_valid && (
                <AlertCircle className="w-5 h-5 text-red-500" title="Verificação de integridade falhou" />
              )}
            </div>
            
            <div className="mt-2 text-sm">
              <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                <div>
                  <span className="font-medium">Total de Logs:</span> {chainStatus.total_logs.toLocaleString()}
                </div>
                <div>
                  <span className="font-medium">Primeiro Log:</span> {chainStatus.first_log_date || 'N/A'}
                </div>
                <div>
                  <span className="font-medium">Último Log:</span> {chainStatus.last_log_date || 'N/A'}
                </div>
              </div>
              
              {chainStatus.is_valid && (
                <div className="mt-2 text-xs text-green-700 dark:text-green-300">
                  ✓ Todos os logs foram verificados criptograficamente e a cadeia de integridade está intacta.
                </div>
              )}
              
              {!chainStatus.is_valid && (
                <div className="mt-2 text-xs text-red-700 dark:text-red-300">
                  ⚠️ A verificação de integridade detectou inconsistências na cadeia de auditoria. Contate o administrador do sistema.
                </div>
              )}
            </div>
          </div>
        )}

        {/* Busca Rápida */}
        <div className="mt-4">
          <div className="relative max-w-md">
            <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" />
            <input
              type="text"
              value={quickSearch}
              onChange={(e) => handleQuickSearch(e.target.value)}
              placeholder="Buscar por usuário, ação, documento..."
              className="w-full pl-10 pr-10 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
            {quickSearch && (
              <button
                onClick={() => handleQuickSearch('')}
                className="absolute right-3 top-1/2 transform -translate-y-1/2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
              >
                <X className="w-4 h-4" />
              </button>
            )}
          </div>
        </div>
      </div>

      {/* Filtros Avançados */}
      {showFilters && (
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white">Filtros Avançados</h3>
            <button
              onClick={clearFilters}
              className="text-sm text-gray-600 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-200"
            >
              Limpar todos
            </button>
          </div>
          
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4 mb-4">
            <div>
              <label className="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                Ação
              </label>
              <select
                value={filters.action || ''}
                onChange={(e) => setFilters(prev => ({ ...prev, action: e.target.value }))}
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              >
                <option value="">Todas as Ações</option>
                <option value="LOGIN">Login</option>
                <option value="LOGIN_FAILED">Login Falhou</option>
                <option value="UPLOAD">Upload</option>
                <option value="DOWNLOAD">Download</option>
                <option value="VIEW">Visualização</option>
                <option value="EXPORT_AUDIT">Export de Auditoria</option>
              </select>
            </div>
            
            <div>
              <label className="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                Tipo de Recurso
              </label>
              <select
                value={filters.resourceType || ''}
                onChange={(e) => setFilters(prev => ({ ...prev, resourceType: e.target.value }))}
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              >
                <option value="">Todos os Tipos</option>
                <option value="SYSTEM">Sistema</option>
                <option value="DOCUMENT">Documento</option>
                <option value="USER">Usuário</option>
              </select>
            </div>
            
            <div>
              <label className="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                Usuário
              </label>
              <select
                value={filters.userId || ''}
                onChange={(e) => setFilters(prev => ({ ...prev, userId: e.target.value }))}
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              >
                <option value="">Todos os Usuários</option>
                {users.map(user => (
                  <option key={user.id} value={user.id}>
                    {user.username} ({user.email})
                  </option>
                ))}
              </select>
            </div>
            
            <div>
              <label className="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                Documento
              </label>
              <select
                value={filters.resourceId || ''}
                onChange={(e) => setFilters(prev => ({ ...prev, resourceId: e.target.value }))}
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              >
                <option value="">Todos os Documentos</option>
                {documents.map(doc => (
                  <option key={doc.id} value={doc.id}>
                    {doc.name}
                  </option>
                ))}
              </select>
            </div>
          </div>
          
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
            <div>
              <label className="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                Período
              </label>
              <DateRangePicker
                startDate={filters.startDate}
                endDate={filters.endDate}
                onDateChange={(start, end) => setFilters(prev => ({
                  ...prev,
                  startDate: start || '',
                  endDate: end || ''
                }))}
              />
            </div>
            
            <div>
              <label className="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                Registros por página
              </label>
              <select
                value={filters.limit || 50}
                onChange={(e) => setFilters(prev => ({ ...prev, limit: parseInt(e.target.value) }))}
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              >
                <option value={25}>25 registros</option>
                <option value={50}>50 registros</option>
                <option value={100}>100 registros</option>
                <option value={200}>200 registros</option>
              </select>
            </div>
          </div>
          
          <div className="flex justify-end gap-2">
            <button
              onClick={() => setShowFilters(false)}
              className="px-4 py-2 text-gray-600 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-200"
            >
              Cancelar
            </button>
            <button
              onClick={applyFilters}
              className="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600"
            >
              Aplicar Filtros
            </button>
          </div>
        </div>
      )}

      {/* Timeline de Auditoria */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow">
        <div className="p-6 border-b border-gray-200 dark:border-gray-700">
          <div className="flex items-center justify-between">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
              Timeline de Atividades
            </h3>
            
            {/* Controles de Export */}
            <div className="flex items-center gap-2">
              <button
                onClick={() => exportLogs('csv')}
                disabled={auditData.logs.length === 0 || exporting}
                className="flex items-center gap-2 px-3 py-2 text-sm bg-green-500 text-white rounded hover:bg-green-600 disabled:opacity-50"
                title="Exportar como CSV"
              >
                <FileDown className="w-4 h-4" />
                CSV
              </button>
              
              <button
                onClick={() => exportLogs('pdf')}
                disabled={auditData.logs.length === 0 || exporting}
                className="flex items-center gap-2 px-3 py-2 text-sm bg-red-500 text-white rounded hover:bg-red-600 disabled:opacity-50"
                title="Exportar relatório PDF"
              >
                <FileDown className="w-4 h-4" />
                PDF
              </button>
            </div>
          </div>
          
          {/* Info de paginação */}
          {auditData.total > 0 && (
            <div className="mt-2 text-sm text-gray-600 dark:text-gray-400">
              Mostrando {((auditData.page - 1) * (filters.limit || 50)) + 1} - {Math.min(auditData.page * (filters.limit || 50), auditData.total)} de {auditData.total.toLocaleString()} registros
            </div>
          )}
        </div>
        
        <div className="p-6">
          {loading ? (
            <div className="text-center py-8">
              <div className="animate-spin w-8 h-8 border-4 border-blue-500 border-t-transparent rounded-full mx-auto mb-2"></div>
              <p className="text-gray-600 dark:text-gray-400">Carregando logs de auditoria...</p>
            </div>
          ) : auditData.logs.length === 0 ? (
            <div className="text-center py-8">
              <Shield className="w-12 h-12 text-gray-400 mx-auto mb-4" />
              <h4 className="text-lg font-medium text-gray-900 dark:text-white mb-2">
                Nenhum log encontrado
              </h4>
              <p className="text-gray-600 dark:text-gray-400 mb-4">
                Não há registros de auditoria para os filtros aplicados
              </p>
              {Object.values(filters).some(v => v && v !== '' && v !== 1 && v !== 50) && (
                <button
                  onClick={clearFilters}
                  className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
                >
                  Limpar Filtros
                </button>
              )}
            </div>
          ) : (
            <>
              <div className="space-y-3">
                {auditData.logs.map((log) => (
                  <div 
                    key={log.id} 
                    className="flex gap-4 p-4 border border-gray-200 dark:border-gray-700 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors cursor-pointer group"
                    onClick={() => openLogDetails(log)}
                  >
                    <div className="flex-shrink-0">
                      {getActionIcon(log.action)}
                    </div>
                    
                    <div className="flex-grow min-w-0">
                      <div className="flex items-center justify-between mb-2">
                        <div className="flex items-center gap-2 min-w-0">
                          <span className="font-semibold text-gray-900 dark:text-white truncate">
                            {log.username}
                          </span>
                          <span className="text-gray-500 dark:text-gray-400">•</span>
                          <span className="text-gray-600 dark:text-gray-300">
                            {log.action}
                          </span>
                          {log.resource_name && (
                            <>
                              <span className="text-gray-500 dark:text-gray-400">•</span>
                              <span className="text-gray-600 dark:text-gray-300 truncate">
                                {log.resource_name}
                              </span>
                            </>
                          )}
                        </div>
                        
                        <div className="flex items-center gap-2 flex-shrink-0">
                          <span className={`px-2 py-1 text-xs font-medium rounded-full ${getSuccessColor(log.is_success)}`}>
                            {log.is_success ? 'Sucesso' : 'Falha'}
                          </span>
                          <span className="text-sm text-gray-500 dark:text-gray-400">
                            {log.timestamp}
                          </span>
                          <Eye className="w-4 h-4 text-gray-400 group-hover:text-gray-600 dark:group-hover:text-gray-300 opacity-0 group-hover:opacity-100 transition-opacity" />
                        </div>
                      </div>
                      
                      <div className="text-sm text-gray-600 dark:text-gray-400">
                        <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
                          <div>
                            <span className="font-medium">Tipo:</span> {log.resource_type}
                          </div>
                          {log.ip_address && (
                            <div>
                              <span className="font-medium">IP:</span> {log.ip_address}
                            </div>
                          )}
                          <div className="flex items-center gap-1">
                            <Hash className="w-3 h-3" />
                            <span className="font-mono text-xs">{log.current_hash.substring(0, 12)}...</span>
                          </div>
                          <div className="text-right">
                            <span className="text-xs text-blue-600 dark:text-blue-400 group-hover:underline">
                              Ver detalhes →
                            </span>
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
              
              {/* Paginação */}
              {auditData.totalPages > 1 && (
                <div className="mt-6 flex items-center justify-between">
                  <div className="text-sm text-gray-600 dark:text-gray-400">
                    Página {auditData.page} de {auditData.totalPages}
                  </div>
                  
                  <div className="flex items-center gap-2">
                    <button
                      onClick={() => handlePageChange(auditData.page - 1)}
                      disabled={!auditData.hasPrevious || loading}
                      className="flex items-center gap-1 px-3 py-2 text-sm border border-gray-300 dark:border-gray-600 rounded hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                      <ChevronLeft className="w-4 h-4" />
                      Anterior
                    </button>
                    
                    <span className="px-3 py-2 text-sm bg-blue-100 dark:bg-blue-900 text-blue-700 dark:text-blue-200 rounded">
                      {auditData.page}
                    </span>
                    
                    <button
                      onClick={() => handlePageChange(auditData.page + 1)}
                      disabled={!auditData.hasNext || loading}
                      className="flex items-center gap-1 px-3 py-2 text-sm border border-gray-300 dark:border-gray-600 rounded hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                      Próxima
                      <ChevronRight className="w-4 h-4" />
                    </button>
                  </div>
                </div>
              )}
            </>
          )}
        </div>
      </div>
      
      {/* Drawer de Detalhes */}
      <AuditLogDrawer
        log={selectedLog}
        isOpen={drawerOpen}
        onClose={closeDrawer}
      />
    </div>
  );
}