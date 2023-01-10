# comet

Kubernetes CNI Plugin written with Rust. The main goal is to implement Comet agent using eBPF-based dataplane and WASM container.

## Crates

### netlink

Provides a simple netlink library for Rust.

## Comet components

### Comet plugin

CNI spec implementation

### Comet agent

Deploy to hosts to control network traffic and install plugin

### IPAM

IP address management via Kubernetes resource
