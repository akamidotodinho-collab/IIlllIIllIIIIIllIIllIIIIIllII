@echo off
title ARKIVE - Teste com Logs Detalhados
color 0E
chcp 65001 >nul

echo ========================================
echo    ARKIVE - TESTE COM LOGS DETALHADOS
echo ========================================
echo.

echo Procurando executável ARKIVE...
set "exe_path="
for %%F in (*.exe) do (
    if /I "%%~nF" NEQ "vc_redist.x64" (
        if /I "%%~nF" NEQ "MicrosoftEdgeWebview2Setup" (
            set "exe_path=%%F"
            echo ✓ Executável encontrado: %%F
            goto :found_exe
        )
    )
)

echo ✗ Executável ARKIVE não encontrado nesta pasta
echo   Certifique-se de estar na pasta correta
pause
exit /b 1

:found_exe
echo.
echo Configurando logs detalhados...
set RUST_LOG=debug
set RUST_BACKTRACE=full

echo.
echo ========================================
echo EXECUTANDO ARKIVE COM LOGS...
echo ========================================
echo.
echo Arquivo de log: arkive-debug.log
echo.

echo Iniciando ARKIVE... (aguarde)
"%exe_path%" > arkive-debug.log 2>&1

echo.
echo ========================================
echo           RESULTADO DO TESTE
echo ========================================

if exist arkive-debug.log (
    echo ✓ Log gerado com sucesso
    echo.
    echo CONTEÚDO DO LOG:
    echo ----------------------------------------
    type arkive-debug.log
    echo ----------------------------------------
    echo.
    echo Arquivo salvo como: arkive-debug.log
) else (
    echo ✗ Nenhum log foi gerado
    echo   Possível crash antes da inicialização
)

echo.
echo Verificando se ARKIVE está executando...
tasklist | findstr /i "ARKIVE" >nul
if %errorlevel% == 0 (
    echo ✓ ARKIVE está em execução
    tasklist | findstr /i "ARKIVE"
) else (
    echo ✗ ARKIVE não está executando
)

echo.
echo Pressione qualquer tecla para continuar...
pause >nul
