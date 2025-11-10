# ARKIVE Desktop - Sistema de Gerenciamento de Documentos

## Overview

ARKIVE é um **sistema desktop nativo** de gerenciamento de documentos desenvolvido com Tauri 2.2, Rust e React. O foco é ser primariamente uma aplicação desktop com funcionalidades de upload via dialog nativo, processamento OCR offline, busca instantânea FTS5, e trilha de auditoria completa. A aplicação utiliza APIs nativas do Tauri para file system, dialogs e funcionalidades do sistema operacional. O modo web é disponibilizado apenas para preview quando solicitado.

## User Preferences

Preferred communication style: Simple, everyday language.

## Recent Changes (Nov 10, 2025)

### Funcionalidades Avançadas Implementadas

**1. Extração Automática de Data (NOVO)**
- ✅ Módulo `date_extractor.rs` (14.7KB) com regex PT-BR
- ✅ Suporta formatos: DD/MM/YYYY, DD-MM-YYYY, YYYY-MM-DD, DDMMYY, etc
- ✅ Parser de texto natural: "4 de outubro de 2025", "outubro 2025"
- ✅ Confidence scoring: filename (95%) → content (75%) → fallback (10%)
- ✅ Schema database atualizado: colunas `document_date` e `folder_slug`

**2. Pastas Virtuais Automáticas (NOVO)**
- ✅ Organização automática por YYYY-MM (ex: "2025-10")
- ✅ APIs: `getAvailableFolders()`, `getDocumentsByFolder()`, `getDocumentsByDateRange()`
- ✅ Índices otimizados para performance
- ✅ Migration idempotente (PRAGMA table_info)

**3. Upload em Lote Paralelo (NOVO)**
- ✅ Processamento de 40+ PDFs simultaneamente
- ✅ Batching inteligente (10 arquivos por vez)
- ✅ UI de progresso com barra visual (X de Y arquivos)
- ✅ Contagem de sucessos/erros em tempo real
- ✅ Continua processando mesmo se 1 arquivo falhar

**4. Busca Inteligente PT-BR (NOVO)**
- ✅ Módulo `date_search_parser.rs` (410 linhas)
- ✅ Detecta queries de data: "04/10/2025", "outubro 2025", "4 de outubro"
- ✅ Previne falsos positivos (ex: "setor" não detecta "setembro")
- ✅ Previne queries mistas (ex: "rastreabilidade outubro" vai para FTS5)
- ✅ Word boundaries (\b) para matching preciso
- ✅ Testes de regressão completos

**5. Remoção Completa de Dados Mockados**
- ✅ Removida função `get_recent_activities` com dados fictícios
- ✅ Removidas 220+ linhas de mock data em `auditApi.ts`
- ✅ 100% dados REAIS do banco SQLite

### Status do Projeto
- **Build Frontend:** ✅ Funcionando (Vite 7.0 + React 19.1)
- **Build Backend:** ✅ Rust compilando sem erros funcionais
- **Erros LSP:** 2 (apenas ícones, não afeta funcionalidade)
- **Pronto para Build Windows:** ✅ Sim
- **Funcionalidades Core:** Login, upload em lote, busca inteligente, auditoria, backup, pastas virtuais

### Arquivos Modificados Nesta Sessão
- `src-tauri/src/lib.rs` - Integração busca inteligente
- `src-tauri/src/database_sqlite.rs` - Migration idempotente + helpers
- `src-tauri/src/date_extractor.rs` - NOVO módulo (extração de data)
- `src-tauri/src/date_search_parser.rs` - NOVO módulo (busca por data)
- `src/SimpleApp.tsx` - Upload paralelo + UI de progresso
- `src/services/documentApi.ts` - APIs atualizadas
- `src/services/auditApi.ts` - Dados mockados removidos
- `tsconfig.json` - Lib ES2020 para Promise.allSettled

## System Architecture

### Frontend Architecture
- **Framework**: React 19.1+ com TypeScript para type safety
- **Styling**: Tailwind CSS para design responsivo e consistente
- **Build Tool**: Vite 7.0+ para desenvolvimento rápido e otimizações de build
- **Icons**: Lucide React para iconografia consistente
- **Routing**: Client-side navigation com estado reativo

### Backend Architecture
- **Runtime**: Tauri 2.2 framework para aplicações desktop nativas
- **Core Language**: Rust 1.89+ para performance e segurança de memória
- **Database**: SQLite com SQLx para ORM type-safe
- **Authentication**: bcrypt para hash seguro de senhas
- **File System**: Async file operations com isolamento por usuário

### Desktop Integration (NATIVO)
- **Native APIs**: Upload via dialog nativo, download com save-as, sem substitutos web
- **Window Management**: Configuração nativa com dimensões mínimas (900x600) e redimensionamento
- **Security**: CSP com protocolos Tauri (tauri:, asset:, ipc:) para comunicação nativa
- **Build Pipeline**: GitHub Actions automatizado para MSI/NSIS Windows
- **WebView2**: Integração nativa Windows com instalação automática
- **Icon System**: Multi-resolution icons (32x32, 128x128, ICO, ICNS)

### Data Storage Solutions
- **Primary Database**: SQLite para metadados de documentos e usuários
- **File Storage**: Sistema de arquivos local com organização por usuário
- **Backup System**: Backup automático integrado para recuperação de dados
- **Data Isolation**: Separação completa de dados entre usuários

### Authentication and Authorization
- **Password Security**: bcrypt hashing com salt para proteção de credenciais
- **User Isolation**: Sistema de multi-usuário com dados completamente isolados
- **Session Management**: Controle de sessão baseado em estado local
- **Access Control**: Validação de permissões a nível de arquivo e operação

### Build and Development
- **Desktop-First**: APIs nativas Tauri, sem fallbacks web para funcionalidades core
- **Development Server**: Hot reload com Vite + Tauri dev mode (Replit usa modo web apenas para preview)
- **Windows Build**: GitHub Actions com WebView2, VC++ Redist, e Tesseract OCR opcional
- **Production Build**: MSI/NSIS installers com dependências nativas bundled
- **Asset Management**: Bundling automático de recursos estáticos
- **TypeScript**: Type checking rigoroso em todo o codebase

### Performance Optimizations
- **Code Splitting**: Separação vendor/app chunks para cache eficiente
- **Lazy Loading**: Carregamento sob demanda de componentes pesados
- **Memory Management**: Rust ownership system previne vazamentos
- **File Operations**: I/O assíncrono para responsividade da UI

## External Dependencies

### Core Runtime Dependencies
- **@tauri-apps/api**: 2.7.0 - Interface JavaScript para comunicação com backend Rust
- **@tauri-apps/cli**: 2.7.1 - Tooling para build e desenvolvimento Tauri
- **React Ecosystem**: React 19.1+ e React DOM para UI reativa
- **TypeScript**: 5.8+ para type safety e melhor DX

### UI and Styling
- **Tailwind CSS**: Framework de utilidades para styling consistente
- **Lucide React**: Biblioteca de ícones SVG otimizados
- **CSS Custom Properties**: Theming system para modo claro/escuro

### Build and Development Tools
- **Vite**: 7.0+ bundler moderno com HMR e otimizações
- **@vitejs/plugin-react**: Plugin oficial React para Vite
- **ESBuild**: Minificador rápido para builds de produção

### Rust Backend Dependencies (via Cargo.toml)
- **SQLx**: ORM async type-safe para SQLite
- **bcrypt**: Library segura para hash de senhas
- **serde**: Serialização/deserialização JSON
- **tokio**: Runtime assíncrono para operações I/O
- **anyhow**: Error handling ergonômico

### System Integration
- **Native APIs**: Integração com sistema de arquivos do OS
- **Window Management**: APIs nativas para controle de janelas
- **File System Access**: Permissões seguras para leitura/escrita
- **Cross-platform Support**: Compatibilidade Windows/macOS/Linux