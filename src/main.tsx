import React from 'react'
import ReactDOM from 'react-dom/client'
import './index.css'

// Componente principal ARKIVE
function App() {
  return (
    <div className="min-h-screen bg-gray-900 text-white">
      {/* Header */}
      <div className="bg-gray-800 border-b border-gray-700 p-4">
        <h1 className="text-2xl font-bold text-blue-400">ARKIVE - Guardou, Achou!</h1>
        <p className="text-gray-400">Sistema de Gerenciamento de Documentos</p>
      </div>

      {/* Container principal */}
      <div className="container mx-auto p-6">
        {/* Status do sistema */}
        <div className="bg-gray-800 rounded-lg p-4 mb-6 border border-gray-700">
          <h2 className="text-lg font-semibold mb-2 text-blue-400">Status do Sistema</h2>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="text-center">
              <div className="text-2xl font-bold text-green-400">Online</div>
              <div className="text-sm text-gray-400">Sistema Web</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-yellow-400">0</div>
              <div className="text-sm text-gray-400">Documentos</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-purple-400">PostgreSQL</div>
              <div className="text-sm text-gray-400">Database</div>
            </div>
          </div>
        </div>

        {/* Área de upload */}
        <div className="bg-gray-800 rounded-lg p-6 border border-gray-700 mb-6">
          <h2 className="text-lg font-semibold mb-4 text-blue-400">Upload de Documentos</h2>
          <div className="border-2 border-dashed border-gray-600 rounded-lg p-8 text-center hover:border-blue-400 transition-colors">
            <div className="text-4xl mb-4">📁</div>
            <p className="text-gray-400 mb-2">Clique para selecionar ou arraste arquivos aqui</p>
            <p className="text-sm text-gray-500">Máximo 100MB por arquivo</p>
          </div>
        </div>

        {/* Barra de pesquisa estilo Google */}
        <div className="bg-gray-800 rounded-lg p-6 border border-gray-700 mb-6">
          <div className="max-w-2xl mx-auto">
            <div className="relative">
              <input
                type="text"
                placeholder="Digite aqui para localizar algo"
                className="w-full px-4 py-3 bg-gray-900 border border-gray-600 rounded-full text-white placeholder-gray-400 focus:outline-none focus:border-blue-400 focus:ring-1 focus:ring-blue-400"
              />
              <div className="absolute right-3 top-1/2 transform -y-1/2">
                <span className="text-gray-400">🔍</span>
              </div>
            </div>
          </div>
        </div>

        {/* Lista de documentos */}
        <div className="bg-gray-800 rounded-lg p-6 border border-gray-700">
          <h2 className="text-lg font-semibold mb-4 text-blue-400">Documentos Recentes</h2>
          <div className="text-center text-gray-400 py-8">
            <div className="text-4xl mb-4">📄</div>
            <p>Nenhum documento encontrado</p>
            <p className="text-sm text-gray-500">Faça upload de arquivos para começar</p>
          </div>
        </div>
      </div>
    </div>
  )
}

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
)
          