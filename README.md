# socketup

A tiny static readiness probe for distroless containers.

## Why

Distroless images usually do not include shell utilities (`sh`, `curl`, `nc`), and Kubernetes TCP probes are too limited for richer health checks.

`socketup` provides a minimal static binary you can inject and execute directly.

## Security stance

- **Numeric-only targets**: checks only accept numeric `ip:port` (or `8080` / `:8080` shorthand to `127.0.0.1:8080`), so hostnames are rejected at config load.
- **Read-only allowlist config**: probe targets are defined in config and selected by check name; there is no runtime `--target` override.

## CLI

```bash
socketup --config /etc/socketup/socketup.yaml check <name>
socketup --config /etc/socketup/socketup.yaml validate
socketup install /work/socketup
socketup version
```

Global flags:

- `--config <path>` (env: `SOCKETUP_CONFIG`, default `/etc/socketup/socketup.yaml`)
- `-q`, `--quiet`
- `-v`, `--verbose`

## Config

```yaml
defaults:
  timeout_ms: 1000
  mode: all

checks:
  admin:
    type: tcp
    targets:
      - "127.0.0.1:9901"

  ingress:
    type: tcp
    timeout_ms: 2000
    mode: all
    targets:
      - "127.0.0.1:8080"
      - "127.0.0.1:9901"

  any-listener:
    type: tcp
    mode: any
    targets:
      - "127.0.0.1:8080"
      - "127.0.0.1:9901"
```

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Check succeeded |
| 1 | Check failed |
| 2 | Bad CLI usage |
| 3 | Config file not found/readable |
| 4 | Config parse error |
| 5 | Config validation error |
| 6 | Named check does not exist |
| 7 | Timeout detected |
| 8 | Internal error |

## Quickstart

```dockerfile
FROM rust:1-bookworm AS build
WORKDIR /src
COPY . .
RUN RUSTFLAGS='-C target-feature=+crt-static' cargo build --release --target x86_64-unknown-linux-gnu --bin socketup

FROM scratch AS socketup-probe
COPY --from=build /src/target/x86_64-unknown-linux-gnu/release/socketup /socketup
COPY docker/socketup.yaml /etc/socketup/socketup.yaml
ENTRYPOINT ["/socketup"]
```

```yaml
initContainers:
  - name: socketup-init
    image: ghcr.io/phlax/socketup:latest
    command: ["/socketup", "install", "/work/socketup"]
    volumeMounts:
      - name: probe-bin
        mountPath: /work

containers:
  - name: app
    readinessProbe:
      exec:
        command: ["/work/socketup", "check", "admin"]
    volumeMounts:
      - name: probe-bin
        mountPath: /work
```

## Roadmap

- HTTP checks
- TLS checks
- Unix socket checks
- Sequence checks
- Parallel probing
