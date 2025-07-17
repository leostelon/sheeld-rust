
# 🛡️ Sheeld VPN - Rust Implementation

Sheeld VPN is a decentralized proxy solution written in **Rust**, designed for privacy, performance, and extensibility. It leverages peer-to-peer networking and SOCKS5 tunneling to build a privacy-preserving proxy system.

---

## 🔧 Features

- ✅ **Peer discovery & messaging** via [`libp2p`] using **Gossipsub**
- ✅ **SOCKS5 proxy server** using [`fast_socks5`]
- ⚙️ Built for **low latency** and **modular extension**
- 🌍 Peer-based relay routing (decentralized VPN nodes)

---

## 📦 Dependencies

- [`libp2p`](https://github.com/libp2p/rust-libp2p) – for peer discovery and message propagation using Gossipsub
- [`fast_socks5`](https://crates.io/crates/fast_socks5) – for lightweight and efficient SOCKS5 proxy server implementation

---

## 📌 Roadmap & Tasks

| Task                          | Status   |
|-------------------------------|----------|
| ✅ Use `libp2p` with Gossipsub for peer discovery and messaging | **Completed** |
| ✅ Integrate `fast_socks5` for SOCKS5 proxy server             | **Completed*** |
| 🔄 Implement proxy chaining for multi-hop routing              | *Pending* |
| 🔄 Add end-to-end encryption for traffic                       | *Pending* |
| 🔄 Monetization mechanism (e.g., token-based usage)            | *Pending* |

<sub>*Basic implementation of SOCKS5</sub>

---
## 🤝 Contribution

PRs and suggestions are welcome! Please open an issue to discuss before submitting major changes.