# VM Execution Backends

APEX supports multiple execution backends for running deep tasks in isolation. This document covers the available options and setup instructions.

## Available Backends

| Backend | Security | Performance | Setup Complexity |
|---------|----------|-------------|------------------|
| Docker | High | Excellent | Low (default) |
| gVisor | Very High | Good | Medium |
| Firecracker | Very High | Excellent | High (requires KVM) |

## Docker (Default)

**Status**: Implemented and default

Docker provides excellent isolation with resource limits:

```bash
# Resource limits applied:
--memory=2048m        # 2GB memory limit
--cpus=2              # 2 CPU cores
--pids-limit=256      # Max 256 processes
--network=none        # Network isolation
--read-only           # Read-only filesystem by default
--tmpfs=/tmp:rw       # Writable tmpfs for temp files
```

**Enable**:
```powershell
# In apex.bat, router-llm mode sets APEX_USE_DOCKER=1 automatically
.\apex.bat router-llm
```

## gVisor (Recommended for Production)

gVisor provides a stronger security boundary than Docker, running containers with a kernel-level sandbox.

### Installation

1. Download gVisor:
```bash
curl -fsSL https://gvisor.dev/install.sh | bash
```

2. Verify installation:
```bash
runsc --version
```

3. Set environment:
```bash
set APEX_USE_GVISOR=1
set APEX_RUNSC_PATH=C:\path\to\runsc.exe
```

### Configuration

gVisor is configured automatically in APEX. The following limits are enforced:
- 2 vCPUs
- 2GB memory
- Network isolation via gVisor's internal network stack
- Syscall filtering via seccomp

### Performance Notes

gVisor intercepts syscalls, which adds ~5-15% overhead vs native execution. For most LLM tasks, this is negligible.

## Firecracker (High Performance)

Firecracker provides VM-level isolation with near-native performance. Requires KVM virtualization.

### Requirements

1. **Hardware**: CPU with VT-x/AMD-V virtualization
2. **OS**: Linux (recommended) or macOS with Linux VMs
3. **KVM**: Must have KVM access

### Installation

1. Install Firecracker:
```bash
# Linux
sudo apt install firecracker

# Or download binary
curl -Lo /usr/local/bin/firecracker https://github.com/firecracker-microvm/firecracker/releases/latest/firecracker-v$(uname -m)
chmod +x /usr/local/bin/firecracker
```

2. Download VM image:
```bash
# Download minimal Ubuntu image
curl -Lo /var/lib/firecracker/images/rootfs.ext4 https://github.com/firecracker-microvm/firecracker-starter-aarch64/raw/master/images/ubuntu-18.04-server.ext4
curl -Lo /var/lib/firecracker/images/vmlinux https://github.com/firecracker-microvm/firecracker-starter-aarch64/raw/master/images/vmlinux
```

3. Configure APEX:
```powershell
set APEX_USE_FIRECRACKER=1
set APEX_VM_KERNEL=C:\path\to\vmlinux
set APEX_VM_ROOTFS=C:\path\to\rootfs.ext4
```

### Security Features

- VM-level isolation (stronger than containers)
- No host kernel access
- Configurable network policies
- Resource metering
- **vsock support** for secure communication (v0.2.0+)

### vsock Communication (v0.2.0)

APEX v0.2.0 implements vsock for secure communication with Firecracker VMs:

```
/tmp/vsock-{vm_id}.sock  # Unix socket for command execution
```

The execution flow:
1. Router spawns Firecracker VM with configured kernel/rootfs
2. VM starts an agent listening on vsock
3. Router sends commands via Unix socket
4. Agent executes and returns results
5. VM is returned to pool for reuse

This provides better isolation than network-based communication.

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `APEX_USE_DOCKER` | Enable Docker backend | 0 |
| `APEX_USE_GVISOR` | Enable gVisor backend | 0 |
| `APEX_USE_FIRECRACKER` | Enable Firecracker backend | 0 |
| `APEX_VM_VCPU` | Number of virtual CPUs | 2 |
| `APEX_VM_MEMORY_MIB` | Memory in MiB | 2048 |
| `APEX_DOCKER_IMAGE` | Docker image to use | apex-execution:latest |
| `APEX_VM_KERNEL` | Path to Firecracker kernel | - |
| `APEX_VM_ROOTFS` | Path to Firecracker rootfs | - |
| `APEX_RUNSC_PATH` | Path to runsc binary | runsc |

## Development vs Production

### Development
- Use Docker (default) for fastest iteration
- gVisor for testing security boundaries

### Production
- **Recommended**: gVisor for balance of security/performance
- **Maximum security**: Firecracker with custom kernel

## Troubleshooting

### Docker container conflicts
If you see "container name already in use":
```bash
# Clean up old containers
docker rm -f $(docker ps -aq --filter "name=apex-vm-*")
```

### gVisor won't start
```bash
# Check if kernel module is loaded
lsmod | grep pti

# Manually start runsc
runsc --version
runsc --help
```

### Firecracker KVM access denied
```bash
# Check KVM permissions
ls -la /dev/kvm

# Add user to kvm group
sudo usermod -a -G kvm $USER
# Then logout and login
```
