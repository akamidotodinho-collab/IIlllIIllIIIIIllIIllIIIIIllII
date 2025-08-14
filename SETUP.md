# Configuração do Repositório GitHub

Este guia orienta como configurar o repositório GitHub com CI/CD automático.

## 🚀 Configuração Automática

### 1. Criar Repositório no GitHub

1. Acesse [GitHub](https://github.com)
2. Clique em "New repository"
3. Nome: `arkive-desktop`
4. Descrição: `Sistema inteligente de gerenciamento de documentos desktop`
5. Marque "Add a README file"
6. Escolha licença: MIT
7. Clique "Create repository"

### 2. Upload do Código

```bash
# Clonar repositório vazio
git clone https://github.com/SEU-USUARIO/arkive-desktop.git
cd arkive-desktop

# Copiar arquivos do projeto (substitua CAMINHO pelo local dos arquivos)
cp -r CAMINHO/src .
cp -r CAMINHO/src-tauri .
cp -r CAMINHO/.github .
cp CAMINHO/.gitignore .
cp CAMINHO/README.md .
cp CAMINHO/CONTRIBUTING.md .
cp CAMINHO/LICENSE .
cp CAMINHO/tsconfig.json .
cp CAMINHO/vite.config.js .
cp CAMINHO/index.html .

# Commit inicial
git add .
git commit -m "feat: initial commit with complete ARKIVE desktop app"
git push origin main
```

### 3. Configurar GitHub Actions

Os workflows já estão configurados em `.github/workflows/build.yml` e serão executados automaticamente.

## 🔧 Funcionalidades Automáticas

### Build Automático
- **Trigger**: Push ou Pull Request
- **Ambiente**: Windows Latest + Rust 1.89 + Node 18
- **Output**: Executável .exe compilado

### Release Automático
- **Trigger**: Tags (ex: `v1.0.0`)
- **Processo**: Build + Upload para GitHub Releases
- **Download**: Executáveis disponíveis publicamente

### Status Badge
Badge no README mostra status do build em tempo real.

## 📦 Como Fazer Release

```bash
# Criar tag de versão
git tag v1.0.0
git push origin v1.0.0

# GitHub Actions automaticamente:
# 1. Compila o projeto
# 2. Cria release
# 3. Anexa executáveis
```

## 🛠️ Comandos Úteis

```bash
# Desenvolvimento local
npm run tauri:dev

# Build local
npm run tauri:build

# Verificar status
git status

# Ver histórico
git log --oneline

# Criar branch
git checkout -b feature/nova-funcionalidade
```

## 🚦 Monitoramento

### GitHub Actions
- Acesse: `https://github.com/SEU-USUARIO/arkive-desktop/actions`
- Veja builds em tempo real
- Downloads de artifacts

### Releases
- Acesse: `https://github.com/SEU-USUARIO/arkive-desktop/releases`
- Baixe executáveis prontos
- Veja notas de versão

## 🔗 Links Importantes

- **Repositório**: `https://github.com/SEU-USUARIO/arkive-desktop`
- **Issues**: `https://github.com/SEU-USUARIO/arkive-desktop/issues`
- **Actions**: `https://github.com/SEU-USUARIO/arkive-desktop/actions`
- **Releases**: `https://github.com/SEU-USUARIO/arkive-desktop/releases`

## 📞 Próximos Passos

1. ✅ Configurar repositório
2. ✅ Fazer upload do código
3. ✅ Verificar primeiro build
4. ✅ Fazer primeiro release
5. ✅ Compartilhar link para download

O repositório estará 100% profissional e automatizado!