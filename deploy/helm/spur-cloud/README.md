# spur-cloud Helm chart

Deploys the Spur Cloud control plane (API + frontend), optional in-cluster
Postgres, RBAC, and ingress. Does **not** deploy `spurctld` or the
`spur-k8s` operator — those live in [ROCm/spur](https://github.com/ROCm/spur).

## TL;DR

```bash
# 1. Generate a JWT signing key + DB password
JWT=$(openssl rand -hex 32)
DBPW=$(openssl rand -hex 16)

# 2. Install
helm install spur-cloud ./deploy/helm/spur-cloud \
  --namespace spur-cloud --create-namespace \
  --set secrets.jwtSecret="$JWT" \
  --set secrets.dbPassword="$DBPW" \
  --set ingress.host=gpu.example.com \
  --set config.publicUrl=https://gpu.example.com
```

## What gets installed

| Resource | Default | Toggle |
|----------|---------|--------|
| `Deployment` spur-cloud-api (2 replicas) | on | `api.enabled` |
| `Deployment` spur-cloud-frontend (2 replicas) | on | `frontend.enabled` |
| `Ingress` (host + `/` → frontend, `/api` → api) | on | `ingress.enabled` |
| `Secret` (spur-cloud.toml + db-password) | on | `secrets.create` |
| `ServiceAccount` + `ClusterRole`/`Binding` | on | `serviceAccount.create`, `rbac.create` |
| `StatefulSet` postgres (1 replica) | on | `postgres.enabled` |
| `Namespace` for session pods | on | `createSessionNamespace` |

## Required secrets

The chart fails to render unless these are set (or you provide an
`existingSecret` and turn `secrets.create=false`):

- `secrets.jwtSecret` — JWT signing key (`openssl rand -hex 32`)
- `secrets.dbPassword` — when `postgres.enabled=true`
- `secrets.githubClientSecret` — when `config.auth.github.enabled=true`
- `secrets.oktaClientSecret` — when `config.auth.okta.enabled=true`

## External Postgres

```yaml
postgres:
  enabled: false
database:
  url: "postgresql://user:pass@rds.example.com:5432/spur_cloud"
```

When using ExternalSecrets / sealed-secrets:

```yaml
secrets:
  create: false
  existingSecret: my-existing-secret
```

The existing secret must contain key `spur-cloud.toml` (full rendered
config) and, if using in-cluster Postgres, key `db-password`.

## Image references

Defaults point at `ghcr.io/rocm/spur-cloud-{api,frontend}`. Override:

```yaml
api:
  image:
    repository: my-registry.example.com/spur-cloud-api
    tag: v0.2.0
frontend:
  image:
    repository: my-registry.example.com/spur-cloud-frontend
    tag: v0.2.0
image:
  pullSecrets:
    - name: my-registry-creds
```

## Verify

```bash
helm lint ./deploy/helm/spur-cloud \
  --set secrets.jwtSecret=test --set secrets.dbPassword=test
helm template spur-cloud ./deploy/helm/spur-cloud \
  --set secrets.jwtSecret=test --set secrets.dbPassword=test
```

## Limitations / TODO

- No HPA, NetworkPolicy, PodDisruptionBudget yet.
- Postgres is single-replica with no backup. Production should use managed
  Postgres or a real operator (CNPG, Zalando).
- No HA story for spurctld here — see the spur chart.
