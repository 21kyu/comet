# comet

Kubernetes CNI Plugin written with Rust. The main goal is to implement Comet agent using eBPF-based dataplane and WASM container.

## Crates

### netlink

Provides a simple netlink library for Rust.

Once this crate is more or less implemented, it will be moved to a separate repository (https://github.com/21kyu/lnwasi).
The end goal is to make it a library for web assembly.

## Comet components

### Comet plugin

CNI spec implementation

### Comet agent

Deploy to hosts to control network traffic and install plugin

### IPAM

IP address management via Kubernetes resource
