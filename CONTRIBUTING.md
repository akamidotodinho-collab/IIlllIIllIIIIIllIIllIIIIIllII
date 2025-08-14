# Contribuindo para o ARKIVE

Obrigado pelo interesse em contribuir! Este documento orienta como participar do desenvolvimento.

## 🚀 Como Começar

### 1. Fork e Clone
```bash
git fork https://github.com/seu-usuario/arkive-desktop
git clone https://github.com/SEU-USUARIO/arkive-desktop.git
cd arkive-desktop
```

### 2. Configurar Ambiente
```bash
# Instalar dependências
npm install

# Verificar se tudo funciona
npm run tauri dev
```

## 📋 Processo de Contribuição

### 1. Criar Issue
- Descreva o problema ou feature
- Use os templates disponíveis
- Aguarde feedback antes de implementar

### 2. Desenvolver
```bash
# Criar branch
git checkout -b feature/nova-funcionalidade

# Desenvolver e testar
npm run tauri dev

# Verificar build
npm run tauri build
```

### 3. Pull Request
- Descreva as mudanças claramente
- Inclua screenshots se aplicável
- Garanta que os testes passam
- Siga o padrão de código

## 🎯 Diretrizes de Código

### Rust (Backend)
- Use `rustfmt` para formatação
- Documente funções públicas
- Trate erros adequadamente
- Prefira `Result<T, E>` a panics

### TypeScript/React (Frontend)
- Use TypeScript estrito
- Componentes funcionais com hooks
- Props tipadas com interfaces
- Nomes descritivos para variáveis

### Commits
```
tipo(escopo): descrição breve

Descrição mais detalhada se necessário

Fixes #123
```

Tipos: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

## 🧪 Testes

### Rust
```bash
cd src-tauri
cargo test
```

### Frontend
```bash
npm test
```

### Build Completo
```bash
npm run tauri build
```

## 📦 Releases

O projeto usa versionamento semântico (SemVer):

- **MAJOR**: Mudanças incompatíveis
- **MINOR**: Funcionalidades compatíveis
- **PATCH**: Correções de bugs

## 🔍 Code Review

Todos os PRs passam por review:

- **Funcionalidade**: Feature funciona conforme esperado
- **Código**: Qualidade, performance e manutenibilidade
- **Testes**: Cobertura adequada
- **Documentação**: Atualizada conforme necessário

## 🐛 Reportando Bugs

Use o template de issue com:

- **Ambiente**: OS, versão do Rust, Node.js
- **Passos**: Como reproduzir
- **Esperado**: Comportamento esperado
- **Atual**: O que realmente acontece
- **Logs**: Mensagens de erro relevantes

## 💡 Sugerindo Features

Para novas funcionalidades:

- Descreva o problema que resolve
- Explique a solução proposta
- Considere alternativas
- Discuta impacto na performance

## 🏷️ Labels

- `bug`: Algo não funciona
- `enhancement`: Nova funcionalidade
- `documentation`: Melhorias na documentação
- `good first issue`: Bom para iniciantes
- `help wanted`: Precisa de ajuda externa

## 📞 Comunidade

- **GitHub Discussions**: Perguntas gerais
- **Issues**: Bugs e features específicas
- **Email**: Contato direto com mantenedores

Obrigado por contribuir! 🎉