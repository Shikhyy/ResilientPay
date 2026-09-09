
# Infrastructure

Infrastructure supports local development, continuous integration, and future deployment workflows.

## Local development

The first milestone should provide a reproducible local environment using documented commands and controlled test data.

Expected local services may include:
- PostgreSQL
- backend service
- simulator
- optional observability stack

## CI

CI should validate changed areas and execute cross-component checks when interfaces are affected.

## Deployment boundary

Research infrastructure must be clearly separated from any future production configuration. Prototype credentials, simulated money, and local secrets must never be treated as production values.

## Future production

Production deployment will require additional architecture for:
- high availability
- secret management
- key management
- data protection
- disaster recovery
- monitoring
- incident response
- regulatory controls

The existence of deployment scripts must not be interpreted as production readiness.
