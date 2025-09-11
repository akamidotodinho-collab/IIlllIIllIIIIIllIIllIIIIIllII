import React from 'react';
import { X, Copy, Shield, Clock, User, FileText, Eye, EyeOff, Hash } from 'lucide-react';
import { AuditLog } from '../services/auditApi';

interface AuditLogDrawerProps {
  log: AuditLog | null;
  isOpen: boolean;
  onClose: () => void;
}

export default function AuditLogDrawer({ log, isOpen, onClose }: AuditLogDrawerProps) {
  if (!isOpen || !log) return null;

  const copyToClipboard = async (text: string, label: string) => {
    try {
      await navigator.clipboard.writeText(text);
      // Aqui você pode adicionar uma notificação de sucesso
      console.log(`${label} copiado para área de transferência`);
    } catch (err) {
      console.error('Erro ao copiar:', err);
    }
  };

  const getActionIcon = (action: string) => {
    switch (action.toUpperCase()) {
      case 'LOGIN':
        return <User className="w-5 h-5 text-green-500" />;
      case 'LOGIN_FAILED':
        return <User className="w-5 h-5 text-red-500" />;
      case 'UPLOAD':
        return <FileText className="w-5 h-5 text-blue-500" />;
      case 'DOWNLOAD':
        return <FileText className="w-5 h-5 text-purple-500" />;
      case 'VIEW':
        return <Eye className="w-5 h-5 text-gray-500" />;
      case 'EXPORT_AUDIT':
        return <Shield className="w-5 h-5 text-orange-500" />;
      default:
        return <Clock className="w-5 h-5 text-gray-400" />;
    }
  };

  const getSuccessColor = (isSuccess: boolean) => {
    return isSuccess 
      ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200'
      : 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200';
  };

  const formatMetadata = (metadata: string) => {
    try {
      const parsed = JSON.parse(metadata);
      return JSON.stringify(parsed, null, 2);
    } catch {
      return metadata;
    }
  };

  const CopyableField = ({ label, value, isSensitive = false }: { 
    label: string; 
    value: string; 
    isSensitive?: boolean; 
  }) => {
    const [isVisible, setIsVisible] = React.useState(!isSensitive);
    
    return (
      <div className="space-y-2">
        <div className="flex items-center justify-between">
          <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
            {label}
          </label>
          <div className="flex items-center gap-2">
            {isSensitive && (
              <button
                onClick={() => setIsVisible(!isVisible)}
                className="p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                title={isVisible ? 'Ocultar' : 'Mostrar'}
              >
                {isVisible ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
              </button>
            )}
            <button
              onClick={() => copyToClipboard(value, label)}
              className="p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
              title="Copiar"
            >
              <Copy className="w-4 h-4" />
            </button>
          </div>
        </div>
        <div className="p-3 bg-gray-50 dark:bg-gray-800 rounded border font-mono text-sm break-all">
          {isVisible ? value : '••••••••••••••••••••••••••••••••••••••••'}
        </div>
      </div>
    );
  };

  return (
    <>
      {/* Overlay */}
      <div 
        className="fixed inset-0 bg-black bg-opacity-50 z-40"
        onClick={onClose}
      />
      
      {/* Drawer */}
      <div className="fixed right-0 top-0 h-full w-full max-w-2xl bg-white dark:bg-gray-900 shadow-xl z-50 overflow-y-auto">
        {/* Header */}
        <div className="sticky top-0 bg-white dark:bg-gray-900 border-b border-gray-200 dark:border-gray-700 p-6">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              {getActionIcon(log.action)}
              <div>
                <h2 className="text-xl font-bold text-gray-900 dark:text-white">
                  Detalhes do Log de Auditoria
                </h2>
                <p className="text-sm text-gray-600 dark:text-gray-400">
                  ID: {log.id}
                </p>
              </div>
            </div>
            <button
              onClick={onClose}
              className="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800"
            >
              <X className="w-6 h-6" />
            </button>
          </div>
        </div>

        {/* Content */}
        <div className="p-6 space-y-6">
          {/* Status e Ação */}
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                Status
              </label>
              <span className={`inline-flex px-3 py-1 text-sm font-medium rounded-full ${getSuccessColor(log.is_success)}`}>
                {log.is_success ? 'Sucesso' : 'Falha'}
              </span>
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                Ação
              </label>
              <div className="flex items-center gap-2">
                {getActionIcon(log.action)}
                <span className="font-medium text-gray-900 dark:text-white">
                  {log.action}
                </span>
              </div>
            </div>
          </div>

          {/* Informações do Usuário */}
          <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-4 space-y-4">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white flex items-center gap-2">
              <User className="w-5 h-5" />
              Informações do Usuário
            </h3>
            
            <div className="grid grid-cols-2 gap-4">
              <CopyableField label="ID do Usuário" value={log.user_id} />
              <CopyableField label="Nome de Usuário" value={log.username} />
            </div>
            
            {log.ip_address && (
              <CopyableField label="Endereço IP" value={log.ip_address} />
            )}
          </div>

          {/* Informações do Recurso */}
          <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-4 space-y-4">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white flex items-center gap-2">
              <FileText className="w-5 h-5" />
              Informações do Recurso
            </h3>
            
            <div className="grid grid-cols-2 gap-4">
              <CopyableField label="Tipo" value={log.resource_type} />
              {log.resource_id && (
                <CopyableField label="ID do Recurso" value={log.resource_id} />
              )}
            </div>
            
            {log.resource_name && (
              <CopyableField label="Nome do Recurso" value={log.resource_name} />
            )}
          </div>

          {/* Informações Criptográficas */}
          <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-4 space-y-4">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white flex items-center gap-2">
              <Hash className="w-5 h-5" />
              Integridade Criptográfica
            </h3>
            
            <CopyableField 
              label="Hash da Cadeia" 
              value={log.current_hash} 
              isSensitive={true}
            />
            
            {log.file_hash && (
              <CopyableField 
                label="Hash do Arquivo" 
                value={log.file_hash} 
                isSensitive={true}
              />
            )}
          </div>

          {/* Timestamp */}
          <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-4">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white flex items-center gap-2 mb-4">
              <Clock className="w-5 h-5" />
              Timestamp
            </h3>
            <CopyableField label="Data e Hora" value={log.timestamp} />
          </div>

          {/* Metadados */}
          {log.metadata && log.metadata !== '{}' && (
            <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-4">
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
                Metadados Adicionais
              </h3>
              <div className="space-y-2">
                <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
                  JSON Formatado
                </label>
                <div className="p-3 bg-gray-100 dark:bg-gray-900 rounded border">
                  <pre className="text-xs font-mono text-gray-800 dark:text-gray-200 whitespace-pre-wrap break-words">
                    {formatMetadata(log.metadata)}
                  </pre>
                </div>
                <button
                  onClick={() => copyToClipboard(formatMetadata(log.metadata), 'Metadados')}
                  className="flex items-center gap-2 text-sm text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-300"
                >
                  <Copy className="w-4 h-4" />
                  Copiar Metadados
                </button>
              </div>
            </div>
          )}

          {/* Verificação de Integridade */}
          <div className="bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg p-4">
            <div className="flex items-center gap-2 mb-2">
              <Shield className="w-5 h-5 text-green-600 dark:text-green-400" />
              <span className="font-semibold text-green-800 dark:text-green-200">
                Log Verificado
              </span>
            </div>
            <p className="text-sm text-green-700 dark:text-green-300">
              Este log foi verificado criptograficamente e faz parte de uma cadeia íntegra. 
              Qualquer alteração seria detectada pelo sistema de auditoria.
            </p>
          </div>
        </div>
      </div>
    </>
  );
}