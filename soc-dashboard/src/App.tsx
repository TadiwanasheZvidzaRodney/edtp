import React, { useState, useEffect } from 'react';

// Inline lucide icons for premium aesthetics
const ShieldIcon = () => <svg className="w-6 h-6 text-primary" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" /></svg>;
const ActivityIcon = () => <svg className="w-5 h-5 text-textMuted" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" /></svg>;

function App() {
  const [alerts, setAlerts] = useState<any[]>([]);

  useEffect(() => {
    // Simulated websocket stream of real-time alerts from the gRPC Gateway
    const timer = setInterval(() => {
      setAlerts(prev => [{
        id: Math.random().toString(36).substr(2, 9),
        endpoint: 'EP-WIN-' + Math.floor(Math.random() * 9000 + 1000),
        type: Math.random() > 0.5 ? 'Suspicious PowerShell Spawn' : 'Network Discovery Anomaly',
        time: new Date().toLocaleTimeString(),
        severity: Math.random() > 0.8 ? 'critical' : 'warning'
      }, ...prev].slice(0, 8)); // Keep top 8
    }, 2500);
    return () => clearInterval(timer);
  }, []);

  return (
    <div className="flex h-screen bg-bg overflow-hidden p-6 gap-6">
      {/* Sidebar */}
      <aside className="w-64 glass-panel flex flex-col p-4">
        <div className="flex items-center gap-3 mb-8">
          <ShieldIcon />
          <h1 className="text-xl font-bold tracking-wider text-textMain">EDTP <span className="text-primary text-sm uppercase font-black">Core</span></h1>
        </div>
        <nav className="flex-1 space-y-2">
          <a href="#" className="flex items-center gap-3 px-4 py-2 rounded-lg bg-primary/10 text-primary border border-primary/20 transition-all shadow-glow">
            <ActivityIcon />
            <span className="font-medium">Live Telemetry</span>
          </a>
          <a href="#" className="flex items-center gap-3 px-4 py-2 rounded-lg text-textMuted hover:bg-surface hover:text-textMain transition-colors">
            <span className="font-medium">Agentic AI</span>
          </a>
        </nav>
      </aside>

      {/* Main Content */}
      <main className="flex-1 flex flex-col gap-6">
        
        {/* Top Stats */}
        <div className="grid grid-cols-3 gap-6">
          <div className="glass-panel p-6 flex flex-col">
            <span className="text-xs font-medium text-textMuted uppercase tracking-wider mb-2">Fleet Status</span>
            <div className="flex items-end gap-3">
              <span className="text-4xl font-bold text-secure">2,410</span>
              <span className="text-sm text-textMuted mb-1 font-medium">Agents Online</span>
            </div>
          </div>
          <div className="glass-panel p-6 flex flex-col">
            <span className="text-xs font-medium text-textMuted uppercase tracking-wider mb-2">Active AI Investigations</span>
            <div className="flex items-end gap-3">
              <span className="text-4xl font-bold text-primary animate-pulse-slow">3</span>
              <span className="text-sm text-textMuted mb-1 font-medium">LangGraph Orchs</span>
            </div>
          </div>
          <div className="glass-panel p-6 flex flex-col border border-critical/30 shadow-glow-critical relative overflow-hidden">
            <div className="absolute top-0 right-0 w-32 h-32 bg-critical/10 rounded-full blur-3xl"></div>
            <span className="text-xs font-medium text-critical uppercase tracking-wider mb-2 z-10">Critical Alerts</span>
            <div className="flex items-end gap-3 z-10">
              <span className="text-4xl font-bold text-critical">1</span>
              <span className="text-sm text-textMuted mb-1 font-medium">Pending Isolation</span>
            </div>
          </div>
        </div>

        {/* Telemetry Grid */}
        <div className="glass-panel flex-1 p-6 flex flex-col">
          <div className="flex items-center justify-between mb-6 border-b border-borderSubtle pb-4">
            <h2 className="text-lg font-semibold tracking-wide">Live Threat Stream</h2>
            <div className="flex items-center gap-2">
              <span className="relative flex h-3 w-3">
                <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-primary opacity-75"></span>
                <span className="relative inline-flex rounded-full h-3 w-3 bg-primary"></span>
              </span>
              <span className="text-xs text-primary font-medium tracking-wide uppercase">Connected to Gateway</span>
            </div>
          </div>
          
          <div className="overflow-y-auto flex-1 pr-2">
            <table className="w-full text-left">
              <thead>
                <tr className="text-textMuted text-xs uppercase tracking-wider">
                  <th className="pb-4 font-medium">Timestamp</th>
                  <th className="pb-4 font-medium">Endpoint</th>
                  <th className="pb-4 font-medium">Detection Rule / ML Anomaly</th>
                  <th className="pb-4 font-medium">Severity</th>
                  <th className="pb-4 font-medium text-right">Action</th>
                </tr>
              </thead>
              <tbody>
                {alerts.map((alert) => (
                  <tr key={alert.id} className="border-b border-borderSubtle/30 hover:bg-surface transition-colors group cursor-pointer">
                    <td className="py-4 text-textMuted text-sm font-mono">{alert.time}</td>
                    <td className="py-4 font-mono text-sm font-medium text-textMain">{alert.endpoint}</td>
                    <td className="py-4 font-medium text-sm text-textMain">{alert.type}</td>
                    <td className="py-4">
                      <span className={`px-2 py-1 rounded text-[10px] font-black uppercase tracking-widest ${
                        alert.severity === 'critical' 
                          ? 'bg-critical/20 text-critical border border-critical/30 shadow-[0_0_10px_rgba(255,42,95,0.2)]' 
                          : 'bg-warning/20 text-warning border border-warning/30'
                      }`}>
                        {alert.severity}
                      </span>
                    </td>
                    <td className="py-4 text-right">
                      {alert.severity === 'critical' ? (
                        <button className="px-4 py-1.5 bg-critical/10 text-critical text-xs font-bold uppercase tracking-wide rounded border border-critical/30 hover:bg-critical hover:text-white transition-all shadow-[0_0_10px_rgba(255,42,95,0.2)] hover:shadow-[0_0_15px_rgba(255,42,95,0.5)]">
                          Isolate
                        </button>
                      ) : (
                        <button className="px-4 py-1.5 bg-borderSubtle/30 text-textMuted text-xs font-bold uppercase tracking-wide rounded border border-borderSubtle/50 hover:bg-borderSubtle transition-all opacity-0 group-hover:opacity-100">
                          Investigate
                        </button>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      </main>
    </div>
  );
}

export default App;
