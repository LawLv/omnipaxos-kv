# Omnipaxos-KV: Distributed SQL Database with Strong Consistency

A distributed SQL database built on top of [OmniPaxos](https://github.com/haraldng/omnipaxos), ensuring **strong consistency**, **high availability**, and support for **multi-level read consistency**.

> 🛠️ Each server maintains its own PostgreSQL database instance. OmniPaxos ensures consistency across these replicas by replicating all queries via a distributed changelog.

## Features

-  **Strongly consistent replication** with [OmniPaxos](https://github.com/haraldng/omnipaxos)
-  **Local SQL database** on each server (PostgreSQL), dynamically initialized
-  **Write queries** replicated across all nodes and executed in order
-  **Read queries** with 3 consistency levels:
  - `local` – read directly from the current node
  - `leader` – routed to and processed by the current leader
  - `linearizable` – guarantees visibility of all prior writes
-  **Tested read/write correctness** under all consistency levels
-  **Dockerized setup** for easy local testing or cloud deployment
-  **GCP deployment support**

---

## Architecture Overview

```text
+-------------+       +------------+       +------------+
|   Client    | <---> |  Server 1  | <---> |  Server 2  |
|             |       |  (Leader)  | <---> |  Server 3  |
+-------------+       +------------+       +------------+

Each server:
- Hosts a local PostgreSQL database
- Uses OmniPaxos for consistent query replication
- Applies and responds to SQL queries
```

---

## How to Run

### Prerequisites

- Docker & Docker Compose
- Rust & Cargo (for development)
- PostgreSQL driver (`sqlx`)

### Build & Run

#### 1. Start Local Cluster

```bash
./run-local-cluster.sh
```

- Starts 3 server instances using configs:
  - `server-1-config.toml`
  - `server-2-config.toml`
  - `server-3-config.toml`
- Each server:
  - Connects to its local PostgreSQL
  - Initializes table with `CREATE TABLE IF NOT EXISTS`
  - Participates in OmniPaxos log replication

#### 2. Start Clients

```bash
./run-local-client.sh
```

- Clients use configs like `client-1-config.toml` / `client-2-config.toml`
- On startup:
  - Establish TCP connection to a server
  - Send registration (`ClientRegister`)
  - Issue SQL queries (`INSERT ... ON CONFLICT DO UPDATE`, `SELECT`)
  - Specify consistency level per request: `local`, `leader`, `linearizable`

---

## Query Consistency Levels

| Consistency Level | Description                     | Behavior                                                      |
|-------------------|----------------------------------|---------------------------------------------------------------|
| `local`           | Fast, no coordination            | Query directly on the current server                          |
| `leader`          | Coordinated via Paxos leader     | Forward to leader, leader executes and responds               |
| `linearizable`    | Full consistency                 | Wait until all prior writes applied, then execute query       |

---

## Implementation Highlights

### Write Path

- All SQL write queries are wrapped in OmniPaxos `KVCommand`
- Once agreed upon, each server applies the command to its own PostgreSQL instance

### Read Path

- `local`: direct query on the current server
- `leader`: server checks if it's the leader; if not, forwards to the leader
- `linearizable`: server waits until all previous commands are applied (via log index tracking), then performs the query

---

## Testing

- ✔Test cases verify that:
  - Writes are consistently reflected across all nodes
  - Reads under all three consistency levels return expected results
- Known issue: some duplicate message logs during client-server interactions (under investigation)

---

## Development Timeline

### 3/7
- Implemented consistency-aware `SELECT` logic in `update_database_and_respond` (server)
- `leader` read level added with forwarding if needed
- Some duplicate log outputs observed during test runs

### 3/12
- Fixed `leader` read response path:
  - Leader now returns result to the requesting server, which forwards it to the client

### 3/14
- Added `linearizable` read support:
  - Server waits until local `current_decided_idx` >= `omnipaxos.get_decided_idx()` before querying

### 3/15
- Deployed project on Google Cloud Platform:
  - Created VM with Docker & Rust
  - Uploaded code and ran cluster/client containers

---

## 📁 Project Structure

```
.
├── server/
│   ├── src/
│   ├── config/
│   └── run-local-cluster.sh
├── client/
│   ├── src/
│   ├── config/
│   └── run-local-client.sh
├── omnipaxos-kv/
│   └── src/
├── common/
│   └── kvcommand.rs
├── Cargo.toml
```

---

## GCP Deployment Steps

1. Create VM on Google Cloud with Docker installed
2. Set up SSH access
3. Upload your `omnipaxos-kv` project
4. Run cluster via `run-local-cluster.sh`
5. Start clients and test remote behavior

---

## Repository

GitHub: [https://github.com/LawLv/omnipaxos-kv](https://github.com/LawLv/omnipaxos-kv)

---

## Future Improvements

- **Sharding for Scalability**  
  The current design replicates the entire database in a single OmniPaxos cluster. To scale out, the system can be extended to support sharding:
  - Use multiple OmniPaxos clusters for different key ranges (e.g., keys 1–100, 101–200, etc.)
  - Optionally support cross-shard transactions (2PC) or automatic rebalancing

- **Improved GCP Deployment**  
  - Automate VM setup using Terraform or shell scripts  
  - Use Docker Compose or Kubernetes on GCP VM/cluster  
  - Include basic firewall and network configuration instructions  
  - Add monitoring/logging via `journalctl`, `docker logs`, or Prometheus

- **Optional: UI Support**  
  A simple web interface could help users submit SQL queries and inspect system status (not implemented yet)


