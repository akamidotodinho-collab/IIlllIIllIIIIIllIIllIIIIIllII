import { useState, useEffect } from 'react';
import { Search, Upload, FileText, Image, Download, Trash2, Plus } from 'lucide-react';

interface Document {
  id: string;
  name: string;
  size: number;
  type: string;
  date: string;
}

export default function SimpleApp() {
  const [documents, setDocuments] = useState<Document[]>([]);
  const [searchTerm, setSearchTerm] = useState('');
  const [user, setUser] = useState('Usuário Local');

  // Filtrar documentos por busca
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

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      {/* Header */}
      <header className="bg-white dark:bg-gray-800 shadow-sm border-b">
        <div className="px-6 py-4">
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
              <div className="text-sm text-gray-600 dark:text-gray-400">
                {documents.length} documentos
              </div>
              
              <button
                onClick={handleUpload}
                className="flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
              >
                <Plus className="w-4 h-4" />
                Adicionar
              </button>
            </div>
          </div>
          
          {/* Busca */}
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
        </div>
      </header>

      {/* Conteúdo */}
      <main className="p-6">
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
      </main>
    </div>
  );
}