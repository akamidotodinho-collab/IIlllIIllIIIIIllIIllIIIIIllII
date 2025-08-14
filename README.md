# ARKIVE - Guardou, Achou! 📂

Sistema inteligente de gerenciamento de documentos desktop desenvolvido com Tauri, Rust e React.

## 🚀 Características

- **Interface Nativa**: Aplicação desktop com performance superior
- **Segurança**: Autenticação com bcrypt e isolamento de dados por usuário
- **Busca Inteligente**: Localização rápida de documentos
- **Preview de Arquivos**: Visualização de PDFs, imagens e textos
- **Backup Automático**: Sistema de backup integrado
- **Tema Escuro/Claro**: Interface adaptável

## 🛠️ Tecnologias

- **Frontend**: React + TypeScript + Tailwind CSS
- **Backend**: Rust + SQLite + SQLx
- **Desktop**: Tauri 2.2
- **Autenticação**: bcrypt
- **Build**: Vite + Cargo

## 📋 Pré-requisitos

- Rust 1.89+ ([instalar](https://rustup.rs/))
- Node.js 18+ ([instalar](https://nodejs.org/))
- Git ([instalar](https://git-scm.com/))

## 🔧 Instalação e Execução

### Download Rápido
```bash
git clone https://github.com/seu-usuario/arkive-desktop.git
cd arkive-desktop
npm install
npm run tauri build
```

### Desenvolvimento
```bash
# Instalar dependências
npm install

# Executar em modo desenvolvimento
npm run tauri dev

# Build de produção
npm run tauri build
```

## 📦 Releases Automáticos

Este projeto utiliza GitHub Actions para builds automáticos:

- **Push no main**: Build automático
- **Tags**: Release com executáveis
- **Pull Requests**: Verificação de build

### Download da Versão Estável
Acesse a [página de releases](https://github.com/seu-usuario/arkive-desktop/releases) para baixar o executável mais recente.

## 🏗️ Estrutura do Projeto

```
arkive-desktop/
├── src/                    # Frontend React
├── src-tauri/             # Backend Rust
│   ├── src/               # Código Rust
│   ├── Cargo.toml         # Dependências Rust
│   └── tauri.conf.json    # Configuração Tauri
├── .github/workflows/     # CI/CD GitHub Actions
├── package.json           # Dependências Node.js
└── README.md             # Este arquivo
```

## 🔐 Funcionalidades

### Autenticação
- Registro de usuários
- Login seguro com bcrypt
- Sessões isoladas por usuário

### Gerenciamento de Documentos
- Upload via drag-and-drop
- Categorização de arquivos
- Preview de PDFs e imagens
- Busca por nome e conteúdo

### Dashboard
- Estatísticas em tempo real
- Atividades recentes
- Métricas de uso

### Sistema de Backup
- Backup automático do banco
- Restauração de dados
- Histórico de backups

## 🚦 Status do Build

[![Build Status](https://github.com/seu-usuario/arkive-desktop/workflows/Build%20ARKIVE%20Desktop/badge.svg)](https://github.com/seu-usuario/arkive-desktop/actions)

## 📄 Licença

Este projeto está sob a licença MIT. Veja o arquivo [LICENSE](LICENSE) para mais detalhes.

## 🤝 Contribuindo

1. Fork o projeto
2. Crie uma branch para sua feature (`git checkout -b feature/AmazingFeature`)
3. Commit suas mudanças (`git commit -m 'Add some AmazingFeature'`)
4. Push para a branch (`git push origin feature/AmazingFeature`)
5. Abra um Pull Request

## 📞 Suporte

- 🐛 **Bugs**: [Issues](https://github.com/seu-usuario/arkive-desktop/issues)
- 💡 **Features**: [Discussions](https://github.com/seu-usuario/arkive-desktop/discussions)
- 📧 **Email**: seu-email@exemplo.com

---

**ARKIVE** - Desenvolvido com ❤️ usando tecnologias modernas