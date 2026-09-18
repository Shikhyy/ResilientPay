
# Infrastructure

Infrastructure supports local development, continuous integration, and future deployment workflows.

## Local Development with Docker Compose

A turnkey multi-service setup is provided using Docker Compose:
- **PostgreSQL 16**: Relational storage engine with automated migrations.
- **ResilientPay Reconciliation Backend**: Go microservice compiled via multi-stage alpine build.

### Running Services
From the repository root:
```bash
docker compose -f infrastructure/docker-compose.yml up --build -d
```

### Verifying Service Health
```bash
# Check service health and logs
docker compose -f infrastructure/docker-compose.yml ps
docker compose -f infrastructure/docker-compose.yml logs -f backend

# Test backend health endpoint
curl -i http://localhost:8080/health
```

### Tearing Down
```bash
docker compose -f infrastructure/docker-compose.yml down -v
```

## Continuous Integration (CI)

CI workflows reside in `.github/workflows/ci.yml` and execute automated gates on push and pull requests:
- **Rust Core SDK**: Compiles `sdk/core`, executes unit and integration tests, and validates frozen protocol vectors.
- **Go Backend**: Runs race detection (`go test -race -v ./...`) across all packages.
- **Python Simulator**: Executes test suite and benchmark invariants.

## Deployment Boundary

Research infrastructure must be clearly separated from any future production configuration. Prototype credentials, simulated money, and local secrets must never be treated as production values.

## Future Production

Production deployment will require additional architecture for:
- high availability & multi-region database failover
- HSM-backed secret and key management
- rate limiting & DDoS mitigation
- disaster recovery & automated backup replication
- audit logging & SIEM observability
- strict regulatory compliance (RBI / NPCI certifications)

The existence of deployment scripts must not be interpreted as live UPI production readiness.
