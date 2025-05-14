# L4GeoLB
Rust based layer 4 proximity load balancer primarily for TCP connections.

Features:
- Programatic retrieval of proxied frontend -> backend connection informtion from PostgreSQL on initialization and periodically.
- Round robin & distance based connection balancing.
- Limiting connections.
- 10s periodic health checks to determine backend should still be in serving pool.
- REST API to add additional L4 proxies.