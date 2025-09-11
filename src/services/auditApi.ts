// API centralizada para auditoria - compatível com Tauri Desktop e Web
import { invoke as tauriInvoke } from '@tauri-apps/api/core';

// Tipos para trilha de auditoria
export interface AuditLog {
  id: string;
  user_id: string;
  username: string;
  action: string;
  resource_type: string;
  resource_id?: string;
  resource_name?: string;
  ip_address?: string;
  file_hash?: string;
  current_hash: string;
  metadata: string;
  timestamp: string;
  is_success: boolean;
}

export interface AuditChainStatus {
  is_valid: boolean;
  total_logs: number;
  first_log_date?: string;
  last_log_date?: string;
}

export interface AuditFilters {
  action?: string;
  resourceType?: string;
  userId?: string;
  resourceId?: string;
  startDate?: string;
  endDate?: string;
  daysBack?: number;
  page?: number;
  limit?: number;
}

export interface PaginatedAuditLogs {
  logs: AuditLog[];
  total: number;
  page: number;
  totalPages: number;
  hasNext: boolean;
  hasPrevious: boolean;
}

export interface ExportOptions {
  format: 'json' | 'csv' | 'pdf';
  includeSignature?: boolean;
  includeMeta?: boolean;
  filters?: AuditFilters;
}

export interface ComplianceReport {
  exportDate: string;
  totalLogs: number;
  chainIntegrity: AuditChainStatus;
  filters: AuditFilters;
  signature?: string;
  fileHash?: string;
}

// Detecção de ambiente
const isTauriEnvironment = (): boolean => {
  return typeof window !== 'undefined' && window.__TAURI__ !== undefined;
};

// Mock data para desenvolvimento web
const generateMockAuditLogs = (filters: AuditFilters = {}): AuditLog[] => {
  const now = new Date();
  const mockLogs: AuditLog[] = [
    {
      id: '1',
      user_id: 'user1',
      username: 'admin@empresa.com',
      action: 'LOGIN',
      resource_type: 'SYSTEM',
      resource_id: null,
      resource_name: null,
      ip_address: '192.168.1.100',
      file_hash: null,
      current_hash: 'a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456',
      metadata: JSON.stringify({ ip_address: '192.168.1.100', success: true, browser: 'Chrome 118.0' }),
      timestamp: new Date(now.getTime() - 300000).toLocaleString('pt-BR'), // 5 min atrás
      is_success: true,
    },
    {
      id: '2',
      user_id: 'user1',
      username: 'admin@empresa.com',
      action: 'UPLOAD',
      resource_type: 'DOCUMENT',
      resource_id: 'doc1',
      resource_name: 'contrato_servicos_2025.pdf',
      ip_address: '192.168.1.100',
      file_hash: 'f1e2d3c4b5a6789012345678901234567890fedcba1234567890fedcba123456',
      current_hash: 'b2c3d4e5f6a7890123456789012345678901bcdef01234567890bcdef0123456',
      metadata: JSON.stringify({ 
        file_size: 2048576, 
        file_type: 'application/pdf',
        upload_source: 'drag_drop',
        scan_result: 'clean'
      }),
      timestamp: new Date(now.getTime() - 600000).toLocaleString('pt-BR'), // 10 min atrás
      is_success: true,
    },
    {
      id: '3',
      user_id: 'user2',
      username: 'usuario@empresa.com',
      action: 'LOGIN_FAILED',
      resource_type: 'SYSTEM',
      resource_id: null,
      resource_name: null,
      ip_address: '192.168.1.101',
      file_hash: null,
      current_hash: 'c3d4e5f6a7b8901234567890123456789012cdef011234567890cdef01234567',
      metadata: JSON.stringify({ 
        ip_address: '192.168.1.101', 
        reason: 'invalid_password',
        attempt_count: 3,
        locked_until: new Date(now.getTime() + 900000).toISOString()
      }),
      timestamp: new Date(now.getTime() - 900000).toLocaleString('pt-BR'), // 15 min atrás
      is_success: false,
    },
    {
      id: '4',
      user_id: 'user3',
      username: 'auditor@empresa.com',
      action: 'EXPORT_AUDIT',
      resource_type: 'SYSTEM',
      resource_id: 'audit_export_4',
      resource_name: 'audit_trail_2025-09-11.json',
      ip_address: '192.168.1.102',
      file_hash: null,
      current_hash: 'd4e5f6a7b8c9012345678901234567890123def021234567890def0123456789',
      metadata: JSON.stringify({
        export_format: 'json',
        total_records: 156,
        date_range: '2025-09-01 to 2025-09-11',
        signed: true
      }),
      timestamp: new Date(now.getTime() - 1200000).toLocaleString('pt-BR'), // 20 min atrás
      is_success: true,
    },
    {
      id: '5',
      user_id: 'user1',
      username: 'admin@empresa.com',
      action: 'VIEW',
      resource_type: 'DOCUMENT',
      resource_id: 'doc1',
      resource_name: 'contrato_servicos_2025.pdf',
      ip_address: '192.168.1.100',
      file_hash: 'f1e2d3c4b5a6789012345678901234567890fedcba1234567890fedcba123456',
      current_hash: 'e5f6a7b8c9d0123456789012345678901234ef031234567890ef01234567890',
      metadata: JSON.stringify({
        view_duration: 45,
        zoom_level: 100,
        pages_viewed: [1, 2, 3]
      }),
      timestamp: new Date(now.getTime() - 1500000).toLocaleString('pt-BR'), // 25 min atrás
      is_success: true,
    }
  ];

  // Aplicar filtros simples para mock
  let filtered = mockLogs;

  if (filters.action) {
    filtered = filtered.filter(log => log.action.toLowerCase().includes(filters.action!.toLowerCase()));
  }

  if (filters.resourceType) {
    filtered = filtered.filter(log => log.resource_type === filters.resourceType);
  }

  if (filters.userId) {
    filtered = filtered.filter(log => log.user_id === filters.userId);
  }

  return filtered;
};

const generateMockChainStatus = (): AuditChainStatus => ({
  is_valid: true,
  total_logs: 1247,
  first_log_date: '01/09/2025 08:00:00',
  last_log_date: new Date().toLocaleString('pt-BR'),
});

// API Class
export class AuditApi {
  private static instance: AuditApi;
  
  public static getInstance(): AuditApi {
    if (!AuditApi.instance) {
      AuditApi.instance = new AuditApi();
    }
    return AuditApi.instance;
  }

  /**
   * Buscar logs de auditoria com paginação
   */
  async getAuditLogs(filters: AuditFilters = {}): Promise<PaginatedAuditLogs> {
    if (isTauriEnvironment()) {
      try {
        const response = await tauriInvoke('get_audit_logs', {
          action: filters.action || null,
          resourceType: filters.resourceType || null,
          userId: filters.userId || null,
          resourceId: filters.resourceId || null,
          startDate: filters.startDate || null,
          endDate: filters.endDate || null,
          daysBack: filters.daysBack || 7,
          page: filters.page || 1,
          limit: filters.limit || 50
        });
        
        // Se o backend ainda não retorna paginação, simular
        if (Array.isArray(response)) {
          const logs = response as AuditLog[];
          return {
            logs,
            total: logs.length,
            page: 1,
            totalPages: 1,
            hasNext: false,
            hasPrevious: false
          };
        }
        
        return response as PaginatedAuditLogs;
      } catch (error) {
        console.error('Erro ao buscar logs via Tauri:', error);
        throw error;
      }
    } else {
      // Mock para desenvolvimento web
      await this.simulateDelay(300); // Simular latência de rede
      
      const allLogs = generateMockAuditLogs(filters);
      const page = filters.page || 1;
      const limit = filters.limit || 50;
      const startIndex = (page - 1) * limit;
      const endIndex = startIndex + limit;
      
      const logs = allLogs.slice(startIndex, endIndex);
      const total = allLogs.length;
      const totalPages = Math.ceil(total / limit);
      
      return {
        logs,
        total,
        page,
        totalPages,
        hasNext: page < totalPages,
        hasPrevious: page > 1
      };
    }
  }

  /**
   * Verificar integridade da cadeia de auditoria
   */
  async verifyAuditChain(): Promise<AuditChainStatus> {
    if (isTauriEnvironment()) {
      try {
        return await tauriInvoke('verify_audit_chain') as AuditChainStatus;
      } catch (error) {
        console.error('Erro ao verificar cadeia via Tauri:', error);
        throw error;
      }
    } else {
      // Mock para desenvolvimento web
      await this.simulateDelay(500);
      return generateMockChainStatus();
    }
  }

  /**
   * Exportar logs de auditoria para compliance
   */
  async exportAuditLogs(options: ExportOptions): Promise<ComplianceReport> {
    if (isTauriEnvironment()) {
      try {
        return await tauriInvoke('export_audit_logs', options) as ComplianceReport;
      } catch (error) {
        console.error('Erro ao exportar logs via Tauri:', error);
        throw error;
      }
    } else {
      // Mock para desenvolvimento web
      await this.simulateDelay(1000);
      
      const logs = generateMockAuditLogs(options.filters);
      const chainStatus = generateMockChainStatus();
      
      // Simular export baseado no formato
      const exportData = this.generateMockExport(logs, options);
      
      return {
        exportDate: new Date().toISOString(),
        totalLogs: logs.length,
        chainIntegrity: chainStatus,
        filters: options.filters || {},
        signature: options.includeSignature ? this.generateMockSignature() : undefined,
        fileHash: this.generateMockHash()
      };
    }
  }

  /**
   * Obter lista de usuários para filtros
   */
  async getUsers(): Promise<Array<{id: string, username: string, email: string}>> {
    if (isTauriEnvironment()) {
      try {
        return await tauriInvoke('get_users_list') as Array<{id: string, username: string, email: string}>;
      } catch (error) {
        console.error('Erro ao buscar usuários via Tauri:', error);
        return [];
      }
    } else {
      // Mock para desenvolvimento web
      await this.simulateDelay(200);
      return [
        { id: 'user1', username: 'admin', email: 'admin@empresa.com' },
        { id: 'user2', username: 'usuario', email: 'usuario@empresa.com' },
        { id: 'user3', username: 'auditor', email: 'auditor@empresa.com' },
        { id: 'user4', username: 'gestor', email: 'gestor@empresa.com' }
      ];
    }
  }

  /**
   * Obter lista de documentos para filtros
   */
  async getDocuments(): Promise<Array<{id: string, name: string, type: string}>> {
    if (isTauriEnvironment()) {
      try {
        return await tauriInvoke('get_documents_list') as Array<{id: string, name: string, type: string}>;
      } catch (error) {
        console.error('Erro ao buscar documentos via Tauri:', error);
        return [];
      }
    } else {
      // Mock para desenvolvimento web
      await this.simulateDelay(200);
      return [
        { id: 'doc1', name: 'contrato_servicos_2025.pdf', type: 'application/pdf' },
        { id: 'doc2', name: 'relatorio_financeiro_q3.xlsx', type: 'application/xlsx' },
        { id: 'doc3', name: 'politica_seguranca.docx', type: 'application/docx' },
        { id: 'doc4', name: 'manual_usuario.pdf', type: 'application/pdf' }
      ];
    }
  }

  // Métodos auxiliares privados
  private async simulateDelay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  private generateMockExport(logs: AuditLog[], options: ExportOptions): string {
    switch (options.format) {
      case 'csv':
        return this.generateCSV(logs);
      case 'pdf':
        return 'mock-pdf-content';
      case 'json':
      default:
        return JSON.stringify(logs, null, 2);
    }
  }

  private generateCSV(logs: AuditLog[]): string {
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
  }

  private generateMockSignature(): string {
    return 'SIGNATURE_RSA_2048_' + Math.random().toString(36).substring(2, 15).toUpperCase();
  }

  private generateMockHash(): string {
    return 'SHA256_' + Math.random().toString(36).substring(2, 15).toUpperCase();
  }
}

// Export singleton instance
export const auditApi = AuditApi.getInstance();