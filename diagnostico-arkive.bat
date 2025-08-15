@echo off
title ARKIVE Desktop - Diagnostico Automatico
color 0E
chcp 65001 >nul

echo ========================================
echo       ARKIVE DESKTOP - DIAGNÓSTICO
echo ========================================
echo.

echo [1/6] Verificando arquivo executável...
set "exe_encontrado=0"
for %%F in (*.exe) do (
    if /I "%%~nF" NEQ "vc_redist.x64" (
        if /I "%%~nF" NEQ "MicrosoftEdgeWebview2Setup" (
            echo ✓ Executável encontrado: %%F
            echo   Tamanho: %%~zF bytes
            set "arkive_exe=%%F"
            set "exe_encontrado=1"
        )
    )
)

if %exe_encontrado% == 0 (
    echo ✗ Executável ARKIVE não encontrado
    echo   Certifique-se de ter extraído o arquivo ZIP
    pause
    exit /b 1
)

echo.
echo [2/6] Verificando WebView2 Runtime...
reg query "HKLM\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" >nul 2>&1
if %errorlevel% == 0 (
    echo ✓ WebView2 Runtime instalado
    set "webview2_ok=1"
) else (
    echo ✗ WebView2 Runtime NÃO encontrado
    set "webview2_ok=0"
)

echo.
echo [3/6] Verificando Visual C++ Redistributable...
reg query "HKLM\SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64" >nul 2>&1
if %errorlevel% == 0 (
    echo ✓ VC++ Redistributable instalado
    set "vcpp_ok=1"
) else (
    reg query "HKLM\SOFTWARE\WOW6432Node\Microsoft\VisualStudio\14.0\VC\Runtimes\x64" >nul 2>&1
    if %errorlevel% == 0 (
        echo ✓ VC++ Redistributable instalado
        set "vcpp_ok=1"
    ) else (
        echo ✗ VC++ Redistributable NÃO encontrado
        set "vcpp_ok=0"
    )
)

echo.
echo [4/6] Verificando Windows Defender...
powershell -command "Get-MpThreat" >nul 2>&1
if %errorlevel% == 0 (
    echo ⚠️ Windows Defender ativo - pode estar bloqueando
) else (
    echo ✓ Windows Defender não reportou ameaças
)

echo.
echo [5/6] Instalando dependências faltando...

if %webview2_ok% == 0 (
    if exist "MicrosoftEdgeWebview2Setup.exe" (
        echo Instalando WebView2 Runtime...
        MicrosoftEdgeWebview2Setup.exe /silent /install
        timeout /t 10 >nul
        echo ✓ WebView2 instalado
    ) else (
        echo ⚠️ MicrosoftEdgeWebview2Setup.exe não encontrado
    )
)

if %vcpp_ok% == 0 (
    if exist "vc_redist.x64.exe" (
        echo Instalando Visual C++ Redistributable...
        vc_redist.x64.exe /quiet /norestart
        timeout /t 5 >nul
        echo ✓ VC++ Redistributable instalado
    ) else (
        echo ⚠️ vc_redist.x64.exe não encontrado
    )
)

echo.
echo [6/6] Testando execução do ARKIVE...
echo Executando: %arkive_exe%

REM Criar exceção no Windows Defender
powershell -command "Add-MpPreference -ExclusionPath '%CD%\%arkive_exe%'" >nul 2>&1

echo Aguarde 3 segundos...
timeout /t 3 >nul

start "" "%arkive_exe%"

timeout /t 5 >nul

tasklist | findstr /i "ARKIVE" >nul
if %errorlevel% == 0 (
    echo ✓ ARKIVE executando com sucesso!
    echo ✓ Processo encontrado na lista de tarefas
) else (
    echo ✗ ARKIVE não iniciou corretamente
    echo.
    echo SOLUÇÕES ADICIONAIS:
    echo 1. Execute como Administrador (clique direito → Executar como admin)
    echo 2. Desative temporariamente o antivírus
    echo 3. Adicione exceção no Windows Defender manualmente
    echo 4. Verifique se o Windows está atualizado
)

echo.
echo ========================================
echo         DIAGNÓSTICO CONCLUÍDO
echo ========================================
echo.
echo Pressione qualquer tecla para sair...
pause >nul
