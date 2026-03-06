@echo off
setlocal enabledelayedexpansion

set "PROJECT_DIR=%~dp0"
set "CORE_DIR=%PROJECT_DIR%core"
set "UI_DIR=%PROJECT_DIR%ui"
set "ROUTER_PORT=3000"
set "UI_PORT=8083"
set "LLAMA_PORT=8080"
set "LLAMA_DIR=D:\Users\ashah\Documents\llama.cpp"
set "LLAMA_SERVER=%LLAMA_DIR%\llama-server.exe"
set "LLAMA_MODEL=C:\Users\ashah\.ollama\models\Qwen3-4B-Instruct-2507-Q4_K_M.gguf"

goto :main

:usage
echo APEX Management Script
echo.
echo Usage: apex.bat [command]
echo.
echo Commands:
echo   start          Start all services (router, llama-server, UI)
echo   stop           Stop all services
echo   restart        Restart all services
echo   build          Build all components (core + UI)
echo   test           Run tests
echo   router         Start router only
echo   router-llm     Start router with LLM enabled
echo   router-llm-no-llama Start router with LLM (llama already running)
echo   router-gvisor    Start router with gVisor isolation
echo   router-firecracker Start router with Firecracker isolation
echo   router-mock     Start router with Mock isolation
echo   llama          Start llama-server
echo   ui             Start UI dev server
echo   ui-serve       Serve built UI
echo   status         Show status of all services
echo   llama-test     Test if llama-server is running
echo   docker-build   Build Docker execution image
echo   docker-test    Test Docker execution
echo.
echo Environment:
echo   LLAMA_MODEL    Model file path (default: %LLAMA_MODEL%)
echo   APEX_USE_DOCKER Enable Docker execution (set to 1)
echo.
exit /b 0

:main
if "%~1"=="" goto usage
set "CMD=%~1"

if "%CMD%"=="start" goto cmd_start
if "%CMD%"=="stop" goto cmd_stop
if "%CMD%"=="restart" goto cmd_restart
if "%CMD%"=="build" goto cmd_build
if "%CMD%"=="test" goto cmd_test
if "%CMD%"=="router" goto cmd_router
if "%CMD%"=="router-llm" goto cmd_router_llm
if "%CMD%"=="router-llm-no-llama" goto cmd_router_llm_no_start
if "%CMD%"=="router2" goto cmd_router2
if "%CMD%"=="router2-llm" goto cmd_router2_llm
if "%CMD%"=="router-gvisor" goto cmd_router_gvisor
if "%CMD%"=="router-firecracker" goto cmd_router_firecracker
if "%CMD%"=="router-mock" goto cmd_router_mock
if "%CMD%"=="llama" goto cmd_llama
if "%CMD%"=="llama-test" goto cmd_llama_test
if "%CMD%"=="docker-build" goto cmd_docker_build
if "%CMD%"=="docker-test" goto cmd_docker_test
if "%CMD%"=="ui" goto cmd_ui
if "%CMD%"=="ui-serve" goto cmd_ui_serve
if "%CMD%"=="status" goto cmd_status
if "%CMD%"=="port" goto cmd_port
goto usage

:cmd_start
echo Starting all services...
echo Starting llama-server...
start "APEX Llama-Server" cmd /c "cd /d "%PROJECT_DIR%" && apex.bat llama"
echo Waiting for Llama-Server to be ready...
set /a attempts=0
:wait_start_llama
ping -n 3 127.0.0.1 >nul 2>&1
set /a attempts+=1
netstat -ano | findstr ":%LLAMA_PORT% " | findstr LISTENING >nul
if errorlevel 1 (
    if %attempts% LSS 15 (
        echo Waiting... (%attempts%/15)
        goto wait_start_llama
    )
    echo Llama-Server may not be ready, starting router anyway...
) else (
    echo Llama-Server is ready!
)
start "APEX Router" cmd /c "cd /d "%PROJECT_DIR%" && apex.bat router-llm-no-llama"
ping -n 4 127.0.0.1 >nul 2>&1
start "APEX UI" cmd /c "cd /d "%PROJECT_DIR%" && apex.bat ui"
echo All services started.
echo   - Llama-Server: http://localhost:%LLAMA_PORT%
echo   - Router: http://localhost:%ROUTER_PORT%
echo   - UI: http://localhost:%UI_PORT%
exit /b 0

:cmd_stop
echo Stopping all services...
taskkill /F /IM llama-server.exe 2>nul
taskkill /F /IM apex-router.exe 2>nul
taskkill /F /IM node.exe 2>nul
echo All services stopped.
exit /b 0

:cmd_restart
echo Restarting all services...
call :cmd_stop
ping -n 3 127.0.0.1 >nul 2>&1
call :cmd_start
exit /b 0

:cmd_build
echo Building APEX...
cd /d "%CORE_DIR%"
echo Building Core (Rust)...
cargo build --release
if errorlevel 1 (
    echo Core build failed!
    exit /b 1
)
echo Building UI...
cd /d "%UI_DIR%"
call pnpm build
if errorlevel 1 (
    echo UI build failed!
    exit /b 1
)
echo Build complete.
exit /b 0

:cmd_test
echo Running tests...
cd /d "%CORE_DIR%"
cargo test --release
exit /b

:cmd_router
echo Starting Router (without LLM)...
cd /d "%CORE_DIR%"
cargo run --release --bin apex-router
exit /b 0

:cmd_router_llm
echo Starting Llama-Server first...
call :cmd_llama
echo Waiting for Llama-Server to be ready...
set /a attempts=0
:wait_llama
ping -n 3 127.0.0.1 >nul 2>&1
set /a attempts+=1
netstat -ano | findstr ":%LLAMA_PORT% " | findstr LISTENING >nul
if errorlevel 1 (
    if %attempts% LSS 15 (
        echo Waiting... (%attempts%/15)
        goto wait_llama
    )
    echo Llama-Server may not be ready, continuing anyway...
) else (
    echo Llama-Server is ready!
)
echo Starting Router with LLM...
cd /d "%CORE_DIR%"
set "APEX_USE_LLM=1"
set "APEX_USE_DOCKER=1"
set "APEX_EXECUTION_ISOLATION=docker"
set "APEX_DOCKER_IMAGE=apex-execution:latest"
set "LLAMA_SERVER_URL=http://127.0.0.1:%LLAMA_PORT%"
set "LLAMA_MODEL=%LLAMA_MODEL%"
cargo run --release --bin apex-router
exit /b 0

:cmd_router_llm_no_start
echo Starting Router with LLM (assuming Llama-Server is already running)...
cd /d "%CORE_DIR%"
set "APEX_USE_LLM=1"
set "APEX_USE_DOCKER=1"
set "APEX_EXECUTION_ISOLATION=docker"
set "APEX_DOCKER_IMAGE=apex-execution:latest"
set "LLAMA_SERVER_URL=http://127.0.0.1:%LLAMA_PORT%"
set "LLAMA_MODEL=%LLAMA_MODEL%"
cargo run --release --bin apex-router
exit /b 0

:cmd_router_docker
echo Starting Router with Docker isolation...
cd /d "%CORE_DIR%"
set "APEX_USE_DOCKER=1"
set "APEX_EXECUTION_ISOLATION=docker"
set "APEX_DOCKER_IMAGE=apex-execution:latest"
cargo run --release --bin apex-router
exit /b 0

:cmd_router2
echo Starting Router on port 3001 (for testing)...
cd /d "%CORE_DIR%"
set "APEX_PORT=3001"
cargo run --release --bin apex-router
exit /b 0

:cmd_router2_llm
echo Starting Llama-Server first...
call :cmd_llama
echo Waiting for Llama-Server to be ready...
set /a attempts=0
:wait_llama2
ping -n 3 127.0.0.1 >nul 2>&1
set /a attempts+=1
netstat -ano | findstr ":%LLAMA_PORT% " | findstr LISTENING >nul
if errorlevel 1 (
    if %attempts% LSS 15 (
        echo Waiting... (%attempts%/15)
        goto wait_llama2
    )
    echo Llama-Server may not be ready, continuing anyway...
) else (
    echo Llama-Server is ready!
)
echo Starting Router with LLM on port 3001...
cd /d "%CORE_DIR%"
set "APEX_PORT=3001"
set "APEX_USE_LLM=1"
set "LLAMA_SERVER_URL=http://127.0.0.1:%LLAMA_PORT%"
set "LLAMA_MODEL=%LLAMA_MODEL%"
cargo run --release --bin apex-router
exit /b 0

:cmd_router_gvisor
echo Starting Router with gVisor isolation...
cd /d "%CORE_DIR%"
set "APEX_EXECUTION_ISOLATION=gvisor"
set "APEX_RUNSC_PATH=\\wsl$\Ubuntu-20.04\usr\local\bin\runsc"
cargo run --release --bin apex-router
exit /b 0

:cmd_router_firecracker
echo Starting Router with Firecracker isolation...
cd /d "%CORE_DIR%"
set "APEX_EXECUTION_ISOLATION=firecracker"
set "APEX_USE_FIRECRACKER=1"
set "APEX_VM_KERNEL=\\wsl$\Ubuntu-20.04\usr\local\bin\vmlinux"
set "APEX_VM_ROOTFS=\\wsl$\Ubuntu-20.04\tmp\rootfs.ext4"
set "APEX_FIRECRACKER_PATH=\\wsl$\Ubuntu-20.04\usr\local\bin\firecracker"
cargo run --release --bin apex-router
exit /b 0

:cmd_router_mock
echo Starting Router with Mock isolation (no real execution)...
cd /d "%CORE_DIR%"
set "APEX_EXECUTION_ISOLATION=mock"
cargo run --release --bin apex-router
exit /b 0

:cmd_port
echo Current port configuration:
echo   Router: %ROUTER_PORT%
echo   UI: %UI_PORT%
echo   Llama: %LLAMA_PORT%
echo.
echo To run router on different port:
echo   apex.bat router2        - router on 3001
echo   apex.bat router2-llm    - router with LLM on 3001
exit /b 0

:cmd_llama
echo Starting llama-server on port %LLAMA_PORT%...
echo Model: %LLAMA_MODEL%
set "LOCAL_LLAMA=%PROJECT_DIR%temp-llama\llama-server.exe"
if exist "%LOCAL_LLAMA%" (
    echo Using local llama-server from temp-llama folder
    start "llama-server" cmd /c ""%LOCAL_LLAMA%" -m "%LLAMA_MODEL%" --port %LLAMA_PORT% -c 4096"
) else if exist "%LLAMA_SERVER%" (
    echo Using llama-server from %LLAMA_DIR%
    start "llama-server" cmd /c ""%LLAMA_SERVER%" -m "%LLAMA_MODEL%" --port %LLAMA_PORT% -c 4096"
) else (
    echo ERROR: llama-server.exe not found
    echo   Local: %LOCAL_LLAMA%
    echo   External: %LLAMA_SERVER%
    echo Please check LLAMA_DIR in apex.bat or add llama-server.exe to temp-llama folder
    exit /b 1
)
exit /b 0

:cmd_llama_test
echo Testing llama-server on port %LLAMA_PORT%...
netstat -ano | findstr ":%LLAMA_PORT% " | findstr LISTENING >nul
if errorlevel 1 (
    echo ERROR: No process listening on port %LLAMA_PORT%
    echo Is llama-server running?
    exit /b 1
)
echo Port %LLAMA_PORT% is listening.
echo Testing API endpoint...
curl -s "http://localhost:%LLAMA_PORT%/v1/models" >nul 2>&1
if errorlevel 1 (
    echo WARNING: Port is open but API not responding
    echo Is llama-server fully loaded?
    exit /b 1
)
echo SUCCESS: llama-server is running and responding!
echo.
curl -s "http://localhost:%LLAMA_PORT%/v1/models" | findstr /C:"model"
exit /b 0

:cmd_ui
echo Starting UI dev server...
for /f "tokens=5" %%a in ('netstat -ano ^| findstr :%UI_PORT% ^| findstr LISTENING') do (
    echo Killing existing process on port %UI_PORT% (PID: %%a)
    taskkill /F /PID %%a 2>nul
)
cd /d "%UI_DIR%"
call pnpm dev
exit /b 0

:cmd_ui_serve
echo Serving built UI on port %UI_PORT%...
cd /d "%UI_DIR%"
npx serve dist -l %UI_PORT%
exit /b 0

:cmd_docker_build
echo Building Docker execution image...
cd /d "%PROJECT_DIR%execution"
docker build -t apex-execution:latest .
echo Build complete.
echo To run with Docker: set APEX_USE_DOCKER=1 and restart router
exit /b 0

:cmd_docker_test
echo Testing Docker execution...
docker run --rm apex-execution:latest python -c "print('Docker execution working!')"
exit /b 0

:cmd_status
echo Checking service status...
echo.
echo Port %ROUTER_PORT% (Router):
netstat -ano | findstr ":%ROUTER_PORT% " | findstr LISTENING
if errorlevel 1 echo   - Not running
echo.
echo Port %LLAMA_PORT% (Llama-Server):
netstat -ano | findstr ":%LLAMA_PORT% " | findstr LISTENING
if errorlevel 1 echo   - Not running
echo.
echo Port %UI_PORT% (UI):
netstat -ano | findstr ":%UI_PORT% " | findstr LISTENING
if errorlevel 1 echo   - Not running
echo.
echo Processes:
tasklist /FI "IMAGENAME eq apex-router.exe" 2>nul | findstr /I apex-router.exe
if errorlevel 1 echo   - Router: Not running
tasklist /FI "IMAGENAME eq llama-server.exe" 2>nul | findstr /I llama-server.exe
if errorlevel 1 echo   - Llama-Server: Not running
exit /b 0
