# APEX Kubernetes Deployment Guide

> ⚠️ **Status: PRE-ALPHA** - This is an experimental research project. Not production ready.

This guide covers deploying APEX on Kubernetes for production use.

## Prerequisites

- Kubernetes 1.28+
- Helm 3.12+
- PostgreSQL 14+ (external or managed)
- Docker/Containerd
- Optional: NATS for distributed mode

## Architecture

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Gateway   │────▶│   Router    │────▶│   Worker    │
│   (L1)      │     │   (L2/L3)   │     │   (L4/L5)  │
└─────────────┘     └─────────────┘     └─────────────┘
                           │                   │
                           ▼                   ▼
                    ┌─────────────┐     ┌─────────────┐
                    │  PostgreSQL │     │   Skills    │
                    │  (Memory)   │     │   (Local)   │
                    └─────────────┘     └─────────────┘
```

## Quick Start

### Using Helm

```bash
# Add APEX helm repository
helm repo add apex https://apex.github.io/apex-helm
helm repo update

# Install APEX
helm install apex apex/apex \
  --set router.image.tag=v1.3.0 \
  --set postgres.enabled=true \
  --set postgres.host=apex-postgres \
  --set postgres.port=5432
```

### Using kubectl

```bash
kubectl apply -f kubernetes/
```

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `APEX_DATABASE_URL` | PostgreSQL connection string | `postgres://apex:apex@localhost:5432/apex` |
| `APEX_NATS_ENABLED` | Enable NATS | `false` |
| `APEX_NATS_URL` | NATS server URL | `nats://localhost:4222` |
| `APEX_SHARED_SECRET` | HMAC signing secret | (required) |
| `APEX_USE_LLM` | Enable LLM | `false` |

### Resource Limits

```yaml
resources:
  limits:
    cpu: "2"
    memory: "4Gi"
  requests:
    cpu: "500m"
    memory: "1Gi"
```

## Kubernetes Manifests

### Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: apex-router
spec:
  replicas: 2
  selector:
    matchLabels:
      app: apex-router
  template:
    metadata:
      labels:
        app: apex-router
    spec:
      containers:
      - name: router
        image: apex-router:v1.3.0
        ports:
        - containerPort: 3000
        env:
        - name: APEX_DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: apex-secrets
              key: database-url
        - name: APEX_SHARED_SECRET
          valueFrom:
            secretKeyRef:
              name: apex-secrets
              key: shared-secret
        resources:
          limits:
            cpu: "2"
            memory: "4Gi"
```

### Service

```yaml
apiVersion: v1
kind: Service
metadata:
  name: apex-router
spec:
  selector:
    app: apex-router
  ports:
  - port: 80
    targetPort: 3000
  type: LoadBalancer
```

### Ingress

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: apex-ingress
  annotations:
    nginx.ingress.kubernetes.io/rewrite-target: /
spec:
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
              number: 80
```

### ConfigMap

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: apex-config
data:
  APEX_PORT: "3000"
  APEX_NATS_ENABLED: "false"
  APEX_LOG_LEVEL: "info"
```

### Secrets

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: apex-secrets
type: Opaque
stringData:
  database-url: "postgres://user:pass@host:5432/apex"
  shared-secret: "your-secret-here"
```

## Scaling

### Horizontal Pod Autoscaler

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: apex-router-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: apex-router
  minReplicas: 2
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
```

### Worker Scaling

Workers can be scaled independently:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: apex-worker
spec:
  replicas: 4  # Scale based on workload
```

## Persistence

### PostgreSQL

Use a managed PostgreSQL service (Cloud SQL, RDS, etc.) or deploy manually:

```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: apex-postgres
spec:
  serviceName: postgres
  replicas: 1
  selector:
    matchLabels:
      app: postgres
  template:
    metadata:
      labels:
        app: postgres
    spec:
      containers:
      - name: postgres
        image: postgres:14
        env:
        - name: POSTGRES_DB
          value: apex
        - name: POSTGRES_USER
          value: apex
        - name: POSTGRES_PASSWORD
          valueFrom:
            secretKeyRef:
              name: apex-secrets
              key: postgres-password
        volumeMounts:
        - name: data
          mountPath: /var/lib/postgresql/data
  volumeClaimTemplates:
  - metadata:
      name: data
    spec:
      accessModes: ["ReadWriteOnce"]
      resources:
        requests:
          storage: 10Gi
```

## Health Checks

```yaml
livenessProbe:
  httpGet:
    path: /health
    port: 3000
  initialDelaySeconds: 30
  periodSeconds: 10
readinessProbe:
  httpGet:
    path: /health
    port: 3000
  initialDelaySeconds: 5
  periodSeconds: 5
```

## Monitoring

### Prometheus scraping

```yaml
annotations:
  prometheus.io/scrape: "true"
  prometheus.io/port: "3000"
  prometheus.io/path: "/api/v1/metrics"
```

### Grafana Dashboard

Import `kubernetes/apex-dashboard.json` for pre-built dashboards.

## Security

### Network Policies

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
          app: ingress-controller
    ports:
    - protocol: TCP
      port: 3000
  egress:
  - to:
    - podSelector:
        matchLabels:
          app: postgres
    ports:
    - protocol: TCP
      port: 5432
```

### RBAC

```yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: apex-role
rules:
- apiGroups: [""]
  resources: ["configmaps"]
  verbs: ["get", "list", "watch"]
```

## Distributed Mode with NATS

For multi-instance deployment:

```yaml
env:
- name: APEX_NATS_ENABLED
  value: "true"
- name: APEX_NATS_URL
  value: "nats://nats:4222"
```

Deploy NATS:

```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: nats
spec:
  replicas: 3
  # ... NATS configuration
```

## Troubleshooting

### Check logs

```bash
kubectl logs -l app=apex-router
```

### Check status

```bash
kubectl get pods -l app=apex-router
```

### Port forward for debugging

```bash
kubectl port-forward svc/apex-router 3000:80
```

## Files

- `kubernetes/deployment.yaml` - Main deployment
- `kubernetes/service.yaml` - Service definition
- `kubernetes/ingress.yaml` - Ingress configuration
- `kubernetes/configmap.yaml` - ConfigMap
- `kubernetes/secrets.yaml` - Secrets template
- `kubernetes/hpa.yaml` - Autoscaling
- `kubernetes/network-policy.yaml` - Network policies
