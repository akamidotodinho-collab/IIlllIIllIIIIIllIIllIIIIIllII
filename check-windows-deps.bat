@echo off
echo =========================================
echo ARKIVE - Verificador de Dependencias Windows
echo =========================================
echo.

echo Verificando WebView2...
reg query "HKLM\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" >nul 2>&1
if %errorlevel%==0 (
    echo ✅ WebView2 encontrado
) else (
    echo ❌ WebView2 NAO encontrado
    echo    Solucao: winget install Microsoft.EdgeWebView2Runtime
)

echo.
echo Verificando Visual C++ Redistributable...
reg query "HKLM\SOFTWARE\WOW6432Node\Microsoft\VisualStudio\14.0\VC\Runtimes\x64" >nul 2>&1
if %errorlevel%==0 (
    echo ✅ Visual C++ Redistributable encontrado
) else (
    echo ❌ Visual C++ Redistributable NAO encontrado  
    echo    Solucao: winget install Microsoft.VCRedist.2015+.x64
)

echo.
echo Verificando permissoes de escrita...
echo teste > "%LOCALAPPDATA%\test_write.txt" 2>nul
if exist "%LOCALAPPDATA%\test_write.txt" (
    echo ✅ Permissoes de escrita OK
    del "%LOCALAPPDATA%\test_write.txt"
) else (
    echo ❌ Problema de permissoes
    echo    Solucao: Executar como administrador
)

echo.
echo Verificando SmartScreen (comum em novos executaveis)...
echo ⚠️  Se o Windows bloquear o ARKIVE:
echo    1. Clique em "Mais informacoes"
echo    2. Clique em "Executar assim mesmo"
echo    3. Ou: Propriedades do arquivo ^> Desbloquear

echo.
echo =========================================
echo Verificacao concluida!
echo =========================================
pause