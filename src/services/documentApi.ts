// API Service para Sistema de Documentos - ARKIVE
// Conecta frontend React com backend Tauri/Rust
import { invoke } from '@tauri-apps/api/core';

// ================================
// INTERFACES E TIPOS
// ================================

export interface Document {
  id: string;
  name: string;
  file_path: string;
  file_type: string;
  file_size: number;
  created_at: string;
  updated_at: string;
  tags: string[];
}

export interface User {
  id: string;
  username: string;
  created_at: string;
}

export interface OCRResult {
  extracted_text: string;
  document_type: string;
  extracted_fields: Record<string, any>;
  confidence_score: number;
  processing_time_ms: number;
}

export interface SearchResponse {
  results: SearchResult[];
  total_found: number;
  search_time_ms: number;
  indexed_docs: number;
  total_docs: number;
}

export interface SearchResult {
  document_id: string;
  document_name: string;
  document_type: string;
  file_path: string;
  relevance_score: number;
  matched_content: string;
  created_at: string;
}

export interface AppStats {
  total_documents: number;
  total_storage: number;
  total_users: number;
  recent_activities: number;
}

// ================================
// API DE AUTENTICAÇÃO
// ================================

class AuthAPI {
  // Login do usuário
  static async login(username: string, password: string): Promise<User> {
    try {
      const result = await invoke<string>('login', {
        username,
        password
      });
      
      console.log('✅ Login realizado com sucesso');
      return JSON.parse(result);
    } catch (error) {
      console.error('❌ Erro no login:', error);
      throw new Error(String(error));
    }
  }

  // Registro de novo usuário
  static async register(username: string, password: string): Promise<User> {
    try {
      const result = await invoke<string>('register', {
        username,
        password
      });
      
      console.log('✅ Usuário registrado com sucesso');
      return JSON.parse(result);
    } catch (error) {
      console.error('❌ Erro no registro:', error);
      throw new Error(String(error));
    }
  }

  // Obter usuário atual
  static async getCurrentUser(): Promise<User | null> {
    try {
      const result = await invoke<string>('get_current_user');
      if (result === 'null' || result === '') {
        return null;
      }
      return JSON.parse(result);
    } catch (error) {
      console.warn('⚠️ Nenhum usuário autenticado');
      return null;
    }
  }

  // Logout
  static async logout(): Promise<void> {
    try {
      await invoke('logout');
      console.log('✅ Logout realizado');
    } catch (error) {
      console.error('❌ Erro no logout:', error);
      throw new Error(String(error));
    }
  }
}

// ================================
// API DE DOCUMENTOS
// ================================

export class DocumentAPI {
  // Obter lista de documentos
  static async getDocuments(): Promise<Document[]> {
    try {
      const result = await invoke<string>('get_documents');
      const documents = JSON.parse(result);
      console.log(`📄 ${documents.length} documentos carregados`);
      return documents;
    } catch (error) {
      console.error('❌ Erro ao carregar documentos:', error);
      throw new Error(String(error));
    }
  }

  // Processar documento com OCR simples
  static async processDocumentOCR(filePath: string): Promise<OCRResult> {
    try {
      const result = await invoke<string>('process_document_simple_ocr', {
        filePath
      });
      
      const ocrResult = JSON.parse(result);
      console.log(`🔍 OCR processado: ${ocrResult.document_type} (${ocrResult.processing_time_ms}ms)`);
      return ocrResult;
    } catch (error) {
      console.error('❌ Erro no processamento OCR:', error);
      throw new Error(String(error));
    }
  }

  // Criar documento no backend
  static async createDocument(filePath: string, ocrResult: OCRResult): Promise<{id: string}> {
    try {
      const result = await invoke<{id: string}>('create_document', {
        filePath,
        documentType: ocrResult.document_type,
        extractedText: ocrResult.extracted_text,
        extractedFields: ocrResult.extracted_fields,
        processingMethod: ocrResult.processing_method
      });
      console.log(`📄 Documento criado: ${result.id}`);
      return result;
    } catch (error) {
      console.error('❌ Erro ao criar documento:', error);
      throw new Error(String(error));
    }
  }

  // Deletar documento do backend
  static async deleteDocument(documentId: string): Promise<void> {
    try {
      await invoke('delete_document', { documentId });
      console.log(`🗑️ Documento ${documentId} excluído com sucesso`);
    } catch (error) {
      console.error('❌ Erro ao deletar documento:', error);
      throw new Error(String(error));
    }
  }

  // Obter tipos de documento suportados
  static async getSupportedTypes(): Promise<string[]> {
    try {
      const result = await invoke<string[]>('get_supported_document_types');
      return result;
    } catch (error) {
      console.error('❌ Erro ao obter tipos suportados:', error);
      return ['PDF', 'JPEG', 'PNG', 'GIF'];
    }
  }
}

// ================================
// API DE BUSCA FTS5
// ================================

export class SearchAPI {
  // Buscar documentos por texto
  static async searchDocuments(
    query: string, 
    limit?: number, 
    useFTS?: boolean
  ): Promise<SearchResponse> {
    try {
      const result = await invoke<SearchResponse>('search_documents', {
        query,
        limit: limit || 20,
        useFts: useFTS !== false // default true
      });
      
      console.log(`🔍 Busca "${query}" retornou ${result.total_found} resultados (${result.search_time_ms}ms)`);
      return result;
    } catch (error) {
      console.error('❌ Erro na busca:', error);
      throw new Error(String(error));
    }
  }

  // Indexar documento para busca
  static async indexDocument(
    documentId: string,
    extractedText: string,
    documentType: string,
    extractedFields: Record<string, any>
  ): Promise<boolean> {
    try {
      const result = await invoke<boolean>('index_document_for_search', {
        documentId,
        extractedText,
        documentType,
        extractedFields
      });
      
      if (result) {
        console.log(`📝 Documento ${documentId} indexado para busca`);
      }
      return result;
    } catch (error) {
      console.error('❌ Erro ao indexar documento:', error);
      throw new Error(String(error));
    }
  }

  // Obter estatísticas de busca
  static async getSearchStatistics(): Promise<{
    total_documents: number;
    indexed_documents: number;
    indexing_percentage: number;
    fts5_available: boolean;
  }> {
    try {
      const result = await invoke<any>('get_search_statistics');
      console.log(`📊 Estatísticas: ${result.indexed_documents}/${result.total_documents} indexados (${result.indexing_percentage}%)`);
      return result;
    } catch (error) {
      console.error('❌ Erro ao obter estatísticas:', error);
      return {
        total_documents: 0,
        indexed_documents: 0,
        indexing_percentage: 0,
        fts5_available: false
      };
    }
  }
}

// ================================
// API DE ESTATÍSTICAS
// ================================

class StatsAPI {
  // Obter estatísticas gerais
  static async getStats(): Promise<AppStats> {
    try {
      const result = await invoke<string>('get_stats');
      const stats = JSON.parse(result);
      console.log(`📊 Stats: ${stats.total_documents} docs, ${this.formatSize(stats.total_storage)} storage`);
      return stats;
    } catch (error) {
      console.error('❌ Erro ao obter estatísticas:', error);
      return {
        total_documents: 0,
        total_storage: 0,
        total_users: 1,
        recent_activities: 0
      };
    }
  }

  // Formatar tamanho de arquivo
  private static formatSize(bytes: number): string {
    if (bytes < 1024) return bytes + ' B';
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
    if (bytes < 1024 * 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
    return (bytes / (1024 * 1024 * 1024)).toFixed(1) + ' GB';
  }
}

// ================================
// UTILITÁRIOS
// ================================

class AppAPI {
  // Verificar se está rodando no Tauri
  static isTauriEnvironment(): boolean {
    return typeof window !== 'undefined' && (window as any).__TAURI__ !== undefined;
  }

  // Mostrar notificação de sucesso
  static showSuccess(message: string): void {
    console.log('✅ ' + message);
    // TODO: Implementar toast notifications
  }

  // Mostrar erro
  static showError(message: string): void {
    console.error('❌ ' + message);
    // TODO: Implementar toast notifications
  }

  // Formatar data brasileira
  static formatDate(dateStr: string): string {
    try {
      const date = new Date(dateStr);
      return date.toLocaleString('pt-BR', {
        day: '2-digit',
        month: '2-digit', 
        year: 'numeric',
        hour: '2-digit',
        minute: '2-digit'
      });
    } catch {
      return dateStr;
    }
  }
}

// ================================
// EXPORT PRINCIPAL
// ================================

export {
  AuthAPI,
  DocumentAPI, 
  SearchAPI,
  StatsAPI,
  AppAPI
};