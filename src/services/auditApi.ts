// API centralizada para auditoria - SOMENTE Tauri Desktop (SEM DADOS MOCKADOS)
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
  return typeof window !== 'undefined' && (window as any).__TAURI__ !== undefined;
};

// API Class - SOMENTE TAURI (SEM MOCKS)
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
   * SOMENTE DADOS REAIS DO BANCO DE DADOS - SEM MOCKS
   */
  async getAuditLogs(filters: AuditFilters = {}): Promise<PaginatedAuditLogs> {
    if (!isTauriEnvironment()) {
      throw new Error('ARKIVE deve ser executado como aplicação desktop Tauri. Dados mockados foram removidos por solicitação do usuário.');
    }

    try {
      const response = await tauriInvoke('get_audit_logs', {
        action: filters.action || null,
        resource_type: filters.resourceType || null,
        user_id: filters.userId || null,
        resource_id: filters.resourceId || null,
        start_date: filters.startDate || null,
        end_date: filters.endDate || null,
        days_back: filters.daysBack || 7,
        page: filters.page || 1,
        limit: filters.limit || 50
      });
      
      // Se o backend retorna array simples, envolver em estrutura paginada
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
      throw new Error(`Falha ao buscar logs de auditoria: ${error}`);
    }
  }

  /**
   * Verificar integridade da cadeia de auditoria
   * SOMENTE DADOS REAIS DO BANCO DE DADOS - SEM MOCKS
   */
  async verifyAuditChain(): Promise<AuditChainStatus> {
    if (!isTauriEnvironment()) {
      throw new Error('ARKIVE deve ser executado como aplicação desktop Tauri. Dados mockados foram removidos por solicitação do usuário.');
    }

    try {
      return await tauriInvoke('verify_audit_chain') as AuditChainStatus;
    } catch (error) {
      console.error('Erro ao verificar cadeia via Tauri:', error);
      throw new Error(`Falha ao verificar integridade da cadeia: ${error}`);
    }
  }

  /**
   * Exportar logs de auditoria para compliance
   * SOMENTE DADOS REAIS DO BANCO DE DADOS - SEM MOCKS
   */
  async exportAuditLogs(options: ExportOptions): Promise<ComplianceReport> {
    if (!isTauriEnvironment()) {
      throw new Error('ARKIVE deve ser executado como aplicação desktop Tauri. Dados mockados foram removidos por solicitação do usuário.');
    }

    try {
      return await tauriInvoke('export_audit_logs', options as Record<string, unknown>) as ComplianceReport;
    } catch (error) {
      console.error('Erro ao exportar logs via Tauri:', error);
      throw new Error(`Falha ao exportar logs de auditoria: ${error}`);
    }
  }

  /**
   * Obter lista de usuários para filtros
   * SOMENTE DADOS REAIS DO BANCO DE DADOS - SEM MOCKS
   */
  async getUsers(): Promise<Array<{id: string, username: string, email: string}>> {
    if (!isTauriEnvironment()) {
      throw new Error('ARKIVE deve ser executado como aplicação desktop Tauri. Dados mockados foram removidos por solicitação do usuário.');
    }

    try {
      const users = await tauriInvoke('get_users_list') as Array<{id: string, username: string, email: string}>;
      return users || [];
    } catch (error) {
      console.error('Erro ao buscar usuários via Tauri:', error);
      return [];
    }
  }

  /**
   * Obter lista de documentos para filtros
   * SOMENTE DADOS REAIS DO BANCO DE DADOS - SEM MOCKS
   */
  async getDocuments(): Promise<Array<{id: string, name: string, type: string}>> {
    if (!isTauriEnvironment()) {
      throw new Error('ARKIVE deve ser executado como aplicação desktop Tauri. Dados mockados foram removidos por solicitação do usuário.');
    }

    try {
      const documents = await tauriInvoke('get_documents_list') as Array<{id: string, name: string, type: string}>;
      return documents || [];
    } catch (error) {
      console.error('Erro ao buscar documentos via Tauri:', error);
      return [];
    }
  }
}

// Export singleton instance
export const auditApi = AuditApi.getInstance();
