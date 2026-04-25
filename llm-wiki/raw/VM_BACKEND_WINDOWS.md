# VM Execution Backends on Windows 11

> **Platform**: Windows 11 with WSL2
> **Last Updated**: 2026-03-09

## Overview

APEX supports multiple execution backends for running deep tasks in isolation. This document covers the available options on Windows 11.

## Available Backends

| Backend | Security | Performance | Windows Support | Status |
|---------|----------|-------------|-----------------|--------|
| Docker | ⭐⭐⭐⭐ | Excellent | Native | ✅ Default |
| Firecracker (WSL2) | ⭐⭐⭐⭐⭐ | Excellent | WSL2 | ✅ Ready |
| gVisor (WSL2) | ⭐⭐⭐⭐⭐ | Good | WSL2 | 🔧 Needs setup |
| Mock | N/A | N/A | Native | ✅ Built-in |

## Docker (Default - Recommended)

### Security Features

APEX's Docker backend includes robust security hardening:

```rust
// From core/router/src/vm_pool.rs
--memory 2048m              // 2GB memory limit
--cpus 2                    // 2 CPU cores
--pids-limit 256            // Max 256 processes
--network none              // Network isolation
--read-only                 // Read-only filesystem
--tmpfs /tmp:rw,exec       // Writable tmpfs
--cap-drop ALL              // Drop all capabilities
--privileged=false          // No privileged mode
--restart no                // No auto-restart
--rm                        // Auto-remove on exit
```

### Usage

```powershell
# Start router with Docker isolation
apex.bat router-docker

# Or manually
set APEX_USE_DOCKER=1
set APEX_EXECUTION_ISOLATION=docker
set APEX_DOCKER_IMAGE=apex-execution:latest
cargo run --release --bin apex-router
```

### Test Docker

```powershell
# Test Docker is working
apex.bat docker-test

# Or manually
docker run --rm apex-execution:latest python -c "print('OK')"
```

## Firecracker (via WSL2)

### Why WSL2?

Firecracker requires KVM (Kernel-based Virtual Machine), which is Linux-only. On Windows, we use WSL2 which provides a Linux kernel that can run Firecracker.

### Requirements

1. **WSL2** with Ubuntu-20.04 installed
2. **Firecracker binary** installed in WSL2
3. **Linux kernel** (vmlinux) - uncompressed ELF
4. **Root filesystem** (ext4 image)

### Current Status

| Component | Status | Path |
|-----------|--------|------|
| WSL2 Ubuntu-20.04 | ✅ Running | - |
| Firecracker v1.14.2 | ✅ Installed | `/usr/local/bin/firecracker` |
| Linux Kernel v5.10 | ✅ Ready | `/tmp/vmlinux` (42MB) |
| Ubuntu Rootfs | ✅ Ready | `/tmp/rootfs.ext4` (512MB) |

### Usage

```powershell
# Using apex.bat
apex.bat router-firecracker

# Or manually
set APEX_EXECUTION_ISOLATION=firecracker
set APEX_USE_FIRECRACKER=1
set APEX_VM_KERNEL=\\wsl$\Ubuntu-20.04\tmp\vmlinux
set APEX_VM_ROOTFS=\\wsl$\Ubuntu-20.04\tmp\rootfs.ext4
set APEX_FIRECRACKER_PATH=\\wsl$\Ubuntu-20.04\usr\local\bin\firecracker
cargo run --release --bin apex-router
```

### Rebuild Assets

If you need to rebuild the kernel/rootfs:

```powershell
# In WSL2
wsl -d Ubuntu-20.04

# Download kernel
curl -L -o /tmp/vmlinux https://s3.amazonaws.com/spec.ccfc.min/ci-artifacts/kernels/x86_64/vmlinux-5.10.bin

# Create rootfs
debootstrap focal /tmp/rootfs http://archive.ubuntu.com/ubuntu/
dd if=/dev/zero of=/tmp/rootfs.ext4 bs=1M count=512
mkfs.ext4 -F /tmp/rootfs.ext4
mount -o loop /tmp/rootfs.ext4 /mnt/rootfs
cp -a /tmp/rootfs/. /mnt/rootfs/
umount /mnt/rootfs
```

## gVisor (via WSL2)

gVisor provides a stronger security boundary than Docker, running containers with a kernel-level sandbox.

### Requirements

1. **gVisor runsc binary** installed in WSL2
2. **Docker** (gVisor typically runs as a Docker runtime)

### Setup

```powershell
# In WSL2
wsl -d Ubuntu-20.04

# Install gVisor
curl -fsSL https://gvisor.dev/install.sh | bash

# Verify
runsc --version
```

### Usage

```powershell
# Using apex.bat
apex.bat router-gvisor

# Or manually
set APEX_EXECUTION_ISOLATION=gvisor
set APEX_RUNSC_PATH=\\wsl$\Ubuntu-20.04\usr\local\bin\runsc
cargo run --release --bin apex-router
```

## Mock (Testing)

For development/testing without any isolation:

```powershell
apex.bat router-mock

# Or manually
set APEX_EXECUTION_ISOLATION=mock
cargo run --release --bin apex-router
```

## Comparison

### Security

| Backend | Isolation Level | Network | Capabilities |
|---------|-----------------|---------|--------------|
| Docker | Container | `--network none` | Dropped |
| Firecracker | MicroVM | Isolated | Minimal |
| gVisor | Sandboxed | Isolated | Filtered |

### Performance

| Backend | Boot Time | Memory Overhead |
|---------|-----------|-----------------|
| Docker | ~1s | ~20MB |
| Firecracker | ~100ms | ~5MB |
| gVisor | ~1s | ~10MB |

## Recommendations

### For Development
- Use **Docker** - most reliable on Windows
- Use **Mock** for unit tests

### For Production (when Firecracker works)
- Use **Firecracker** - best security/performance
- Use **gVisor** as fallback

### Current Status
- **Docker**: ✅ Production-ready
- **Firecracker**: ⚠️ Requires KVM in WSL2 (may not work on all hardware)
- **gVisor**: 🔧 Needs testing

## Verification Commands

```powershell
# Check WSL2
wsl -l -v

# Check Docker
docker --version

# Check Firecracker (in WSL2)
wsl -d Ubuntu-20.04 firecracker --version

# Check gVisor (in WSL2)
wsl -d Ubuntu-20.04 runsc --version

# Test Docker execution
docker run --rm apex-execution:latest python --version
```

## Troubleshooting

### "KVM not available" with Firecracker

Firecracker requires hardware virtualization (VT-x/AMD-V). Check:
1. BIOS/UEFI has virtualization enabled
2. WSL2 is configured to use virtualization

```powershell
# Check virtualization
systeminfo | findstr /C:"Virtualization"
```

### Docker not starting

```powershell
# Start Docker Desktop
start "" "C:\Program Files\Docker\Docker\Docker Desktop.exe"

# Or use the batch file
apex.bat docker-start
```
