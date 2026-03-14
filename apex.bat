@echo off
setlocal enabledelayedexpansion

set "PROJECT_DIR=%~dp0"
set "CORE_DIR=%PROJECT_DIR%core"
set "UI_DIR=%PROJECT_DIR%ui"
set "ROUTER_PORT=3000"
set "UI_PORT=8083"
set "LLAMA_PORT=8080"
set "EMBED_PORT=8081"
set "LLAMA_DIR=D:\Users\ashah\Documents\llama.cpp"
set "LLAMA_SERVER=%LLAMA_DIR%\llama-server.exe"
set "LLAMA_MODEL=C:\Users\ashah\.ollama\models\Qwen3-4B-Instruct-2507-Q4_K_M.gguf"
set "EMBED_MODEL=D:\Users\ashah\AppData\Local\Programs\LM Studio\resources\app\.webpack\bin\bundled-models\nomic-ai\nomic-embed-text-v1.5-GGUF\nomic-embed-text-v1.5.Q4_K_M.gguf"
set "DOCKER Desktop=C:\Program Files\Docker\Docker\Docker Desktop.exe"

goto :main

:usage
echo APEX Management Script
echo.
echo Usage: apex.bat [command] or apex.bat menu
echo.
echo Commands:
echo   menu            Interactive menu (recommended)
echo   start           Start all services (router, llama-server, UI)
echo   start-full     Start all services including embedding server
echo   stop           Stop all services
echo   restart        Restart all services
echo   build          Build all components (core + UI)
echo   test           Run tests
echo   router         Start router only
echo   router-llm     Start router with LLM enabled
echo   router-llm-no-llama Start router with LLM (llama already running)
echo   router-docker  Start router with Docker isolation
echo   router-mock    Start router with Mock isolation (no Docker)
echo   llama          Start llama-server (LLM)
echo   embed          Start embedding server (nomic-embed-text)
echo   embed-test     Test if embedding server is running
echo   ui             Start UI dev server
echo   ui-serve       Serve built UI
echo   status         Show status of all services
echo   llama-test     Test if llama-server is running
echo   docker-start   Start Docker Desktop
echo   docker-build   Build Docker execution image
echo   docker-test    Test Docker execution
echo.
echo Environment:
echo   LLAMA_MODEL    LLM model file path
echo   EMBED_MODEL    Embedding model file path
echo.
exit /b 0

:main
if "%~1"=="" goto cmd_menu
if "%~1"=="menu" goto cmd_menu
set "CMD=%~1"

if "%CMD%"=="start" goto cmd_start
if "%CMD%"=="start-full" goto cmd_start_full
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
if "%CMD%"=="router-docker" goto cmd_router_docker
if "%CMD%"=="llama" goto cmd_llama
if "%CMD%"=="llama-test" goto cmd_llama_test
if "%CMD%"=="embed" goto cmd_embed
if "%CMD%"=="embed-test" goto cmd_embed_test
if "%CMD%"=="docker-start" goto cmd_docker_start
if "%CMD%"=="docker-build" goto cmd_docker_build
if "%CMD%"=="docker-test" goto cmd_docker_test
if "%CMD%"=="ui" goto cmd_ui
if "%CMD%"=="ui-serve" goto cmd_ui_serve
if "%CMD%"=="status" goto cmd_status
if "%CMD%"=="port" goto cmd_port
if "%CMD%"=="menu" goto cmd_menu
goto usage

:cmd_menu
cls
echo ========================================
echo         APEX Management Menu
echo ========================================
echo.
echo Select an option:
echo.
echo  [1] Start All (LLM + UI + Router)
echo  [2] Start Full (LLM + Embed + UI + Router)
echo  [3] Start UI Only
echo  [4] Start Router Only (no LLM)
echo  [5] Start Router with LLM (requires llama-server)
echo  [6] Start Router with Mock (no Docker needed)
echo  [7] Start Docker Desktop
echo  [8] Start Llama-Server (LLM)
echo  [9] Start Embedding Server (Nomic)
echo  [0] Status / View Running Services
echo  [B] Build All
echo  [T] Run Tests
echo  [S] Stop All Services
echo  [Q] Quit
echo.
set /p "choice=Enter choice: "

if "%choice%"=="1" goto cmd_start
if "%choice%"=="2" goto cmd_start_full
if "%choice%"=="3" goto cmd_ui
if "%choice%"=="4" goto cmd_router
if "%choice%"=="5" goto cmd_router_llm_no_start
if "%choice%"=="6" goto cmd_router_mock
if "%choice%"=="7" goto cmd_docker_start
if "%choice%"=="8" goto cmd_llama
if "%choice%"=="9" goto cmd_embed
if "%choice%"=="0" goto cmd_status
if "%choice%"=="b" goto cmd_build
if "%choice%"=="B" goto cmd_build
if "%choice%"=="t" goto cmd_test
if "%choice%"=="T" goto cmd_test
if "%choice%"=="s" goto cmd_stop
if "%choice%"=="S" goto cmd_stop
if "%choice%"=="q" goto cmd_quit
if "%choice%"=="Q" goto cmd_quit

echo Invalid choice, press any key to retry...
pause >nul
goto cmd_menu

:cmd_quit
exit /b 0

:check_docker
echo Checking Docker status...
docker info >nul 2>&1
if errorlevel 1 (
    echo Docker is not running.
    set /p "start_docker=Start Docker Desktop now? (Y/N): "
    if /i "%start_docker%"=="Y" goto cmd_docker_start
    echo WARNING: Docker not available. Using mock isolation.
    set "APEX_EXECUTION_ISOLATION=mock"
    set "APEX_USE_DOCKER=0"
    exit /b 1
)
echo Docker is running.
exit /b 0

:cmd_docker_start
echo Starting Docker Desktop...
if exist "%DOCKER Desktop%" (
    start "" "%DOCKER Desktop%"
    echo Waiting for Docker to start...
    set /a attempts=0
    :wait_docker
    ping -n 3 127.0.0.1 >nul 2>&1
    set /a attempts+=1
    docker info >nul 2>&1
    if errorlevel 1 (
        if %attempts% LSS 30 (
            echo Waiting for Docker... (%attempts%/30)
            goto wait_docker
        )
        echo Docker may not be ready yet.
    ) else (
        echo Docker is ready!
    )
) else (
    echo ERROR: Docker Desktop not found at:
    echo   %DOCKER Desktop%
    echo Please install Docker Desktop from https://www.docker.com/products/docker-desktop
    exit /b 1
)
exit /b 0

:cmd_start
echo Starting all services (without embedding server)...
call :check_docker
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
echo.
echo To start embedding server: apex.bat embed
exit /b 0

:cmd_start_full
echo Starting all services INCLUDING embedding server...
call :check_docker
echo Starting llama-server (LLM)...
start "APEX Llama-Server" cmd /c "cd /d "%PROJECT_DIR%" && apex.bat llama"
echo Starting embedding server...
start "APEX Embed" cmd /c "cd /d "%PROJECT_DIR%" && apex.bat embed"
echo Waiting for services to be ready...
set /a attempts=0
:wait_start_full
ping -n 3 127.0.0.1 >nul 2>&1
set /a attempts+=1
netstat -ano | findstr ":%LLAMA_PORT% " | findstr LISTENING >nul
set /a llama_ready=!errorlevel!
netstat -ano | findstr ":%EMBED_PORT% " | findstr LISTENING >nul
set /a embed_ready=!errorlevel!
if !llama_ready! NEQ 0 (
    if !embed_ready! NEQ 0 (
        if %attempts% LSS 15 (
            echo Waiting... (%attempts%/15)
            goto wait_start_full
        )
    )
)
if !llama_ready! EQU 0 echo Llama-Server is ready!
if !embed_ready! EQU 0 echo Embedding server is ready!
start "APEX Router" cmd /c "cd /d "%PROJECT_DIR%" && apex.bat router-llm-no-llama"
ping -n 4 127.0.0.1 >nul 2>&1
start "APEX UI" cmd /c "cd /d "%PROJECT_DIR%" && apex.bat ui"
echo All services started.
echo   - Llama-Server: http://localhost:%LLAMA_PORT%
echo   - Embedding Server: http://localhost:%EMBED_PORT%
echo   - Router: http://localhost:%ROUTER_PORT%
echo   - UI: http://localhost:%UI_PORT%
exit /b 0

:cmd_stop
echo Stopping all services...
taskkill /F /IM llama-server.exe 2>nul
taskkill /F /IM apex-router.exe 2>nul
taskkill /F /IM node.exe 2>nul

echo Cleaning up APEX Docker containers...
docker rm -f apex-vm-0 apex-vm-1 apex-vm-2 apex-vm-3 apex-vm-4 2>nul
docker rm -f apex-vm-vm-0 apex-vm-vm-1 apex-vm-vm-2 apex-vm-vm-3 apex-vm-vm-4 2>nul

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
set "APEX_MEMORY_EMBEDDING_URL=http://127.0.0.1:%EMBED_PORT%"
cargo run --release --bin apex-router
exit /b 0

:cmd_router_llm_no_start
call :check_docker
echo Starting Router with LLM (assuming Llama-Server is already running)...
set "PATH=%PATH%;C:\Users\ashah\.bun\bin"
cd /d "%CORE_DIR%"
set "APEX_USE_LLM=1"
set "APEX_USE_DOCKER=1"
set "APEX_EXECUTION_ISOLATION=docker"
set "APEX_DOCKER_IMAGE=apex-execution:latest"
set "APEX_SKILL_POOL_ENABLED=1"
set "APEX_SKILL_POOL_WORKER=%PROJECT_DIR%skills\pool_worker.ts"
set "LLAMA_SERVER_URL=http://127.0.0.1:%LLAMA_PORT%"
set "LLAMA_MODEL=%LLAMA_MODEL%"
set "APEX_MEMORY_EMBEDDING_URL=http://127.0.0.1:%EMBED_PORT%"
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
set "APEX_MEMORY_EMBEDDING_URL=http://127.0.0.1:%EMBED_PORT%"
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
set "APEX_VM_KERNEL=\\wsl$\Ubuntu-20.04\tmp\vmlinux"
set "APEX_VM_ROOTFS=\\wsl$\Ubuntu-20.04\tmp\rootfs.ext4"
set "APEX_FIRECRACKER_PATH=\\wsl$\Ubuntu-20.04\usr\local\bin\firecracker"
cargo run --release --bin apex-router
exit /b 0

:cmd_router_mock
echo Starting Router with Mock isolation (no Docker needed)...
cd /d "%CORE_DIR%"
set "APEX_EXECUTION_ISOLATION=mock"
set "APEX_USE_DOCKER=0"
cargo run --release --bin apex-router
exit /b 0

:cmd_port
echo Current port configuration:
echo   Router: %ROUTER_PORT%
echo   UI: %UI_PORT%
echo   Llama (LLM): %LLAMA_PORT%
echo   Embedding: %EMBED_PORT%
echo.
echo To run router on different port:
echo   apex.bat router2        - router on 3001
echo   apex.bat router2-llm   - router with LLM on 3001
echo.
echo To start embedding server:
echo   apex.bat embed         - start embedding server
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

:cmd_embed
echo Starting embedding server on port %EMBED_PORT%...
echo Model: %EMBED_MODEL%
set "LOCAL_LLAMA=%PROJECT_DIR%temp-llama\llama-server.exe"
if exist "%LOCAL_LLAMA%" (
    echo Using local llama-server from temp-llama folder
    start "APEX-Embed" cmd /c ""%LOCAL_LLAMA%" -m "%EMBED_MODEL%" --embedding --port %EMBED_PORT% -c 8192"
) else if exist "%LLAMA_SERVER%" (
    echo Using llama-server from %LLAMA_DIR%
    start "APEX-Embed" cmd /c ""%LLAMA_SERVER%" -m "%EMBED_MODEL%" --embedding --port %EMBED_PORT% -c 8192"
) else (
    echo ERROR: llama-server.exe not found
    echo   Local: %LOCAL_LLAMA%
    echo   External: %LLAMA_SERVER%
    echo Please check LLAMA_DIR in apex.bat or add llama-server.exe to temp-llama folder
    exit /b 1
)
exit /b 0

:cmd_embed_test
echo Testing embedding server on port %EMBED_PORT%...
netstat -ano | findstr ":%EMBED_PORT% " | findstr LISTENING >nul
if errorlevel 1 (
    echo ERROR: No process listening on port %EMBED_PORT%
    echo Is embed server running? (apex.bat embed)
    exit /b 1
)
echo Port %EMBED_PORT% is listening.
echo Testing embedding endpoint...
curl -s "http://localhost:%EMBED_PORT%/v1/embeddings" -H "Content-Type: application/json" -d "{\"input\":\"test\"}" >nul 2>&1
if errorlevel 1 (
    echo WARNING: Port is open but embedding API not responding
    echo Is embedding model fully loaded?
    exit /b 1
)
echo SUCCESS: Embedding server is running and responding!
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
echo ========================================
echo         APEX Service Status
echo ========================================
echo.
echo Port %ROUTER_PORT% (Router):
netstat -ano | findstr ":%ROUTER_PORT% " | findstr LISTENING >nul
if errorlevel 1 (
    echo   - Not running
) else (
    echo   - Running
)
echo.
echo Port %LLAMA_PORT% (Llama-Server LLM):
netstat -ano | findstr ":%LLAMA_PORT% " | findstr LISTENING >nul
if errorlevel 1 (
    echo   - Not running
) else (
    echo   - Running
)
echo.
echo Port %EMBED_PORT% (Embedding Server):
netstat -ano | findstr ":%EMBED_PORT% " | findstr LISTENING >nul
if errorlevel 1 (
    echo   - Not running
) else (
    echo   - Running
)
echo.
echo Port %UI_PORT% (UI):
netstat -ano | findstr ":%UI_PORT% " | findstr LISTENING >nul
if errorlevel 1 (
    echo   - Not running
) else (
    echo   - Running
)
echo.
echo Docker:
docker info >nul 2>&1
if errorlevel 1 (
    echo   - Not running
) else (
    echo   - Running
)
echo.
echo ========================================
echo.
echo URLs:
echo   UI:        http://localhost:%UI_PORT%
echo   Router:    http://localhost:%ROUTER_PORT%
echo   Llama:     http://localhost:%LLAMA_PORT%
echo   Embedding: http://localhost:%EMBED_PORT%
echo.
pause
goto :main
