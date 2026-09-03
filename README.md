# Edge-AI Endpoint Detection & Telemetry Platform (EDTP)

EDTP is a next-generation Endpoint Detection and Response (EDR) platform designed to outperform legacy market incumbents by shifting threat detection directly to the edge. By running ultra-lightweight Rust native binaries, capturing telemetry at the kernel level, and utilizing on-device Machine Learning (Isolation Forests/ONNX) to filter noise, EDTP guarantees **sub-1% CPU usage** and reduces network transmission costs by up to **98%**.

When critical threats bypass local heuristics, high-fidelity alerts are streamed via gRPC into a central ClickHouse analytical datastore, where a LangGraph-powered **Agentic AI Orchestrator** autonomously investigates the event and dispatches real-time containment commands (e.g., firewall isolation).

## 🚀 Core Competitive Differentiators

1. **Sub-1% CPU & 30MB RAM Footprint**: Built completely in Rust using low-level kernel interfaces (eBPF on Linux / ETW on Windows) rather than heavy user-space polling agents.
2. **Local Inference (Zero Cloud Latency)**: Runs ML isolation models directly on the endpoint to detect behavioral anomalies offline within milliseconds.
3. **98% Network & Storage Cost Reduction**: Leverages ClickHouse for 10x column-oriented compression and only forwards enriched JSON alerts, eliminating the need to ship raw logs to expensive cloud SIEMs.
4. **Federated "Teleport" Search**: Raw verbose logs are stored locally in a 7-day rolling SQLite buffer on the machine. Analysts can query the agent directly for raw forensics on-demand.

---

## 🏗️ System Architecture

```mermaid
graph TD
    A[Local System Events (Kernel/ETW)] -->|Capture| B(Rust Agent Core)
    B -->|Persist| C[(Local Ring Buffer: 7 days)]
    B -->|Analyze| D[Edge AI & Rule Engine]
    D -.->|Drop Noise| X(Discarded 95%)
    D -->|High-Value Alerts| E[gRPC Ingest Gateway]
    E -->|Publish| F((NATS JetStream))
    F -->|Batch Insert| G[(ClickHouse DB)]
    G -->|Query| H[SOC Dashboard]
    H -.->|On-Demand Query| B
    F -->|Consume| I[Agentic AI Orchestrator]
    I -->|Dispatch Isolation| E
    E -->|gRPC Stream| B
```

---

## 📂 Monorepo Structure

This project is organized as a unified monorepo:

*   **`agent-core/`**: The lightweight edge agent written in Rust. Features a zero-mutex `crossbeam` channel pipeline, SQLite local ring buffer, ETW hooks, and a local YAML Rule Engine.
*   **`ingest-gateway/`**: The central ingestion pipeline written in Rust. Exposes a `tonic` gRPC server to thousands of endpoints, securely buffers alerts into `NATS JetStream`, and asynchronously performs columnar-optimized bulk inserts into ClickHouse.
*   **`agentic-ai/`**: A deterministic orchestration engine written in Python utilizing `LangGraph`. Ingests critical alerts, queries ClickHouse for historical baseline context, uses an LLM to build attack trees, and autonomously fires isolation actions back to the endpoints.
*   **`soc-dashboard/`**: A highly polished, real-time SOC web dashboard built with React, TypeScript, Vite, and Tailwind CSS. strictly follows a premium dark-mode `60/30/10` color matrix.
*   **`protos/`**: Shared Protobuf schemas (e.g., `telemetry.proto`) enforcing strict type contracts between the edge agent and central gateway using an OCSF-inspired standard.
*   **`infrastructure/`**: Local `docker-compose.yml` defining the NATS/JetStream broker and ClickHouse analytical database.

---

## 💻 Tech Stack Matrix

| Layer | Recommended Technology | Why It Outperforms Alternatives |
| :--- | :--- | :--- |
| **Endpoint Agent** | **Rust** + **eBPF** (Linux) / **ETW** (Windows) | Memory-safe, zero runtime GC pauses, native kernel execution. |
| **Edge Filtering** | **smartcore** (Isolation Forest) / **YAML Rules** | Drops noise on the device. Predictable <15MB memory footprint. |
| **Transport** | **gRPC** over HTTP/2 with mTLS | Binary serialization with protocol buffers; lighter and faster than HTTPS REST. |
| **Message Queue** | **NATS JetStream** | Ultra-fast queueing with significantly lower memory footprint than JVM Kafka. |
| **Database** | **ClickHouse** | Ultra-fast column-oriented database. Queries billions of rows in milliseconds using `ZSTD`. |
| **Agentic AI** | **Python** + **LangGraph** + **Qwen/Llama** | Deterministic workflows for automated threat isolation and historical telemetry querying. |
| **Dashboard UI** | **React** + **Vite** + **Tailwind CSS** | Premium glassmorphism UI with real-time WebSocket state management. |

---

## 🏁 Getting Started (Local Development)

### 1. Start the Infrastructure
Spin up NATS and ClickHouse locally using Docker:
```bash
cd infrastructure
docker-compose up -d
```

### 2. Run the Ingestion Gateway
Start the gRPC server and ClickHouse insertion worker:
```bash
cd ingest-gateway
cargo run
```

### 3. Run the Rust Edge Agent
Run the agent (Note: ETW hooks require running the terminal as Administrator):
```bash
cd agent-core
cargo run
```

### 4. Launch the SOC Dashboard
Start the Vite development server:
```bash
cd soc-dashboard
npm install
npm run dev
```

### 5. Start the Agentic AI
Run the LangGraph orchestrator to automatically monitor and isolate threats:
```bash
cd agentic-ai
python -m venv venv
venv\Scripts\activate
pip install -r requirements.txt
python main.py
```
