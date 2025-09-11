# ARKIVE Desktop - Sistema de Gerenciamento de Documentos

## Overview

ARKIVE é um sistema desktop inteligente de gerenciamento de documentos desenvolvido com Tauri 2.2, Rust e React. A aplicação oferece funcionalidades de upload, busca, preview e organização de arquivos com interface nativa, autenticação segura usando bcrypt, e sistema de backup integrado. O projeto utiliza SQLite para armazenamento local de dados e oferece temas claro/escuro para melhor experiência do usuário.

## User Preferences

Preferred communication style: Simple, everyday language.

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

### Desktop Integration
- **Window Management**: Configuração nativa com dimensões mínimas (900x600) e redimensionamento
- **Security**: CSP rigoroso e isolamento de contextos web/native
- **Build Targets**: MSI e NSIS installers para Windows
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
- **Development Server**: Hot reload com Vite + Tauri dev mode
- **Production Build**: Otimização automática com minificação e tree-shaking
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