# APEX Production Hardening Guide

> **Status**: Planning  
> **Version**: 1.3.0

---

## Overview

This document outlines security hardening requirements for deploying APEX in production environments.

---

## 1. Linux Kernel Security

### 1.1 Seccomp Profile

APEX should run with a restrictive seccomp profile to limit syscalls.

**Recommended Profile** (`apex-seccomp.json`):

```json
{
  "defaultAction": "SCMP_ACT_ERRNO",
  "syscalls": [
    {"names": ["read", "write", "close", "stat", "fstat", "mmap", "mprotect", "brk", "rt_sigaction", "rt_sigprocmask", "ioctl", "pread64", "pwrite64", "readv", "writev", "access", "pipe", "select", "mremap", "msync", "mincore", "madvise", "shmget", "shmat", "shmctl", "dup", "dup2", "pause", "nanosleep", "getitimer", "alarm", "setitimer", "getpid", "socket", "connect", "accept", "sendto", "recvfrom", "sendmsg", "recvmsg", "shutdown", "bind", "listen", "getsockname", "getpeername", "socketpair", "setsockopt", "getsockopt", "clone", "fork", "vfork", "execve", "exit", "wait4", "kill", "uname", "semget", "semop", "semctl", "shmdt", "msgget", "msgsnd", "msgrcv", "msgctl", "msgctl", "fcntl", "flock", "fsync", "fdatasync", "truncate", "ftruncate", "getdents", "getcwd", "chdir", "fchdir", "rename", "mkdir", "rmdir", "creat", "link", "unlink", "symlink", "readlink", "chmod", "fchmod", "chown", "fchown", "lchown", "umask", "gettimeofday", "getrlimit", "getrusage", "sysinfo", "times", "ptrace", "getuid", "syslog", "getgid", "setuid", "setgid", "geteuid", "getegid", "setpgid", "getppid", "getpgrp", "setsid", "setreuid", "setregid", "getgroups", "setgroups", "setresuid", "getresuid", "setresgid", "getresgid", "getpgid", "setfsuid", "setfsgid", "getsid", "capget", "capset", "rt_sigpending", "rt_sigtimedwait", "rt_sigqueueinfo", "rt_sigsuspend", "sigaltstack", "utime", "mknod", "uselib", "personality", "ustat", "statfs", "fstatfs", "sysfs", "getpriority", "setpriority", "sched_setparam", "sched_getparam", "sched_setscheduler", "sched_getscheduler", "sched_get_priority_max", "sched_get_priority_min", "sched_rr_get_interval", "mlock", "munlock", "mlockall", "munlockall", "vhangup", "modify_ldt", "pivot_root", "prctl", "arch_prctl", "adjtimex", "setrlimit", "chroot", "sync", "acct", "settimeofday", "mount", "umount2", "swapon", "swapoff", "reboot", "setdomainname", "init_module", "delete_module", "quotactl", "gettid", "reparent", "lookup_dcookie", "setsid", "setlogmask", "mkfifo", "socketcall", "syslog", "setitimer", "getitimer", "unistd"], "action": "SCMP_ACT_ALLOW"}
  ]
}
```

**Apply with Docker:**
```bash
docker run --security-opt seccomp=apex-seccomp.json apex-router:latest
```

---

## 2. AppArmor Profile

### 2.1 Container Profile (`apex-router`)

```apparmor
#include <tunables/global>

profile apex-router flags=(attach_disconnected) {
  #include <abstractions/base>
  #include <abstractions/nameservices>
  #include <abstractions/user-tmp>
  
  # Deny dangerous paths
  deny /proc/sys/** w,
  deny /sys/** w,
  deny /dev/** w,
  
  # Allow specific paths
  /opt/apex/** r,
  /var/lib/apex/** rw,
  /var/log/apex/** rw,
  
  # Network
  network inet stream,
  network inet6 stream,
  network inet dgram,
  
  # Capabilities
  capability net_bind_service,
  capability sys_nice,
  capability sys_resource,
}
```

---

## 3. SIEM Integration

### 3.1 Audit Logging Format

APEX should emit JSON-structured audit logs for SIEM ingestion:

```json
{
  "timestamp": "2026-03-10T12:00:00Z",
  "level": "info",
  "event_type": "task_created",
  "task_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
  "user_tier": "T2",
  "skill_name": "code.generate",
  "source_ip": "192.168.1.100",
  "metadata": {
    "channel": "default",
    "project": "apex"
  }
}
```

### 3.2 Environment Variables

```bash
# Enable structured JSON logging
APEX_JSON_LOGS=1

# Configure audit destination
APEX_AUDIT_FORMAT=json
APEX_AUDIT_ENDPOINT=https://your-siem.example.com/ingest
```

### 3.3 Security Events to Log

| Event | Severity | Description |
|-------|----------|-------------|
| `task_created` | Info | New task created |
| `task_executed` | Info | Task executed |
| `skill_executed` | Info | Skill executed |
| `auth_failed` | Warning | Authentication failed |
| `tier_escalation` | Warning | Permission tier escalation |
| `totp_failed` | Warning | TOTP verification failed |
| `shell_executed` | Critical | Shell command executed |
| `data_access` | Info | Sensitive data accessed |

---

## 4. Network Security

### 4.1 Ingress Configuration

```yaml
# Kubernetes Ingress example
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: apex-ingress
  annotations:
    nginx.ingress.kubernetes.io/rate-limit: "100"
    nginx.ingress.kubernetes.io/proxy-body-size: "10m"
spec:
  tls:
    - hosts:
        - apex.example.com
      secretName: apex-tls
  rules:
    - host: apex.example.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: apex-router
                port:
                  number: 3000
```

### 4.2 Network Policies

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: apex-network-policy
spec:
  podSelector:
    matchLabels:
      app: apex-router
  policyTypes:
    - Ingress
    - Egress
  ingress:
    - from:
        - podSelector:
            matchLabels:
              app: apex-gateway
      ports:
        - protocol: TCP
          port: 3000
  egress:
    - to:
        - namespaceSelector: {}
      ports:
        - protocol: TCP
          port: 5432  # PostgreSQL
    - to:
        - podSelector: {}
      ports:
        - protocol: TCP
          port: 8080  # LLM
```

---

## 5. Secrets Management

### 5.1 Production Secrets

Never use default secrets in production:

```bash
# Generate secure HMAC secret
openssl rand -hex 32

# Set environment
export APEX_SHARED_SECRET="your-secure-secret-here"
export APEX_AUTH_DISABLED=0

# Use external secrets
APEX_SECRET_STORE=aws-secrets-manager
AWS_SECRET_NAME=apex/production
```

### 5.2 Encryption Keys

APEX uses AES-256-GCM for secret storage. Ensure encryption keys are properly managed:

- Use hardware security modules (HSM) in production
- Rotate keys regularly (recommended: 90 days)
- Never commit keys to version control

---

## 6. Monitoring & Alerting

### 6.1 Key Metrics

| Metric | Threshold | Action |
|--------|-----------|--------|
| `auth_failures` | > 10/min | Alert security team |
| `totp_failures` | > 5/min | Alert security team |
| `task_errors` | > 50/min | Alert ops team |
| `rate_limited` | > 1000/min | Review rate limits |
| `execution_time` | > 30s | Optimize/skipped |

### 6.2 Prometheus Alerts

```yaml
groups:
  - name: apex-security
    rules:
      - alert: ApexAuthFailuresHigh
        expr: rate(apex_auth_failures_total[5m]) > 10
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "High auth failure rate"
          
      - alert: ApexTOTPFailuresHigh
        expr: rate(apex_totp_failures_total[5m]) > 5
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "High TOTP failure rate"
```

---

## 7. Deployment Checklist

- [ ] Generate new HMAC secret (32+ random bytes)
- [ ] Enable authentication (`APEX_AUTH_DISABLED=0`)
- [ ] Configure TOTP issuer name
- [ ] Set up log aggregation (JSON logs)
- [ ] Deploy seccomp profile
- [ ] Deploy AppArmor profile (if applicable)
- [ ] Configure network policies
- [ ] Set up secrets management
- [ ] Configure monitoring alerts
- [ ] Test authentication flow
- [ ] Test TOTP verification
- [ ] Test rate limiting
- [ ] Verify audit logs are generated

---

## 8. Container Security

### 8.1 Dockerfile Recommendations

```dockerfile
# Run as non-root user
USER nonroot:nonroot

# Read-only filesystem
READONLY=true

# Drop all capabilities
CAP_DROP ALL
CAP_ADD NET_BIND_SERVICE

# No new privileges
NO_NEW_PRIVS=true

# Seccomp profile (external)
--security-opt seccomp=apex-seccomp.json

# AppArmor profile (external)
--security-opt apparmor=apex-router
```

---

*Document Version: 1.0*  
*Last Updated: 2026-03-10*
