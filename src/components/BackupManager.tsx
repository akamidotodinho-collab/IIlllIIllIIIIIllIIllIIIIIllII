import React, { useState, useEffect, useCallback } from 'react';
import { 
  Shield, Download, Upload, CheckCircle, AlertTriangle, 
  RefreshCw, Trash2, FileArchive, Calendar, HardDrive, 
  Hash, AlertCircle, Database, Clock
} from 'lucide-react';
import { BackupAPI, BackupListItem, BackupInfo } from '../services/backupApi';
import { showSuccessToast, showErrorToast, showInfoToast } from '../utils/toast';

export default function BackupManager() {
  // Estados principais
  const [backups, setBackups] = useState<BackupListItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [refreshing, setRefreshing] = useState(false);
  const [creatingBackup, setCreatingBackup] = useState(false);
  const [verifyingBackup, setVerifyingBackup] = useState<string | null>(null);
  const [restoringBackup, setRestoringBackup] = useState<string | null>(null);
  
  // Estado para detalhes do backup verificado
  const [verifiedBackup, setVerifiedBackup] = useState<{ path: string; info: BackupInfo } | null>(null);

  // Carregar lista de backups
  const loadBackups = useCallback(async () => {
    setLoading(true);
    try {
      const backupList = await BackupAPI.listBackups();
      setBackups(backupList);
      console.log('✅ Backups carregados:', backupList.length);
    } catch (error) {
      console.error('❌ Erro ao carregar backups:', error);
      showErrorToast(`Erro ao carregar backups: ${String(error)}`);
    } finally {
      setLoading(false);
    }
  }, []);

  // Atualizar lista
  const handleRefresh = useCallback(async () => {
    setRefreshing(true);
    try {
      await loadBackups();
    } finally {
      setRefreshing(false);
    }
  }, [loadBackups]);

  // Criar novo backup
  const handleCreateBackup = useCallback(async () => {
    setCreatingBackup(true);
    try {
      const success = await BackupAPI.createBackup();
      
      if (success) {
        showSuccessToast('Backup criado com sucesso!');
        await loadBackups(); // Recarregar lista
      }
    } catch (error) {
      console.error('❌ Erro ao criar backup:', error);
      showErrorToast(`Erro ao criar backup: ${String(error)}`);
    } finally {
      setCreatingBackup(false);
    }
  }, [loadBackups]);

  // Verificar integridade do backup
  const handleVerifyBackup = useCallback(async (backupPath: string) => {
    setVerifyingBackup(backupPath);
    try {
      const info = await BackupAPI.verifyBackup(backupPath);
      setVerifiedBackup({ path: backupPath, info });
      showSuccessToast(`Backup verificado com sucesso! ${info.files_count} arquivo(s), ${BackupAPI.formatSize(info.database_size)}`);
    } catch (error) {
      console.error('❌ Erro ao verificar backup:', error);
      showErrorToast(`Erro ao verificar backup - pode estar corrompido`);
    } finally {
      setVerifyingBackup(null);
    }
  }, []);

  // Restaurar backup
  const handleRestoreBackup = useCallback(async (backupPath: string, fileName: string) => {
    const confirmed = window.confirm(
      `⚠️ ATENÇÃO!\n\nVocê está prestes a restaurar o backup:\n${fileName}\n\nEsta ação irá:\n- Substituir TODOS os dados atuais\n- Não pode ser desfeita\n\nTem certeza que deseja continuar?`
    );

    if (!confirmed) {
      return;
    }

    setRestoringBackup(backupPath);
    try {
      const success = await BackupAPI.restoreBackup(backupPath);
      
      if (success) {
        showSuccessToast('Backup restaurado com sucesso! Reiniciando aplicativo...');
        // Aqui você pode adicionar lógica para reiniciar o app ou recarregar dados
        setTimeout(() => window.location.reload(), 2000);
      }
    } catch (error) {
      console.error('❌ Erro ao restaurar backup:', error);
      showErrorToast(`Erro ao restaurar backup: ${String(error)}`);
    } finally {
      setRestoringBackup(null);
    }
  }, []);

  // Carregar backups ao montar componente
  useEffect(() => {
    loadBackups();
  }, [loadBackups]);

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-3">
            <Shield className="w-6 h-6 text-blue-500" />
            <div>
              <h2 className="text-xl font-bold text-gray-900 dark:text-white">
                Gerenciamento de Backups
              </h2>
              <p className="text-sm text-gray-600 dark:text-gray-400">
                {backups.length} backup(s) disponível(is)
              </p>
            </div>
          </div>

          <div className="flex items-center gap-2">
            <button
              onClick={handleRefresh}
              disabled={refreshing || loading}
              className="flex items-center gap-2 px-3 py-2 text-gray-600 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors disabled:opacity-50"
              title="Atualizar lista"
            >
              <RefreshCw className={`w-4 h-4 ${refreshing ? 'animate-spin' : ''}`} />
            </button>

            <button
              onClick={handleCreateBackup}
              disabled={creatingBackup}
              className="flex items-center gap-2 px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              {creatingBackup ? (
                <>
                  <RefreshCw className="w-4 h-4 animate-spin" />
                  Criando...
                </>
              ) : (
                <>
                  <Upload className="w-4 h-4" />
                  Criar Backup
                </>
              )}
            </button>
          </div>
        </div>

        {/* Informações importantes */}
        <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4">
          <div className="flex items-start gap-3">
            <AlertCircle className="w-5 h-5 text-blue-500 flex-shrink-0 mt-0.5" />
            <div className="text-sm text-blue-800 dark:text-blue-200">
              <p className="font-semibold mb-1">Importante:</p>
              <ul className="list-disc list-inside space-y-1">
                <li>Crie backups regularmente para proteger seus dados</li>
                <li>Guarde backups em local seguro (preferencialmente em outro dispositivo)</li>
                <li>Verifique a integridade dos backups antes de restaurar</li>
                <li>A restauração substitui TODOS os dados atuais</li>
              </ul>
            </div>
          </div>
        </div>
      </div>

      {/* Lista de Backups */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow">
        <div className="p-6 border-b border-gray-200 dark:border-gray-700">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
            Backups Disponíveis
          </h3>
        </div>

        <div className="p-6">
          {loading && (
            <div className="flex items-center justify-center py-12">
              <RefreshCw className="w-8 h-8 text-gray-400 animate-spin" />
            </div>
          )}

          {!loading && backups.length === 0 && (
            <div className="text-center py-12">
              <FileArchive className="w-16 h-16 text-gray-300 dark:text-gray-600 mx-auto mb-4" />
              <p className="text-gray-600 dark:text-gray-400 mb-2">
                Nenhum backup encontrado
              </p>
              <p className="text-sm text-gray-500 dark:text-gray-500">
                Clique em "Criar Backup" para fazer seu primeiro backup
              </p>
            </div>
          )}

          {!loading && backups.length > 0 && (
            <div className="space-y-4">
              {backups.map((backup, index) => {
                const fileName = BackupAPI.getFileName(backup.path);
                const isVerifying = verifyingBackup === backup.path;
                const isRestoring = restoringBackup === backup.path;
                const isVerified = verifiedBackup?.path === backup.path;

                return (
                  <div
                    key={backup.path}
                    className="border border-gray-200 dark:border-gray-700 rounded-lg p-4 hover:shadow-md transition-shadow"
                  >
                    <div className="flex items-start justify-between">
                      <div className="flex-1">
                        <div className="flex items-center gap-2 mb-2">
                          <FileArchive className="w-5 h-5 text-blue-500" />
                          <h4 className="font-semibold text-gray-900 dark:text-white">
                            {fileName}
                          </h4>
                          {isVerified && (
                            <span className="inline-flex items-center gap-1 px-2 py-1 bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200 text-xs font-medium rounded-full">
                              <CheckCircle className="w-3 h-3" />
                              Verificado
                            </span>
                          )}
                        </div>

                        <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm text-gray-600 dark:text-gray-400">
                          <div className="flex items-center gap-2">
                            <Calendar className="w-4 h-4" />
                            <span>{BackupAPI.formatDate(backup.info.created_at)}</span>
                          </div>
                          
                          <div className="flex items-center gap-2">
                            <Database className="w-4 h-4" />
                            <span>{BackupAPI.formatSize(backup.info.database_size)}</span>
                          </div>
                          
                          <div className="flex items-center gap-2">
                            <FileArchive className="w-4 h-4" />
                            <span>{backup.info.files_count} arquivo(s)</span>
                          </div>
                          
                          <div className="flex items-center gap-2">
                            <Hash className="w-4 h-4" />
                            <span className="font-mono text-xs">
                              {backup.info.checksum.slice(0, 8)}...
                            </span>
                          </div>
                        </div>

                        <div className="mt-2 text-xs text-gray-500 dark:text-gray-500">
                          Versão: {backup.info.version}
                        </div>
                      </div>

                      <div className="flex items-center gap-2 ml-4">
                        <button
                          onClick={() => handleVerifyBackup(backup.path)}
                          disabled={isVerifying || isRestoring}
                          className="flex items-center gap-2 px-3 py-2 text-sm bg-green-500 text-white rounded hover:bg-green-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                          title="Verificar integridade"
                        >
                          {isVerifying ? (
                            <RefreshCw className="w-4 h-4 animate-spin" />
                          ) : (
                            <CheckCircle className="w-4 h-4" />
                          )}
                          {isVerifying ? 'Verificando...' : 'Verificar'}
                        </button>

                        <button
                          onClick={() => handleRestoreBackup(backup.path, fileName)}
                          disabled={isVerifying || isRestoring}
                          className="flex items-center gap-2 px-3 py-2 text-sm bg-orange-500 text-white rounded hover:bg-orange-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                          title="Restaurar backup"
                        >
                          {isRestoring ? (
                            <RefreshCw className="w-4 h-4 animate-spin" />
                          ) : (
                            <Download className="w-4 h-4" />
                          )}
                          {isRestoring ? 'Restaurando...' : 'Restaurar'}
                        </button>
                      </div>
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </div>
      </div>

      {/* Informações de Segurança */}
      <div className="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-4">
        <div className="flex items-start gap-3">
          <AlertTriangle className="w-5 h-5 text-yellow-600 dark:text-yellow-500 flex-shrink-0 mt-0.5" />
          <div className="text-sm text-yellow-800 dark:text-yellow-200">
            <p className="font-semibold mb-1">Dicas de Segurança:</p>
            <ul className="list-disc list-inside space-y-1">
              <li>Mantenha pelo menos 3 backups recentes</li>
              <li>Armazene backups em nuvem ou disco externo</li>
              <li>Teste a restauração periodicamente</li>
              <li>Nunca delete todos os backups de uma vez</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  );
}
