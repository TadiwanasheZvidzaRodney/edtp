import asyncio
from typing import TypedDict, List, Literal
from langgraph.graph import StateGraph, END

# Define the orchestration state
class AgentState(TypedDict):
    alert_id: str
    endpoint: str
    raw_logs: List[str]
    analysis: str
    action: str

def ingest_alert(state: AgentState):
    print(f"\n[IngestNode] Received critical alert {state['alert_id']} from {state['endpoint']}")
    return {"raw_logs": ["Detected hidden PowerShell spawn from winword.exe via Agent Rule Engine"]}

def query_history(state: AgentState):
    print("[QueryNode] Querying ClickHouse analytical store for 30-day baseline context...")
    logs = state.get("raw_logs", [])
    logs.append("Context: User on EP-WIN-1042 has never spawned PowerShell before in the last 30 days.")
    return {"raw_logs": logs}

def analyze_threat(state: AgentState):
    print("[LLMNode] Correlating telemetry via LLM...")
    # Simulated deterministic output based on context
    analysis = "High-confidence malicious macro execution (T1059.001). Immediate isolation required to prevent lateral movement."
    print(f"   -> Result: {analysis}")
    return {"analysis": analysis, "action": "isolate"}

def isolate_endpoint(state: AgentState):
    print(f"![ResponseNode] Dispatching ISOLATE command to {state['endpoint']} via gRPC gateway...")
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
    print("Starting LangGraph Orchestration Engine...")
    print("Listening for high-severity alerts from NATS stream...\n")
    
    # Simulate receiving an alert
    state = {"alert_id": "ALT-CRIT-990", "endpoint": "EP-WIN-1042", "raw_logs": [], "analysis": "", "action": ""}
    
    async for output in app.astream(state):
        pass # Nodes handle printing

if __name__ == "__main__":
    asyncio.run(run_ai())
