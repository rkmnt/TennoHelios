import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { RewardOverlay } from "./components/RewardOverlay";
import { SettingsOverlay } from "./components/SettingsOverlay";

interface RewardItem {
  name: string;
  platValue: number;
  ducatValue: number;
}

const LOADING_ITEMS: RewardItem[] = Array(4).fill({ name: "", platValue: 0, ducatValue: 0 });
const isSettingsWindow = getCurrentWindow().label === "settings";

// ── Settings window ──────────────────────────────────────────────────────────

function SettingsApp() {
  const [logPath, setLogPath] = useState("");

  useEffect(() => {
    const unsub = listen<string>("log-path", e => setLogPath(e.payload));
    return () => { unsub.then(f => f()); };
  }, []);

  return (
    <div className="h-screen overflow-hidden" style={{ background: "#0a0a14" }}>
      <SettingsOverlay logPath={logPath} />
    </div>
  );
}

// ── Overlay window ───────────────────────────────────────────────────────────

function OverlayApp() {
  const [showOverlay, setShowOverlay] = useState(false);
  const [items, setItems]             = useState<RewardItem[]>(LOADING_ITEMS);

  useEffect(() => {
    const unsubs = [
      listen<string>("reward-screen-detected", () => {
        setItems(LOADING_ITEMS);
        setShowOverlay(true);
      }),
      listen<RewardItem[]>("reward-items-ready", e => setItems(e.payload)),
      listen("reward-screen-dismissed", () => setShowOverlay(false)),
      listen("toggle-overlay", () => setShowOverlay(v => !v)),
    ];
    return () => { unsubs.forEach(p => p.then(f => f())); };
  }, []);

  return (
    <div className="min-h-screen bg-transparent text-wf-text flex flex-col items-center justify-start pt-2 px-4">
      {showOverlay && <RewardOverlay items={items} />}
    </div>
  );
}

// ── Entry point ──────────────────────────────────────────────────────────────

function App() {
  return isSettingsWindow ? <SettingsApp /> : <OverlayApp />;
}

export default App;
