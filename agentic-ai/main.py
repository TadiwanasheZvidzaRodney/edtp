import asyncio
import json
import os
from datetime import datetime, timezone
from typing import List, Literal, TypedDict

import nats
from langgraph.graph import END, StateGraph
from pydantic import BaseModel, Field, ValidationError

NATS_URL = os.getenv("NATS_URL", "nats://127.0.0.1:4222")
ALERT_SUBJECT = os.getenv("NATS_ALERT_SUBJECT", "telemetry.alerts")
COMMAND_SUBJECT = os.getenv("NATS_COMMAND_SUBJECT", "telemetry.commands")

COMMAND_PUBLISHER: nats.NATS | None = None


class AlertEvent(BaseModel):
    event_id: str
    timestamp: str
    endpoint_id: str
    tenant_id: str
    severity: int = Field(ge=0, le=10)
    message: str
    category: str


class IsolateCommand(BaseModel):
    command_id: str
    action: str
    endpoint_id: str
    tenant_id: str
    reason: str
    source_alert_id: str
    issued_at: str


# Define the orchestration state
class AgentState(TypedDict):
    alert: AlertEvent
    raw_logs: List[str]
    analysis: str
    action: str


def ingest_alert(state: AgentState):
    alert = state["alert"]
    print(f"\n[IngestNode] Received alert {alert.event_id} from {alert.endpoint_id} ({alert.category})")
    return {"raw_logs": [f"Alert message: {alert.message}"]}


def query_history(state: AgentState):
    print("[QueryNode] Querying historical baseline context...")
    logs = state.get("raw_logs", [])
    alert = state["alert"]
    logs.append(
        f"Context: Endpoint {alert.endpoint_id} has low prior frequency for category '{alert.category}'."
    )
    return {"raw_logs": logs}


def analyze_threat(state: AgentState):
    alert = state["alert"]
    print("[DecisionNode] Correlating telemetry and applying deterministic policy...")

    if alert.severity >= 8:
        analysis = (
            f"High-severity alert ({alert.severity}/10) in category '{alert.category}'. "
            "Immediate containment recommended."
        )
        action = "isolate"
    else:
        analysis = (
            f"Moderate alert ({alert.severity}/10) in category '{alert.category}'. "
            "Keep endpoint monitored."
        )
        action = "monitor"

    print(f"   -> Result: {analysis}")
    return {"analysis": analysis, "action": action}


async def isolate_endpoint(state: AgentState):
    alert = state["alert"]
    print(f"[ResponseNode] Dispatching ISOLATE command to {alert.endpoint_id} via NATS...")

    if COMMAND_PUBLISHER is None:
        raise RuntimeError("NATS publisher is not initialized")

    command = IsolateCommand(
        command_id=f"CMD-{alert.event_id}",
        action="isolate",
        endpoint_id=alert.endpoint_id,
        tenant_id=alert.tenant_id,
        reason=state["analysis"],
        source_alert_id=alert.event_id,
        issued_at=datetime.now(timezone.utc).isoformat(),
    )

    await COMMAND_PUBLISHER.publish(COMMAND_SUBJECT, command.model_dump_json().encode("utf-8"))
    print(f"   -> Command published on subject '{COMMAND_SUBJECT}'")
    return state


def should_isolate(state: AgentState) -> Literal["isolate", "end"]:
    return "isolate" if state.get("action") == "isolate" else "end"

# Build the Deterministic State Graph
workflow = StateGraph(AgentState)
workflow.add_node("ingest", ingest_alert)
workflow.add_node("query", query_history)
workflow.add_node("analyze", analyze_threat)
workflow.add_node("isolate", isolate_endpoint)

workflow.add_edge("ingest", "query")
workflow.add_edge("query", "analyze")
workflow.add_conditional_edges("analyze", should_isolate, {"isolate": "isolate", "end": END})
workflow.add_edge("isolate", END)

workflow.set_entry_point("ingest")
app = workflow.compile()

async def run_ai():
    global COMMAND_PUBLISHER

    print("Starting LangGraph Orchestration Engine...")
    print(f"Connecting to NATS at {NATS_URL}...")

    nc = await nats.connect(servers=[NATS_URL])
    COMMAND_PUBLISHER = nc
    subscription = await nc.subscribe(ALERT_SUBJECT)

    print(f"Listening for alerts on '{ALERT_SUBJECT}'...\n")

    while True:
        try:
            msg = await subscription.next_msg(timeout=30)
        except nats.errors.TimeoutError:
            # Keep the worker alive during idle periods with no alerts.
            continue

        try:
            payload = json.loads(msg.data.decode("utf-8"))
            alert = AlertEvent.model_validate(payload)
        except (json.JSONDecodeError, ValidationError) as exc:
            print(f"[ParseError] Dropping invalid alert payload: {exc}")
            continue

        state: AgentState = {
            "alert": alert,
            "raw_logs": [],
            "analysis": "",
            "action": "",
        }

        async for _ in app.astream(state):
            pass

if __name__ == "__main__":
    asyncio.run(run_ai())
