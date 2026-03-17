import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";

// Placeholder — RewardOverlay and full UI come after log_watcher is verified
function App() {
  const [status, setStatus] = useState("Watching EE.log...");
  const [lastEvent, setLastEvent] = useState<string | null>(null);

  useEffect(() => {
    // Listen for reward screen detection events from Rust backend
    const unlisten = listen<string>("reward-screen-detected", (event) => {
      setLastEvent(event.payload);
      setStatus("Reward screen detected!");
    });

    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  return (
    <div className="min-h-screen bg-wf-bg text-wf-text flex flex-col items-center justify-center p-8">
      <h1 className="text-2xl font-bold text-wf-accent mb-4">TennoHelios</h1>
      <p className="text-wf-text/70 mb-2">{status}</p>
      {lastEvent && (
        <p className="text-xs text-wf-text/50 mt-4 font-mono">{lastEvent}</p>
      )}
    </div>
  );
}

export default App;
