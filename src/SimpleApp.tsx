import { useState, useEffect } from 'react';
import { Search, Upload, FileText, Image, Download, Trash2, Plus, Shield, Home, User, LogOut } from 'lucide-react';
import AuditTrail from './components/AuditTrail';
import SearchInterface from './components/SearchInterface';
import { AuthAPI, DocumentAPI, StatsAPI, AppAPI, type Document as APIDocument, type User as APIUser, type AppStats } from './services/documentApi';

// Interface local para compatibilidade
interface Document {
  id: string;
  name: string;
  size: number;
  type: string;
  date: string;
}

export default function SimpleApp() {
  // Estados para dados reais
  const [documents, setDocuments] = useState<Document[]>([]);
  const [searchTerm, setSearchTerm] = useState('');
  const [user, setUser] = useState<APIUser | null>(null);
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [stats, setStats] = useState<AppStats | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  
  // Estados para UI
  const [activeTab, setActiveTab] = useState<'documents' | 'search' | 'audit'>('documents');
  const [loginForm, setLoginForm] = useState({ username: '', password: '' });
  const [showLogin, setShowLogin] = useState(false);

  // Inicializar app - verificar usuário autenticado
  useEffect(() => {
    initializeApp();
  }, []);

  const initializeApp = async () => {
    try {
      setIsLoading(true);
      
      // Verificar se usuário está autenticado
      const currentUser = await AuthAPI.getCurrentUser();
      if (currentUser) {
        setUser(currentUser);
        setIsAuthenticated(true);
        await loadAppData();
      } else {
        setShowLogin(true);
      }
    } catch (error) {
      console.warn('Erro ao inicializar app:', error);
      setShowLogin(true);
    } finally {
      setIsLoading(false);
    }
  };

  // Carregar dados do app
  const loadAppData = async () => {
    try {
      const [docsResult, statsResult] = await Promise.all([
        DocumentAPI.getDocuments(),
        StatsAPI.getStats()
      ]);
      
      // Converter APIDocument para Document local
      const localDocs: Document[] = docsResult.map(doc => ({
        id: doc.id,
        name: doc.name,
        size: doc.file_size,
        type: doc.file_type,
        date: AppAPI.formatDate(doc.created_at)
      }));
      
      setDocuments(localDocs);
      setStats(statsResult);
      
      console.log('✅ Dados carregados:', localDocs.length, 'documentos');
    } catch (error) {
      console.error('❌ Erro ao carregar dados:', error);
      AppAPI.showError('Erro ao carregar dados do sistema');
    }
  };

  // Login do usuário
  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!loginForm.username || !loginForm.password) {
      AppAPI.showError('Preencha usuário e senha');
      return;
    }
    
    try {
      setIsLoading(true);
      const loggedUser = await AuthAPI.login(loginForm.username, loginForm.password);
      setUser(loggedUser);
      setIsAuthenticated(true);
      setShowLogin(false);
      setLoginForm({ username: '', password: '' });
      
      AppAPI.showSuccess(`Bem-vindo, ${loggedUser.username}!`);
      await loadAppData();
    } catch (error) {
      AppAPI.showError('Credenciais inválidas');
      console.error('Erro no login:', error);
    } finally {
      setIsLoading(false);
    }
  };

  // Logout do usuário
  const handleLogout = async () => {
    try {
      await AuthAPI.logout();
      setUser(null);
      setIsAuthenticated(false);
      setDocuments([]);
      setStats(null);
      setShowLogin(true);
      AppAPI.showSuccess('Logout realizado com sucesso');
    } catch (error) {
      console.error('Erro no logout:', error);
    }
  };

  // Filtrar documentos por busca (busca simples local)
  const filteredDocs = documents.filter(doc => 
    doc.name.toLowerCase().includes(searchTerm.toLowerCase())
  );

  // Upload de arquivo (versão simplificada)
  const handleUpload = () => {
    const input = document.createElement('input');
    input.type = 'file';
    input.multiple = true;
    input.accept = '.pdf,.jpg,.png,.gif,.doc,.docx,.xls,.xlsx';
    
    input.onchange = (e) => {
      const files = (e.target as HTMLInputElement).files;
      if (!files) return;

      Array.from(files).forEach(file => {
        const newDoc: Document = {
          id: Date.now().toString() + Math.random().toString(36).substr(2, 9),
          name: file.name,
          size: file.size,
          type: file.type || 'application/octet-stream',
          date: new Date().toLocaleString('pt-BR')
        };
        
        setDocuments(prev => [newDoc, ...prev]);
      });
    };
    
    input.click();
  };

  // Remover documento
  const handleDelete = (id: string) => {
    if (confirm('Remover este documento?')) {
      setDocuments(prev => prev.filter(doc => doc.id !== id));
    }
  };

  // Formatar tamanho do arquivo
  const formatSize = (bytes: number) => {
    if (bytes < 1024) return bytes + ' B';
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
    return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
  };

  // Ícone por tipo de arquivo
  const getFileIcon = (type: string) => {
    if (type.includes('image')) return <Image className="w-5 h-5 text-blue-500" />;
    return <FileText className="w-5 h-5 text-gray-500" />;
  };

  // Tela de login
  if (showLogin || !isAuthenticated) {
    return (
      <div className="min-h-screen bg-gray-50 dark:bg-gray-900 flex items-center justify-center">
        <div className="bg-white dark:bg-gray-800 p-8 rounded-lg shadow-lg w-full max-w-md">
          <div className="text-center mb-6">
            <Shield className="w-12 h-12 text-blue-600 mx-auto mb-4" />
            <h1 className="text-2xl font-bold text-gray-900 dark:text-white">ARKIVE Desktop</h1>
            <p className="text-gray-600 dark:text-gray-400">Sistema de Gerenciamento de Documentos</p>
          </div>
          
          <form onSubmit={handleLogin} className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                Usuário
              </label>
              <input
                type="text"
                value={loginForm.username}
                onChange={(e) => setLoginForm(prev => ({ ...prev, username: e.target.value }))}
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg 
                         bg-white dark:bg-gray-700 text-gray-900 dark:text-white
                         focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                placeholder="seu@email.com"
                disabled={isLoading}
                required
              />
            </div>
            
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                Senha
              </label>
              <input
                type="password"
                value={loginForm.password}
                onChange={(e) => setLoginForm(prev => ({ ...prev, password: e.target.value }))}
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg 
                         bg-white dark:bg-gray-700 text-gray-900 dark:text-white
                         focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                placeholder="••••••••"
                disabled={isLoading}
                required
              />
            </div>
            
            <button
              type="submit"
              disabled={isLoading}
              className="w-full bg-blue-600 hover:bg-blue-700 text-white font-medium py-2 px-4 rounded-lg
                       disabled:opacity-50 disabled:cursor-not-allowed transition-colors
                       flex items-center justify-center gap-2"
            >
              {isLoading ? (
                <>
                  <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
                  Entrando...
                </>
              ) : (
                'Entrar'
              )}
            </button>
            
            <p className="text-sm text-center text-gray-600 dark:text-gray-400 mt-4">
              Primeiro acesso? O sistema criará sua conta automaticamente.
            </p>
          </form>
        </div>
      </div>
    );
  }

  // Loading inicial
  if (isLoading) {
    return (
      <div className="min-h-screen bg-gray-50 dark:bg-gray-900 flex items-center justify-center">
        <div className="text-center">
          <div className="w-8 h-8 border-4 border-blue-600 border-t-transparent rounded-full animate-spin mx-auto mb-4"></div>
          <p className="text-gray-600 dark:text-gray-400">Carregando ARKIVE...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      {/* Header */}
      <header className="bg-white dark:bg-gray-800 shadow-sm border-b">
        <div className="px-6 py-4 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <Shield className="w-8 h-8 text-blue-600" />
            <div>
              <h1 className="text-xl font-bold text-gray-900 dark:text-white">ARKIVE Desktop</h1>
              <p className="text-sm text-gray-600 dark:text-gray-400">Sistema Empresarial de Documentos</p>
            </div>
          </div>
          
          <div className="flex items-center gap-4">
            {stats && (
              <div className="hidden md:flex items-center gap-6 text-sm text-gray-600 dark:text-gray-400">
                <span>{stats.total_documents} docs</span>
                <span>{Math.round(stats.total_storage / 1024 / 1024)} MB</span>
              </div>
            )}
            
            <div className="flex items-center gap-3">
              <div className="flex items-center gap-2">
                <User className="w-4 h-4 text-gray-500" />
                <span className="text-sm text-gray-700 dark:text-gray-300">
                  {user?.username || 'Usuário'}
                </span>
              </div>
              
              <button
                onClick={handleLogout}
                className="p-2 text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700"
                title="Sair"
              >
                <LogOut className="w-4 h-4" />
              </button>
            </div>
          </div>
        </div>
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-2xl font-bold text-gray-900 dark:text-white">
                ARKIVE
              </h1>
              <p className="text-sm text-gray-600 dark:text-gray-400">
                Guardou, Achou! - {user}
              </p>
            </div>
            
            <div className="flex items-center gap-4">
              <button 
                onClick={() => setActiveTab('documents')}
                className={`flex items-center gap-2 px-4 py-2 rounded-lg transition-colors ${
                  activeTab === 'documents' 
                    ? 'bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-200' 
                    : 'hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-600 dark:text-gray-400'
                }`}
              >
                <Home className="w-4 h-4" />
                Documentos
              </button>
              
              <button 
                onClick={() => setActiveTab('audit')}
                className={`flex items-center gap-2 px-4 py-2 rounded-lg transition-colors ${
                  activeTab === 'audit' 
                    ? 'bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-200' 
                    : 'hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-600 dark:text-gray-400'
                }`}
              >
                <Shield className="w-4 h-4" />
                Auditoria
              </button>
              
              {activeTab === 'documents' && (
                <button
                  onClick={handleUpload}
                  className="flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
                >
                  <Plus className="w-4 h-4" />
                  Adicionar
                </button>
              )}
            </div>
          </div>
          
          {/* Busca - apenas para documentos */}
          {activeTab === 'documents' && (
            <div className="mt-4 relative max-w-md">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" />
              <input
                type="text"
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                placeholder="Digite aqui para localizar algo..."
                className="w-full pl-10 pr-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              />
            </div>
          )}
        </div>
      </header>

      {/* Conteúdo */}
      <main className="p-6">
        {activeTab === 'audit' ? (
          <AuditTrail />
        ) : (
          <>
            {documents.length === 0 ? (
          <div className="text-center py-12">
            <Upload className="w-16 h-16 text-gray-400 mx-auto mb-4" />
            <h3 className="text-xl font-medium text-gray-900 dark:text-white mb-2">
              Nenhum documento ainda
            </h3>
            <p className="text-gray-600 dark:text-gray-400 mb-6">
              Comece adicionando seus primeiros arquivos
            </p>
            <button
              onClick={handleUpload}
              className="px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
            >
              Adicionar Documentos
            </button>
          </div>
        ) : (
          <div className="space-y-3">
            {filteredDocs.map(doc => (
              <div
                key={doc.id}
                className="flex items-center justify-between p-4 bg-white dark:bg-gray-800 rounded-lg shadow-sm border dark:border-gray-700"
              >
                <div className="flex items-center gap-3">
                  {getFileIcon(doc.type)}
                  <div>
                    <h4 className="font-medium text-gray-900 dark:text-white">
                      {doc.name}
                    </h4>
                    <p className="text-sm text-gray-600 dark:text-gray-400">
                      {formatSize(doc.size)} • {doc.date}
                    </p>
                  </div>
                </div>
                
                <div className="flex items-center gap-2">
                  <button
                    className="p-2 text-gray-600 dark:text-gray-400 hover:text-blue-600 dark:hover:text-blue-400"
                    title="Download"
                  >
                    <Download className="w-4 h-4" />
                  </button>
                  <button
                    onClick={() => handleDelete(doc.id)}
                    className="p-2 text-gray-600 dark:text-gray-400 hover:text-red-600 dark:hover:text-red-400"
                    title="Remover"
                  >
                    <Trash2 className="w-4 h-4" />
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}

            {searchTerm && filteredDocs.length === 0 && documents.length > 0 && (
              <div className="text-center py-8">
                <p className="text-gray-600 dark:text-gray-400">
                  Nenhum documento encontrado para "{searchTerm}"
                </p>
              </div>
            )}
          </>
        )}
      </main>
    </div>
  );
}