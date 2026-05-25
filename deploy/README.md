# Deployment (optional, research prototype)

Docker Compose runs the `astra-ops` binary plus Prometheus and Grafana. This is for **local observation**, not a production cluster.

## Prerequisites

- Docker Engine with Compose v2
- Ports `8081`â€“`8083`, `9090`, `3000` available

## Start

```bash
docker compose -f deploy/docker-compose.yml up -d --build
```

Each node container:

- Sets `ASTRA_JOURNAL_DIR=/data/journal`
- Exposes Prometheus text metrics on `ASTRA_HTTP_PORT` (plain HTTP, any path)

## Metrics actually exported

| Metric | Type |
|--------|------|
| `astra_websocket_reconnects` | counter |
| `astra_network_latency_ms` | gauge |

Grafana dashboards under `deploy/grafana/dashboards/` reference **only** these series.

## Prometheus

Scrape targets are `node1:8081`, `node2:8082`, `node3:8083` per `deploy/prometheus/prometheus.yml`.

## Stop

```bash
docker compose -f deploy/docker-compose.yml down
```