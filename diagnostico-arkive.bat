@echo off
title ARKIVE Instalado - Diagnostico Avancado
color 0E
chcp 65001 >nul

echo ========================================
echo    ARKIVE INSTALADO - DIAGNÓSTICO
echo ========================================
echo.

set "arkive_path=C:\Users\%USERNAME%\AppData\Local\ARKIVE\app.exe"

echo [1/8] Verificando instalação...
if exist "%arkive_path%" (
    echo ✓ ARKIVE encontrado em: %arkive_path%
    for %%I in ("%arkive_path%") do echo   Tamanho: %%~zI bytes
    for %%I in ("%arkive_path%") do echo   Data: %%~tI
) else (
    echo ✗ ARKIVE não encontrado no caminho esperado
    echo Procurando em outros locais...
    
    for /r "C:\Users\%USERNAME%\AppData" %%f in (app.exe) do (
        if exist "%%f" (
            echo Encontrado em: %%f
            set "arkive_path=%%f"
        )
    )
)

echo.
echo [2/8] Verificando permissões do arquivo...
icacls "%arkive_path%" | findstr /i "%USERNAME%" >nul
if %errorlevel% == 0 (
    echo ✓ Usuário tem permissões no arquivo
) else (
    echo ✗ Possível problema de permissões
    echo Corrigindo permissões...
    takeown /f "%arkive_path%" >nul 2>&1
    icacls "%arkive_path%" /grant "%USERNAME%":F >nul 2>&1
    echo ✓ Permissões corrigidas
)

echo.
echo [3/8] Verificando dependências críticas...

echo Verificando WebView2...
reg query "HKLM\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" >nul 2>&1
if %errorlevel% == 0 (
    for /f "tokens=3" %%a in ('reg query "HKLM\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" /v pv 2^>nul ^| findstr REG_SZ') do echo ✓ WebView2 versão: %%a
) else (
    echo ✗ WebView2 NÃO instalado - CRÍTICO
    echo   Baixe: https://go.microsoft.com/fwlink/p/?LinkId=2124703
)

echo Verificando Visual C++ 2022...
reg query "HKLM\SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64" /v Version >nul 2>&1
if %errorlevel% == 0 (
    for /f "tokens=3" %%a in ('reg query "HKLM\SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64" /v Version 2^>nul ^| findstr REG_SZ') do echo ✓ VC++ versão: %%a
) else (
    echo ✗ VC++ 2022 NÃO instalado - CRÍTICO
    echo   Baixe: https://aka.ms/vs/17/release/vc_redist.x64.exe
)

echo.
echo [4/8] Testando execução com logs...
echo Criando arquivo de teste...

echo @echo off > test_arkive.bat
echo echo Executando ARKIVE com logs completos... >> test_arkive.bat
echo cd /d "C:\Users\%USERNAME%\AppData\Local\ARKIVE" >> test_arkive.bat
echo echo Diretorio: %%CD%% >> test_arkive.bat
echo echo. >> test_arkive.bat
echo start /wait app.exe ^>^> arkive_log.txt 2^>^&1 >> test_arkive.bat
echo echo Codigo de saida: %%ERRORLEVEL%% ^>^> arkive_log.txt >> test_arkive.bat

echo Executando teste...
call test_arkive.bat

timeout /t 3 >nul

echo.
echo [5/8] Verificando processos em execução...
tasklist | findstr /i "app.exe" >nul
if %errorlevel% == 0 (
    echo ✓ Processo ARKIVE encontrado
    tasklist | findstr /i "app.exe"
) else (
    echo ✗ Processo ARKIVE não encontrado
)

echo.
echo [6/8] Analisando logs de erro...
set "log_path=C:\Users\%USERNAME%\AppData\Local\ARKIVE\arkive_log.txt"
if exist "%log_path%" (
    echo ✓ Log encontrado:
    echo ----------------------------------------
    type "%log_path%"
    echo ----------------------------------------
) else (
    echo ⚠️ Nenhum log gerado - aplicação falhou antes de iniciar
)

echo.
echo [7/8] Verificando Event Viewer...
echo Procurando erros recentes do ARKIVE...
powershell -command "Get-WinEvent -FilterHashtable @{LogName='Application'; Level=2} -MaxEvents 5 | Where-Object {$_.Message -like '*app.exe*' -or $_.Message -like '*ARKIVE*'} | Select-Object TimeCreated, Id, LevelDisplayName, Message | Format-List" 2>nul

echo.
echo [8/8] Diagnóstico de compatibilidade...
echo Sistema operacional:
ver

echo Arquitetura:
wmic os get osarchitecture /value | findstr "64"
if %errorlevel% == 0 (
    echo ✓ Sistema 64-bit compatível
) else (
    echo ✗ Possível incompatibilidade de arquitetura
)

echo.
echo ========================================
echo           RELATÓRIO FINAL
echo ========================================

if exist "%log_path%" (
    findstr /i "error\|exception\|failed\|denied" "%log_path%" >nul
    if %errorlevel% == 0 (
        echo ❌ ERRORS ENCONTRADOS NOS LOGS:
        findstr /i "error\|exception\|failed\|denied" "%log_path%"
    ) else (
        echo ✓ Nenhum erro crítico nos logs
    )
) else (
    echo ❌ PROBLEMA CRÍTICO: Aplicação não gera logs
    echo    Possíveis causas:
    echo    1. Falta WebView2 Runtime
    echo    2. Falta Visual C++ 2022
    echo    3. Antivírus bloqueando silenciosamente  
    echo    4. Corrupção do executável
)

echo.
echo PRÓXIMOS PASSOS RECOMENDADOS:
echo 1. Instalar WebView2 Runtime se não estiver presente
echo 2. Instalar Visual C++ 2022 Redistributable 
echo 3. Adicionar exceção no antivírus para toda pasta AppData\Local\ARKIVE
echo 4. Executar como Administrador uma vez
echo 5. Se nada funcionar, desinstalar e reinstalar

echo.
echo Pressione qualquer tecla para continuar...
pause >nul

del test_arkive.bat 2>nul
