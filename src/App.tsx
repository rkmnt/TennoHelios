import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { RewardOverlay } from "./components/RewardOverlay";
import { SettingsOverlay } from "./components/SettingsOverlay";

interface RewardItem {
  name: string;
  platValue: number;
  ducatValue: number;
}

const LOADING_ITEMS: RewardItem[] = Array(4).fill({ name: "", platValue: 0, ducatValue: 0 });

function App() {
  const [showOverlay, setShowOverlay]   = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [items, setItems]               = useState<RewardItem[]>(LOADING_ITEMS);
  const [logPath, setLogPath]           = useState("");

  useEffect(() => {
    const unsubs = [
      listen<string>("reward-screen-detected", () => {
        setItems(LOADING_ITEMS);
        setShowOverlay(true);
        setShowSettings(false);
      }),
      listen<RewardItem[]>("reward-items-ready", e => setItems(e.payload)),
      listen("reward-screen-dismissed", () => setShowOverlay(false)),
      listen("toggle-overlay", () => setShowOverlay(v => !v)),
      listen("show-settings", () => { setShowSettings(true);  setShowOverlay(false); }),
      listen("hide-settings", () => { setShowSettings(false); }),
      listen<string>("log-path", e => setLogPath(e.payload)),
    ];
    return () => { unsubs.forEach(p => p.then(f => f())); };
  }, []);

  return (
    <div className="min-h-screen bg-transparent text-wf-text flex flex-col items-center justify-start pt-2 px-4">
      {showOverlay && <RewardOverlay items={items} />}
      {showSettings && (
        <SettingsOverlay logPath={logPath} />
      )}
    </div>
  );
}

export default App;
