# Kubernetes Deployment

This guide covers deploying DataSynth on Kubernetes using the included Helm chart or raw manifests.

## Prerequisites

- Kubernetes 1.27+
- Helm 3.12+ (for Helm-based deployment)
- `kubectl` configured for your cluster
- A container registry accessible from the cluster
- Metrics Server installed (for HPA)

## Helm Chart

The Helm chart is located at `deploy/helm/datasynth/` and manages all Kubernetes resources.

### Quick Install

```bash
# From the repository root
helm install datasynth ./deploy/helm/datasynth \
  --namespace datasynth \
  --create-namespace
```

### Install with Custom Values

```bash
helm install datasynth ./deploy/helm/datasynth \
  --namespace datasynth \
  --create-namespace \
  --set image.repository=your-registry.example.com/datasynth-server \
  --set image.tag=0.5.0 \
  --set autoscaling.minReplicas=3 \
  --set autoscaling.maxReplicas=15
```

### Upgrade

```bash
helm upgrade datasynth ./deploy/helm/datasynth \
  --namespace datasynth \
  --reuse-values \
  --set image.tag=0.6.0
```

### Uninstall

```bash
helm uninstall datasynth --namespace datasynth
```

## Chart Reference

### values.yaml Key Parameters

| Parameter | Default | Description |
|-----------|---------|-------------|
| `replicaCount` | `2` | Initial replicas (ignored when HPA is enabled) |
| `image.repository` | `datasynth/datasynth-server` | Container image repository |
| `image.tag` | `0.5.0` | Image tag |
| `service.type` | `ClusterIP` | Service type |
| `service.restPort` | `3000` | REST API port |
| `service.grpcPort` | `50051` | gRPC port |
| `resources.requests.cpu` | `500m` | CPU request |
| `resources.requests.memory` | `512Mi` | Memory request |
| `resources.limits.cpu` | `2` | CPU limit |
| `resources.limits.memory` | `2Gi` | Memory limit |
| `autoscaling.enabled` | `true` | Enable HPA |
| `autoscaling.minReplicas` | `2` | Minimum replicas |
| `autoscaling.maxReplicas` | `10` | Maximum replicas |
| `autoscaling.targetCPUUtilizationPercentage` | `70` | CPU scaling target |
| `podDisruptionBudget.enabled` | `true` | Enable PDB |
| `podDisruptionBudget.minAvailable` | `1` | Minimum available pods |
| `apiKeys` | `[]` | API keys (stored in a Secret) |
| `config.enabled` | `false` | Mount DataSynth YAML config via ConfigMap |
| `redis.enabled` | `false` | Deploy Redis sidecar for distributed rate limiting |
| `serviceMonitor.enabled` | `false` | Create Prometheus ServiceMonitor |
| `ingress.enabled` | `false` | Enable Ingress resource |

### Authentication

API keys are stored in a Kubernetes Secret and injected via the `DATASYNTH_API_KEYS` environment variable:

```yaml
# values-production.yaml
apiKeys:
  - "your-secure-api-key-1"
  - "your-secure-api-key-2"
```

For external secret management, use the [External Secrets Operator](https://external-secrets.io/) or mount from a Vault sidecar. See [Security Hardening](security-hardening.md) for details.

### DataSynth Configuration via ConfigMap

To inject a DataSynth generation config into the pods:

```yaml
config:
  enabled: true
  content: |
    global:
      industry: manufacturing
      start_date: "2024-01-01"
      period_months: 12
      seed: 42
    companies:
      - code: "1000"
        name: "Manufacturing Corp"
        currency: USD
        country: US
        annual_transaction_volume: 100000
```

The config is mounted at `/etc/datasynth/datasynth.yaml` as a read-only volume.

## Health Probes

The Helm chart configures three probes:

| Probe | Endpoint | Initial Delay | Period | Failure Threshold |
|-------|----------|---------------|--------|-------------------|
| Startup | `GET /live` | 5s | 5s | 30 (= 2.5 min max startup) |
| Liveness | `GET /live` | 15s | 20s | 3 |
| Readiness | `GET /ready` | 5s | 10s | 3 |

The readiness probe checks configuration validity, memory usage, and disk availability. A pod reporting not-ready is removed from Service endpoints until it recovers.

## Horizontal Pod Autoscaler (HPA)

The chart creates an HPA by default targeting 70% CPU utilization:

```yaml
autoscaling:
  enabled: true
  minReplicas: 2
  maxReplicas: 10
  targetCPUUtilizationPercentage: 70
  # Uncomment to also scale on memory:
  # targetMemoryUtilizationPercentage: 80
```

Custom metrics scaling (e.g., on `synth_active_streams`) requires the Prometheus Adapter:

```yaml
# Custom metrics HPA example (requires prometheus-adapter)
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: datasynth-custom
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: datasynth
  minReplicas: 2
  maxReplicas: 20
  metrics:
    - type: Pods
      pods:
        metric:
          name: synth_active_streams
        target:
          type: AverageValue
          averageValue: "5"
```

## Pod Disruption Budget (PDB)

The PDB ensures at least one pod remains available during voluntary disruptions (node drains, cluster upgrades):

```yaml
podDisruptionBudget:
  enabled: true
  minAvailable: 1
```

For larger deployments, switch to `maxUnavailable`:

```yaml
podDisruptionBudget:
  enabled: true
  maxUnavailable: 1
```

## Ingress and TLS

### Nginx Ingress with cert-manager

```yaml
ingress:
  enabled: true
  className: nginx
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/proxy-read-timeout: "300"
    nginx.ingress.kubernetes.io/proxy-send-timeout: "300"
    nginx.ingress.kubernetes.io/proxy-body-size: "50m"
  hosts:
    - host: datasynth.example.com
      paths:
        - path: /
          pathType: Prefix
  tls:
    - secretName: datasynth-tls
      hosts:
        - datasynth.example.com
```

### WebSocket Support

For Nginx Ingress, WebSocket upgrade is handled automatically for paths starting with `/ws/`. If you use a path-based routing rule, ensure the annotation is set:

```yaml
nginx.ingress.kubernetes.io/proxy-http-version: "1.1"
nginx.ingress.kubernetes.io/configuration-snippet: |
  proxy_set_header Upgrade $http_upgrade;
  proxy_set_header Connection "upgrade";
```

### gRPC Ingress

gRPC requires a separate Ingress resource or an Ingress controller that supports gRPC (e.g., Nginx Ingress with `nginx.ingress.kubernetes.io/backend-protocol: "GRPC"`):

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: datasynth-grpc
  annotations:
    nginx.ingress.kubernetes.io/backend-protocol: "GRPC"
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
spec:
  ingressClassName: nginx
  tls:
    - secretName: datasynth-grpc-tls
      hosts:
        - grpc.datasynth.example.com
  rules:
    - host: grpc.datasynth.example.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: datasynth
                port:
                  name: grpc
```

## Manual Manifests (Without Helm)

If you prefer raw manifests, here is a minimal deployment:

```yaml
---
apiVersion: v1
kind: Namespace
metadata:
  name: datasynth
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: datasynth
  namespace: datasynth
spec:
  replicas: 2
  selector:
    matchLabels:
      app: datasynth
  template:
    metadata:
      labels:
        app: datasynth
    spec:
      securityContext:
        runAsNonRoot: true
        runAsUser: 1000
        fsGroup: 1000
      containers:
        - name: datasynth
          image: datasynth/datasynth-server:0.5.0
          ports:
            - containerPort: 3000
              name: http-rest
            - containerPort: 50051
              name: grpc
          env:
            - name: RUST_LOG
              value: "info"
          securityContext:
            allowPrivilegeEscalation: false
            readOnlyRootFilesystem: true
            capabilities:
              drop: ["ALL"]
          resources:
            requests:
              cpu: 500m
              memory: 512Mi
            limits:
              cpu: "2"
              memory: 2Gi
          livenessProbe:
            httpGet:
              path: /live
              port: http-rest
            initialDelaySeconds: 15
            periodSeconds: 20
          readinessProbe:
            httpGet:
              path: /ready
              port: http-rest
            initialDelaySeconds: 5
            periodSeconds: 10
---
apiVersion: v1
kind: Service
metadata:
  name: datasynth
  namespace: datasynth
spec:
  type: ClusterIP
  ports:
    - port: 3000
      targetPort: http-rest
      name: http-rest
    - port: 50051
      targetPort: grpc
      name: grpc
  selector:
    app: datasynth
```

## Prometheus ServiceMonitor

If you use the Prometheus Operator, enable the ServiceMonitor:

```yaml
serviceMonitor:
  enabled: true
  interval: 30s
  scrapeTimeout: 10s
  path: /metrics
  labels:
    release: prometheus  # Must match your Prometheus Operator selector
```

## Rolling Update Strategy

The chart uses a zero-downtime rolling update strategy:

```yaml
strategy:
  type: RollingUpdate
  rollingUpdate:
    maxUnavailable: 0
    maxSurge: 1
```

Combined with the PDB and readiness probes, this ensures that:

1. A new pod starts and becomes ready before an old pod is terminated.
2. At least `minAvailable` pods are always serving traffic.
3. Config and secret changes trigger a rolling restart via checksum annotations.

## Topology Spread

For multi-zone clusters, use topology spread constraints to distribute pods evenly:

```yaml
topologySpreadConstraints:
  - maxSkew: 1
    topologyKey: topology.kubernetes.io/zone
    whenUnsatisfiable: DoNotSchedule
    labelSelector:
      matchLabels:
        app.kubernetes.io/name: datasynth
```
