# 🚀 DAA Deployment Guide

> **Production-ready deployment strategies for Decentralized Autonomous Agents** - From single-node setups to large-scale distributed systems.

[![Docker](https://img.shields.io/badge/Docker-Ready-blue)](https://docker.com)
[![Kubernetes](https://img.shields.io/badge/Kubernetes-Compatible-green)](https://kubernetes.io)
[![Cloud Native](https://img.shields.io/badge/Cloud%20Native-CNCF-purple)](https://cncf.io)

---

## 📋 Table of Contents

1. [Deployment Overview](#-deployment-overview)
2. [Prerequisites](#-prerequisites)
3. [Single Node Deployment](#-single-node-deployment)
4. [Multi-Node Cluster](#-multi-node-cluster)
5. [Cloud Deployments](#-cloud-deployments)
6. [Configuration Management](#-configuration-management)
7. [Security Hardening](#-security-hardening)
8. [Monitoring & Observability](#-monitoring--observability)
9. [Backup & Recovery](#-backup--recovery)
10. [Troubleshooting](#-troubleshooting)

---

## 🌟 Deployment Overview

DAA supports multiple deployment models to meet different scale and security requirements:

| Deployment Type | Use Case | Complexity | Scalability |
|----------------|----------|------------|-------------|
| **Development** | Local testing | Low | Single node |
| **Staging** | Pre-production testing | Medium | 2-3 nodes |
| **Production** | Live systems | High | 3+ nodes |
| **Enterprise** | Large-scale operations | Very High | 100+ nodes |

### Architecture Patterns

```mermaid
graph TB
    subgraph "Development"
        D1[Single Container]
    end
    
    subgraph "Staging"
        S1[Load Balancer] --> S2[Agent 1]
        S1 --> S3[Agent 2]
        S2 --> S4[Shared DB]
        S3 --> S4
    end
    
    subgraph "Production"
        P1[Ingress] --> P2[Service Mesh]
        P2 --> P3[Agent Cluster]
        P3 --> P4[Distributed Storage]
        P3 --> P5[Monitoring Stack]
    end
```

---

## 🛠️ Prerequisites

### System Requirements

#### Minimum Requirements
- **CPU**: 2 cores, 2.0 GHz
- **RAM**: 4 GB
- **Storage**: 20 GB SSD
- **Network**: 100 Mbps
- **OS**: Linux (Ubuntu 20.04+), macOS, Windows 10+

#### Recommended Requirements
- **CPU**: 4+ cores, 3.0 GHz
- **RAM**: 8+ GB
- **Storage**: 100+ GB NVMe SSD
- **Network**: 1 Gbps
- **OS**: Linux (Ubuntu 22.04 LTS)

### Software Dependencies

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# Install Docker Compose
sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose

# Install kubectl (for Kubernetes deployments)
curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
sudo install -o root -g root -m 0755 kubectl /usr/local/bin/kubectl
```

---

## 💻 Single Node Deployment

Perfect for development, testing, and small-scale production workloads.

### Option 1: Docker Compose (Recommended)

Create a `docker-compose.yml` file:

```yaml
version: '3.8'

services:
  daa-agent:
    image: daa/agent:latest
    container_name: daa-agent
    restart: unless-stopped
    environment:
      # Agent Configuration
      - DAA_AGENT_ID=agent-001
      - DAA_LOG_LEVEL=info
      - DAA_CONFIG_PATH=/config/agent.toml
      
      # Network Configuration
      - DAA_LISTEN_ADDRESS=0.0.0.0:8080
      - DAA_P2P_PORT=9090
      - DAA_BOOTSTRAP_PEERS=bootstrap1.daa.network:9090,bootstrap2.daa.network:9090
      
      # Database Configuration
      - DAA_DB_URL=postgresql://daa:password@postgres:5432/daa
      - DAA_REDIS_URL=redis://redis:6379
      
      # AI Integration
      - ANTHROPIC_API_KEY=${ANTHROPIC_API_KEY}
      - DAA_AI_MODEL=claude-3-sonnet-20240229
      
      # Blockchain Configuration
      - DAA_ETHEREUM_RPC_URL=${ETHEREUM_RPC_URL}
      - DAA_SOLANA_RPC_URL=${SOLANA_RPC_URL}
    
    volumes:
      - ./config:/config:ro
      - ./data:/data
      - ./logs:/logs
    
    ports:
      - "8080:8080"   # HTTP API
      - "9090:9090"   # P2P Network
      - "3000:3000"   # Dashboard (optional)
    
    depends_on:
      - postgres
      - redis
    
    networks:
      - daa-network
    
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 4G
        reservations:
          cpus: '1'
          memory: 2G

  postgres:
    image: postgres:15-alpine
    container_name: daa-postgres
    restart: unless-stopped
    environment:
      - POSTGRES_DB=daa
      - POSTGRES_USER=daa
      - POSTGRES_PASSWORD=password
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./sql/init.sql:/docker-entrypoint-initdb.d/init.sql:ro
    ports:
      - "5432:5432"
    networks:
      - daa-network

  redis:
    image: redis:7-alpine
    container_name: daa-redis
    restart: unless-stopped
    command: redis-server --appendonly yes --requirepass password
    volumes:
      - redis_data:/data
    ports:
      - "6379:6379"
    networks:
      - daa-network

  monitoring:
    image: prom/prometheus:latest
    container_name: daa-prometheus
    restart: unless-stopped
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - prometheus_data:/prometheus
    ports:
      - "9090:9090"
    networks:
      - daa-network

  grafana:
    image: grafana/grafana:latest
    container_name: daa-grafana
    restart: unless-stopped
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - grafana_data:/var/lib/grafana
      - ./monitoring/grafana:/etc/grafana/provisioning
    ports:
      - "3001:3000"
    networks:
      - daa-network

volumes:
  postgres_data:
  redis_data:
  prometheus_data:
  grafana_data:

networks:
  daa-network:
    driver: bridge
```

### Configuration Files

Create the configuration directory structure:

```bash
mkdir -p config monitoring/grafana/{dashboards,datasources}
```

Create `config/agent.toml`:

```toml
[agent]
id = "agent-001"
name = "Treasury Agent"
description = "Autonomous treasury management agent"

[network]
listen_address = "0.0.0.0:8080"
p2p_port = 9090
bootstrap_peers = [
    "bootstrap1.daa.network:9090",
    "bootstrap2.daa.network:9090"
]
max_peers = 50

[orchestrator]
workflow_timeout = "300s"
max_concurrent_workflows = 10
autonomy_interval = "60s"

[rules]
rule_file = "/config/rules.yaml"
audit_enabled = true
audit_retention = "30d"

[economy]
initial_balance = 100000
risk_tolerance = 0.2
rebalance_threshold = 0.1
fee_optimization = true

[ai]
provider = "anthropic"
model = "claude-3-sonnet-20240229"
max_tokens = 4000
temperature = 0.7

[chain]
ethereum_enabled = true
solana_enabled = true
cosmos_enabled = false

[monitoring]
metrics_enabled = true
health_check_interval = "30s"
log_level = "info"

[security]
tls_enabled = true
cert_file = "/config/tls/cert.pem"
key_file = "/config/tls/key.pem"
```

### Deployment Commands

```bash
# Clone the repository
git clone https://github.com/ruvnet/daa.git
cd daa

# Set up environment variables
cp .env.example .env
# Edit .env with your configuration

# Create required directories
mkdir -p {config,data,logs,monitoring}

# Deploy the stack
docker-compose up -d

# Check status
docker-compose ps

# View logs
docker-compose logs -f daa-agent

# Stop the deployment
docker-compose down
```

### Option 2: Native Binary

For optimal performance or custom configurations:

```bash
# Install DAA CLI
cargo install daa-cli

# Initialize project
daa-cli init my-agent --template treasury

# Configure agent
cd my-agent
vim config/agent.toml

# Start agent
daa-cli start --config config/agent.toml --log-level info
```

---

## 🏗️ Multi-Node Cluster

For production environments requiring high availability and scalability.

### Kubernetes Deployment

#### Namespace Setup

```yaml
# namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: daa-system
  labels:
    name: daa-system
    istio-injection: enabled
```

#### ConfigMap

```yaml
# configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: daa-config
  namespace: daa-system
data:
  agent.toml: |
    [agent]
    id = "${DAA_AGENT_ID}"
    name = "${DAA_AGENT_NAME}"
    
    [network]
    listen_address = "0.0.0.0:8080"
    p2p_port = 9090
    bootstrap_peers = [
        "daa-bootstrap-1.daa-system.svc.cluster.local:9090",
        "daa-bootstrap-2.daa-system.svc.cluster.local:9090"
    ]
    
    [orchestrator]
    workflow_timeout = "300s"
    max_concurrent_workflows = 10
    
    [database]
    url = "postgresql://daa:${DB_PASSWORD}@postgres.daa-system.svc.cluster.local:5432/daa"
    
    [cache]
    redis_url = "redis://redis.daa-system.svc.cluster.local:6379"
```

#### Deployment

```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: daa-agents
  namespace: daa-system
spec:
  replicas: 3
  selector:
    matchLabels:
      app: daa-agent
  template:
    metadata:
      labels:
        app: daa-agent
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "9091"
        prometheus.io/path: "/metrics"
    spec:
      serviceAccountName: daa-agent
      securityContext:
        runAsNonRoot: true
        runAsUser: 1000
        fsGroup: 2000
      
      containers:
      - name: daa-agent
        image: daa/agent:v0.2.0
        imagePullPolicy: IfNotPresent
        
        env:
        - name: DAA_AGENT_ID
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
        - name: DAA_AGENT_NAME
          value: "production-agent"
        - name: DAA_CONFIG_PATH
          value: "/config/agent.toml"
        - name: DB_PASSWORD
          valueFrom:
            secretKeyRef:
              name: daa-secrets
              key: db-password
        - name: ANTHROPIC_API_KEY
          valueFrom:
            secretKeyRef:
              name: daa-secrets
              key: anthropic-api-key
        
        volumeMounts:
        - name: config
          mountPath: /config
          readOnly: true
        - name: data
          mountPath: /data
        - name: tls-certs
          mountPath: /certs
          readOnly: true
        
        ports:
        - containerPort: 8080
          name: http
        - containerPort: 9090
          name: p2p
        - containerPort: 9091
          name: metrics
        
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
        
        resources:
          requests:
            memory: "1Gi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "1000m"
      
      volumes:
      - name: config
        configMap:
          name: daa-config
      - name: data
        persistentVolumeClaim:
          claimName: daa-data
      - name: tls-certs
        secret:
          secretName: daa-tls
```

#### Services

```yaml
# services.yaml
apiVersion: v1
kind: Service
metadata:
  name: daa-agents
  namespace: daa-system
  labels:
    app: daa-agent
spec:
  selector:
    app: daa-agent
  ports:
  - name: http
    port: 80
    targetPort: 8080
  - name: p2p
    port: 9090
    targetPort: 9090
  type: ClusterIP

---
apiVersion: v1
kind: Service
metadata:
  name: daa-agents-headless
  namespace: daa-system
  labels:
    app: daa-agent
spec:
  selector:
    app: daa-agent
  ports:
  - name: p2p
    port: 9090
    targetPort: 9090
  clusterIP: None
```

#### Ingress

```yaml
# ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: daa-ingress
  namespace: daa-system
  annotations:
    kubernetes.io/ingress.class: nginx
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/rate-limit: "100"
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
spec:
  tls:
  - hosts:
    - api.daa.yourdomain.com
    secretName: daa-tls
  rules:
  - host: api.daa.yourdomain.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: daa-agents
            port:
              number: 80
```

### Deployment Script

```bash
#!/bin/bash
# deploy.sh

set -e

# Configuration
NAMESPACE="daa-system"
HELM_RELEASE="daa"

# Create namespace
kubectl create namespace $NAMESPACE --dry-run=client -o yaml | kubectl apply -f -

# Apply secrets
kubectl create secret generic daa-secrets \
  --from-literal=db-password="$(openssl rand -base64 32)" \
  --from-literal=anthropic-api-key="$ANTHROPIC_API_KEY" \
  --namespace=$NAMESPACE \
  --dry-run=client -o yaml | kubectl apply -f -

# Apply configurations
kubectl apply -f k8s/

# Wait for deployment
kubectl rollout status deployment/daa-agents -n $NAMESPACE

# Verify deployment
kubectl get pods -n $NAMESPACE
kubectl get services -n $NAMESPACE
kubectl get ingress -n $NAMESPACE

echo "Deployment completed successfully!"
```

---

## ☁️ Cloud Deployments

### AWS EKS

```bash
# Create EKS cluster
eksctl create cluster \
  --name daa-cluster \
  --region us-west-2 \
  --node-type m5.large \
  --nodes 3 \
  --nodes-min 1 \
  --nodes-max 10 \
  --with-oidc \
  --ssh-access \
  --ssh-public-key my-key

# Install AWS Load Balancer Controller
helm repo add eks https://aws.github.io/eks-charts
helm install aws-load-balancer-controller eks/aws-load-balancer-controller \
  --set clusterName=daa-cluster \
  --set serviceAccount.create=false \
  --set serviceAccount.name=aws-load-balancer-controller \
  -n kube-system

# Deploy DAA
kubectl apply -f k8s/aws/
```

### Google GKE

```bash
# Create GKE cluster
gcloud container clusters create daa-cluster \
  --zone us-central1-a \
  --machine-type n1-standard-2 \
  --num-nodes 3 \
  --enable-autoscaling \
  --min-nodes 1 \
  --max-nodes 10 \
  --enable-autorepair \
  --enable-autoupgrade

# Get credentials
gcloud container clusters get-credentials daa-cluster --zone us-central1-a

# Deploy DAA
kubectl apply -f k8s/gcp/
```

### Azure AKS

```bash
# Create resource group
az group create --name daa-rg --location eastus

# Create AKS cluster
az aks create \
  --resource-group daa-rg \
  --name daa-cluster \
  --node-count 3 \
  --enable-addons monitoring \
  --generate-ssh-keys

# Get credentials
az aks get-credentials --resource-group daa-rg --name daa-cluster

# Deploy DAA
kubectl apply -f k8s/azure/
```

---

## ⚙️ Configuration Management

### Environment-Specific Configurations

Use Helm for managing different environments:

```yaml
# values-dev.yaml
replicaCount: 1
image:
  tag: "latest"
resources:
  limits:
    cpu: 500m
    memory: 1Gi
ingress:
  enabled: false

# values-prod.yaml
replicaCount: 3
image:
  tag: "v0.2.0"
resources:
  limits:
    cpu: 2000m
    memory: 4Gi
ingress:
  enabled: true
  host: api.daa.yourdomain.com
```

### Secrets Management

#### Using Kubernetes Secrets

```bash
# Create TLS certificate
kubectl create secret tls daa-tls \
  --cert=path/to/tls.crt \
  --key=path/to/tls.key \
  --namespace=daa-system

# Create application secrets
kubectl create secret generic daa-app-secrets \
  --from-literal=database-url="postgresql://..." \
  --from-literal=api-key="..." \
  --namespace=daa-system
```

#### Using External Secrets Operator

```yaml
# external-secret.yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: daa-secrets
  namespace: daa-system
spec:
  refreshInterval: 15s
  secretStoreRef:
    name: vault-backend
    kind: SecretStore
  target:
    name: daa-secrets
    creationPolicy: Owner
  data:
  - secretKey: db-password
    remoteRef:
      key: secret/daa
      property: db-password
  - secretKey: anthropic-api-key
    remoteRef:
      key: secret/daa
      property: anthropic-api-key
```

---

## 🔒 Security Hardening

### Network Security

#### Network Policies

```yaml
# network-policy.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: daa-network-policy
  namespace: daa-system
spec:
  podSelector:
    matchLabels:
      app: daa-agent
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: ingress-nginx
    ports:
    - protocol: TCP
      port: 8080
  - from:
    - podSelector:
        matchLabels:
          app: daa-agent
    ports:
    - protocol: TCP
      port: 9090
  egress:
  - to:
    - namespaceSelector:
        matchLabels:
          name: kube-system
    ports:
    - protocol: TCP
      port: 53
    - protocol: UDP
      port: 53
  - to: []
    ports:
    - protocol: TCP
      port: 443
```

### Pod Security

#### Pod Security Standards

```yaml
# pod-security-policy.yaml
apiVersion: policy/v1beta1
kind: PodSecurityPolicy
metadata:
  name: daa-psp
spec:
  privileged: false
  allowPrivilegeEscalation: false
  requiredDropCapabilities:
    - ALL
  volumes:
    - 'configMap'
    - 'emptyDir'
    - 'projected'
    - 'secret'
    - 'downwardAPI'
    - 'persistentVolumeClaim'
  runAsUser:
    rule: 'MustRunAsNonRoot'
  seLinux:
    rule: 'RunAsAny'
  fsGroup:
    rule: 'RunAsAny'
```

### Image Security

```dockerfile
# Dockerfile
FROM rust:1.70-slim as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM gcr.io/distroless/cc-debian11
COPY --from=builder /app/target/release/daa-agent /app/daa-agent

USER 1000:1000
EXPOSE 8080 9090

ENTRYPOINT ["/app/daa-agent"]
```

---

## 📊 Monitoring & Observability

### Prometheus Configuration

```yaml
# prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "daa_rules.yml"

scrape_configs:
  - job_name: 'daa-agents'
    kubernetes_sd_configs:
      - role: pod
    relabel_configs:
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_scrape]
        action: keep
        regex: true
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_path]
        action: replace
        target_label: __metrics_path__
        regex: (.+)
      - source_labels: [__address__, __meta_kubernetes_pod_annotation_prometheus_io_port]
        action: replace
        regex: ([^:]+)(?::\d+)?;(\d+)
        replacement: $1:$2
        target_label: __address__
```

### Grafana Dashboard

```json
{
  "dashboard": {
    "title": "DAA Agent Metrics",
    "panels": [
      {
        "title": "Agent Status",
        "type": "stat",
        "targets": [
          {
            "expr": "daa_agent_up",
            "legendFormat": "{{ agent_id }}"
          }
        ]
      },
      {
        "title": "Workflow Execution Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(daa_workflows_executed_total[5m])",
            "legendFormat": "{{ agent_id }}"
          }
        ]
      },
      {
        "title": "Economic Metrics",
        "type": "graph",
        "targets": [
          {
            "expr": "daa_balance_total",
            "legendFormat": "Balance"
          },
          {
            "expr": "daa_transactions_value_total",
            "legendFormat": "Transaction Volume"
          }
        ]
      }
    ]
  }
}
```

### Logging Configuration

```yaml
# fluent-bit.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: fluent-bit-config
  namespace: daa-system
data:
  fluent-bit.conf: |
    [SERVICE]
        Flush         1
        Log_Level     info
        Daemon        off
        Parsers_File  parsers.conf
    
    [INPUT]
        Name              tail
        Path              /var/log/containers/*daa-agent*.log
        multiline.parser  docker, cri
        Tag               daa.*
        Mem_Buf_Limit     50MB
        Skip_Long_Lines   On
    
    [OUTPUT]
        Name  es
        Match daa.*
        Host  elasticsearch.logging.svc.cluster.local
        Port  9200
        Index daa-logs
```

---

## 💾 Backup & Recovery

### Database Backup

```bash
#!/bin/bash
# backup.sh

BACKUP_DIR="/backups"
DATE=$(date +%Y%m%d_%H%M%S)
DB_HOST="postgres.daa-system.svc.cluster.local"
DB_NAME="daa"
DB_USER="daa"

# Create backup
pg_dump -h $DB_HOST -U $DB_USER -d $DB_NAME | gzip > $BACKUP_DIR/daa_backup_$DATE.sql.gz

# Upload to S3
aws s3 cp $BACKUP_DIR/daa_backup_$DATE.sql.gz s3://daa-backups/database/

# Clean up old backups
find $BACKUP_DIR -name "daa_backup_*.sql.gz" -mtime +7 -delete
```

### Configuration Backup

```bash
#!/bin/bash
# backup-config.sh

kubectl get configmaps -n daa-system -o yaml > configmaps-backup.yaml
kubectl get secrets -n daa-system -o yaml > secrets-backup.yaml
kubectl get persistentvolumeclaims -n daa-system -o yaml > pvc-backup.yaml

# Create tarball
tar -czf daa-config-backup-$(date +%Y%m%d).tar.gz *.yaml

# Upload to storage
aws s3 cp daa-config-backup-$(date +%Y%m%d).tar.gz s3://daa-backups/config/
```

### Disaster Recovery

```bash
#!/bin/bash
# disaster-recovery.sh

set -e

echo "Starting disaster recovery..."

# Restore database
kubectl exec -i postgres-0 -n daa-system -- psql -U daa -d daa < daa_backup_latest.sql

# Restore configurations
kubectl apply -f configmaps-backup.yaml
kubectl apply -f secrets-backup.yaml
kubectl apply -f pvc-backup.yaml

# Restart agents
kubectl rollout restart deployment/daa-agents -n daa-system

echo "Disaster recovery completed successfully!"
```

---

## 🔧 Troubleshooting

### Common Issues

#### Agent Won't Start

```bash
# Check logs
kubectl logs -f deployment/daa-agents -n daa-system

# Common issues:
# 1. Configuration errors
kubectl describe configmap daa-config -n daa-system

# 2. Secret issues
kubectl get secrets -n daa-system

# 3. Resource constraints
kubectl describe nodes
```

#### Network Connectivity Issues

```bash
# Test P2P connectivity
kubectl exec -it daa-agent-0 -n daa-system -- telnet daa-agent-1 9090

# Check network policies
kubectl get networkpolicies -n daa-system

# Test DNS resolution
kubectl exec -it daa-agent-0 -n daa-system -- nslookup postgres.daa-system.svc.cluster.local
```

#### Performance Issues

```bash
# Check resource usage
kubectl top pods -n daa-system

# Check metrics
curl -s http://localhost:9091/metrics | grep daa_

# Scale up if needed
kubectl scale deployment daa-agents --replicas=5 -n daa-system
```

### Health Checks

```bash
#!/bin/bash
# health-check.sh

echo "DAA System Health Check"
echo "======================"

# Check namespace
kubectl get namespace daa-system

# Check deployments
kubectl get deployments -n daa-system

# Check services
kubectl get services -n daa-system

# Check ingress
kubectl get ingress -n daa-system

# Check pods
kubectl get pods -n daa-system

# Check PVCs
kubectl get pvc -n daa-system

# Test API endpoint
if curl -s -o /dev/null -w "%{http_code}" http://api.daa.yourdomain.com/health | grep -q "200"; then
    echo "✓ API endpoint is healthy"
else
    echo "✗ API endpoint is not responding"
fi

echo "Health check completed!"
```

---

## 📈 Scaling Strategies

### Horizontal Pod Autoscaler

```yaml
# hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: daa-agents-hpa
  namespace: daa-system
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: daa-agents
  minReplicas: 3
  maxReplicas: 20
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 300
      policies:
      - type: Percent
        value: 100
        periodSeconds: 60
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
      - type: Percent
        value: 10
        periodSeconds: 60
```

### Vertical Pod Autoscaler

```yaml
# vpa.yaml
apiVersion: autoscaling.k8s.io/v1
kind: VerticalPodAutoscaler
metadata:
  name: daa-agents-vpa
  namespace: daa-system
spec:
  targetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: daa-agents
  updatePolicy:
    updateMode: "Auto"
  resourcePolicy:
    containerPolicies:
    - containerName: daa-agent
      maxAllowed:
        cpu: 4000m
        memory: 8Gi
      minAllowed:
        cpu: 100m
        memory: 128Mi
```

---

## 🔗 Related Documentation

- [Configuration Reference](./configuration.md) - Detailed configuration options
- [Monitoring Guide](./monitoring.md) - Comprehensive monitoring setup
- [Security Guide](../architecture/security.md) - Security best practices
- [Troubleshooting Guide](../troubleshooting/README.md) - Problem resolution

---

## 📞 Support

For deployment-related questions:
- [GitHub Issues](https://github.com/ruvnet/daa/issues)
- [Discord Community](https://discord.gg/daa)
- Email: support@daa.dev

---

*This deployment guide is regularly updated with the latest best practices and deployment patterns.*