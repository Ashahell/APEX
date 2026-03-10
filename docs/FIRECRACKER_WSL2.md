# Firecracker Setup on Windows with WSL2

> **Platform**: Windows 11 with WSL2 (Ubuntu-20.04)
> **Status**: ✅ Ready to use

## Current State

| Component | Status | Path |
|-----------|--------|------|
| WSL2 Ubuntu-20.04 | ✅ Running | - |
| Firecracker binary | ✅ Installed | `/usr/local/bin/firecracker` |
| Linux Kernel | ✅ Ready | `/tmp/vmlinux` (42MB) |
| Root Filesystem | ✅ Ready | `/tmp/rootfs.ext4` (512MB) |

## Quick Start

### Option 1: Run with Firecracker

```powershell
# Set environment variables and run
set APEX_EXECUTION_ISOLATION=firecracker
set APEX_USE_FIRECRACKER=1
set APEX_VM_KERNEL=\\wsl$\Ubuntu-20.04\tmp\vmlinux
set APEX_VM_ROOTFS=\\wsl$\Ubuntu-20.04\tmp\rootfs.ext4
set APEX_FIRECRACKER_PATH=\\wsl$\Ubuntu-20.04\usr\local\bin\firecracker

# Or use the batch file
apex.bat router-firecracker
```

### Option 2: Run with Docker (Default, More Reliable on Windows)

```powershell
apex.bat router-docker
```

## Manual Setup (Already Done)

The following has been completed automatically:

1. ✅ Firecracker v1.14.2 installed in WSL2
2. ✅ Linux kernel v5.10 downloaded (42MB)
3. ✅ Ubuntu 20.04 rootfs created (512MB)
4. ✅ Files placed in `/tmp/`

## Troubleshooting

If you need to rebuild the assets:

### Rebuild Kernel

```powershell
wsl -d Ubuntu-20.04 curl -L -o /tmp/vmlinux https://s3.amazonaws.com/spec.ccfc.min/ci-artifacts/kernels/x86_64/vmlinux-5.10.bin
```

### Rebuild Rootfs

```powershell
wsl -d Ubuntu-20.04
debootstrap focal /tmp/rootfs http://archive.ubuntu.com/ubuntu/
dd if=/dev/zero of=/tmp/rootfs.ext4 bs=1M count=512
mkfs.ext4 -F /tmp/rootfs.ext4
mount -o loop /tmp/rootfs.ext4 /mnt/rootfs
cp -a /tmp/rootfs/. /mnt/rootfs/
umount /mnt/rootfs
```

**Option B: Download Prebuilt Firecracker Kernel**

```powershell
# In WSL2
wsl -d Ubuntu-20.04

# Download stable Firecracker kernel (v6.6.87 from fc-runner)
cd /tmp
curl -L -o vmlinux https://github.com/firecracker-microvm/firecracker-kata-common-repo/releases/download/v0.0.2/vmlinux
chmod +x vmlinux

# Verify
file vmlinux
# Output: vmlinux: ELF 64-bit LSB executable, x86-64, version 1 (SYSV)
```

### Step 2: Create Root Filesystem

```bash
# In WSL2
wsl -d Ubuntu-20.04

# Create a minimal ext4 rootfs (using debootstrap)
sudo apt-get update
sudo apt-get install -y debootstrap

# Create rootfs directory
mkdir -p /tmp/rootfs
cd /tmp

# Bootstrap a minimal Debian/Ubuntu rootfs
sudo debootstrap focal /tmp/rootfs http://archive.ubuntu.com/ubuntu/

# Or use a prebuilt rootfs (faster)
curl -L -o rootfs.ext4 https://raw.githubusercontent.com/firecracker-microvm/firecracker Kata-common-repo/releases/download/v0.0.2/rootfs.ext4
```

**Alternative: Create Rootfs Manually**

```bash
# Create empty ext4 image
dd if=/dev/zero of=rootfs.ext4 bs=1M count=512
mkfs.ext4 -F rootfs.ext4

# Mount and populate
mkdir -p /mnt/rootfs
sudo mount -o loop rootfs.ext4 /mnt/rootfs

# Basic structure
sudo mkdir -p /mnt/rootfs/{bin,etc,lib,home,root,tmp,var,usr/bin,usr/lib}
sudo touch /mnt/rootfs/etc/resolv.conf

# Copy essential binaries (from current system)
sudo cp /bin/bash /bin/sh /mnt/rootfs/bin/
sudo cp /usr/bin/python3 /mnt/rootfs/usr/bin/

# Unmount
sudo umount /mnt/rootfs
```

### Step 3: Configure APEX

```powershell
# Update apex.bat or set environment variables
set APEX_EXECUTION_ISOLATION=firecracker
set APEX_USE_FIRECRACKER=1
set APEX_VM_KERNEL=\\wsl$\Ubuntu-20.04\tmp\vmlinux
set APEX_VM_ROOTFS=\\wsl$\Ubuntu-20.04\tmp\rootfs.ext4
set APEX_FIRECRACKER_PATH=\\wsl$\Ubuntu-20.04\usr\local\bin\firecracker
```

### Step 4: Test Firecracker

```bash
# In WSL2 - Test Firecracker can start
cd /tmp

# Create socket
rm -f /tmp/fc.sock

# Start Firecracker (basic test)
./firecracker --api-sock /tmp/fc.sock --kernel vmlinux --root-drive rootfs.ext4 --ncpu 1 --mem-mib 512

# In another terminal, test API
curl --unix-socket /tmp/fc.sock -i -X GET 'http://localhost/flashboot'
```

## Troubleshooting

### "KVM not available" Error

WSL2 doesn't expose KVM directly. You need to enable hardware virtualization in BIOS and use:

```powershell
# Check if virtualization is enabled
systeminfo | findstr /C:"Virtualization"
# Should show: Virtualization Enabled In Firmware: Yes
```

**Note**: Firecracker in WSL2 typically requires nested virtualization or may not work at all depending on your CPU. Docker is the recommended alternative on Windows.

### Alternative: Use Docker with Hardened Settings

Since Firecracker in WSL2 has limitations, use Docker with security hardening:

```powershell
# Already implemented in vm_pool.rs:
# - --network none
# - --read-only
# - --cap-drop ALL
# - --privileged=false
# - --pids-limit 256

# Run with Docker isolation
apex.bat router-docker
```

## Quick Start (One-Line Setup)

```powershell
# Run in WSL2 to download prebuilt assets
wsl -d Ubuntu-20.04 -e sh -c "cd /tmp && curl -L -o vmlinux https://github.com/firecracker-microvm/firecracker-kata-common-repo/releases/download/v0.0.2/vmlinux && curl -L -o rootfs.ext4 https://github.com/firecracker-microvm/firecracker-kata-common-repo/releases/download/v0.0.2/rootfs.ext4 && chmod +x vmlinux"
```

## Verification Commands

```powershell
# Check WSL2 is running
wsl -l -v

# Check Firecracker version
wsl -d Ubuntu-20.04 firecracker --version

# Check kernel exists
wsl -d Ubuntu-20.04 ls -la /tmp/vmlinux

# Check rootfs exists
wsl -d Ubuntu-20.04 ls -la /tmp/rootfs.ext4
```

## Security Notes

When Firecracker is working, it provides:
- ✅ MicroVM isolation (stronger than containers)
- ✅ Near-native performance
- ✅ No shared kernel with host
- ✅ Minimal attack surface

**Current APEX security for Docker** (working):
- Network isolation (`--network none`)
- Read-only filesystem
- Capability dropping
- Process limits
- Memory/CPU limits

Both provide good security for a personal agent platform.
