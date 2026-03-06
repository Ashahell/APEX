$env:APEX_USE_DOCKER="1"
$env:APEX_EXECUTION_ISOLATION="docker"
$env:APEX_DOCKER_IMAGE="apex-execution:latest"
$env:RUST_LOG="debug"
cd E:\projects\APEX\core
.\target\release\apex-router.exe 2>&1 | Tee-Object -FilePath E:\projects\APEX\router_docker.log
