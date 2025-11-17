// API Service para Sistema de Backup - ARKIVE
// Conecta frontend React com backend Tauri/Rust
import { invoke } from '@tauri-apps/api/core';

// ================================
// INTERFACES E TIPOS
// ================================

export interface BackupInfo {
  created_at: string;
  version: string;
  database_size: number;
  files_count: number;
  checksum: string;
}

export interface BackupListItem {
  path: string;
  info: BackupInfo;
}

// ================================
// API DE BACKUP
// ================================

class BackupAPI {
  // Listar todos os backups disponíveis
  static async listBackups(): Promise<BackupListItem[]> {
    try {
      const result = await invoke<Array<[string, BackupInfo]>>('list_available_backups');
      
      // Converter tuplas Rust para objetos TypeScript
      const backups: BackupListItem[] = result.map(([path, info]) => ({
        path,
        info
      }));
      
      console.log(`💾 ${backups.length} backup(s) encontrado(s)`);
      return backups;
    } catch (error) {
      console.error('❌ Erro ao listar backups:', error);
      throw new Error(String(error));
    }
  }

  // Verificar integridade de um backup específico
  static async verifyBackup(backupPath: string): Promise<BackupInfo> {
    try {
      const result = await invoke<BackupInfo>('verify_backup_file', {
        backup_path: backupPath
      });
      
      console.log(`✅ Backup verificado: ${backupPath}`);
      return result;
    } catch (error) {
      console.error('❌ Erro ao verificar backup:', error);
      throw new Error(String(error));
    }
  }

  // Criar novo backup usando dialog nativo
  static async createBackup(): Promise<boolean> {
    try {
      // Abrir dialog para escolher local de salvamento
      const savePath = await invoke<string | null>('save_backup_dialog');
      
      if (!savePath) {
        console.log('⚠️ Criação de backup cancelada pelo usuário');
        return false;
      }
      
      console.log(`💾 Local selecionado para backup: ${savePath}`);
      
      // Chamar comando Tauri para criar backup
      await invoke<BackupInfo>('create_backup_command', {
        backup_path: savePath
      });
      
      console.log(`✅ Backup criado com sucesso: ${savePath}`);
      return true;
    } catch (error) {
      console.error('❌ Erro ao criar backup:', error);
      throw error;
    }
  }

  // Restaurar backup (com confirmação)
  static async restoreBackup(backupPath: string): Promise<boolean> {
    try {
      console.log(`🔄 Tentando restaurar backup de: ${backupPath}`);
      
      // Chamar comando Tauri para restaurar backup
      await invoke<void>('restore_backup_command', {
        backup_path: backupPath
      });
      
      console.log(`✅ Backup restaurado com sucesso de: ${backupPath}`);
      console.log('⚠️ Reinicie o aplicativo para carregar os dados restaurados');
      return true;
    } catch (error) {
      console.error('❌ Erro ao restaurar backup:', error);
      throw error;
    }
  }

  // Formatar tamanho de arquivo
  static formatSize(bytes: number): string {
    if (bytes < 1024) return bytes + ' B';
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
    if (bytes < 1024 * 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
    return (bytes / (1024 * 1024 * 1024)).toFixed(1) + ' GB';
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
        minute: '2-digit',
        second: '2-digit'
      });
    } catch {
      return dateStr;
    }
  }

  // Obter nome do arquivo do caminho
  static getFileName(path: string): string {
    return path.split(/[/\\]/).pop() || path;
  }
}

// ================================
// EXPORT PRINCIPAL
// ================================

export { BackupAPI };
