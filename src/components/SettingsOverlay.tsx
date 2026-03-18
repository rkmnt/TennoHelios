import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

interface SettingsOverlayProps {
  logPath: string;
}

function Row({ label, children, last }: { label: string; children: React.ReactNode; last?: boolean }) {
  return (
    <div className="flex items-start justify-between gap-4 py-2.5"
      style={last ? undefined : { borderBottom: "1px solid #1a1a2e" }}>
      <span className="text-[11px] tracking-widest uppercase font-mono shrink-0 pt-0.5" style={{ color: "#38bdf8aa" }}>
        {label}
      </span>
      <span className="text-[12px] font-mono text-right" style={{ color: "#c0c0cc" }}>
        {children}
      </span>
    </div>
  );
}

export function SettingsOverlay({ logPath }: SettingsOverlayProps) {
  const [customPath, setCustomPath] = useState(logPath);
  const [saved, setSaved] = useState(false);
  const changed = customPath.trim() !== logPath;

  // Screenshot delay
  const [delay, setDelay] = useState(1500);
  const [delaySaved, setDelaySaved] = useState(false);
  const [delayInput, setDelayInput] = useState("1500");

  // Overlay Y percent
  const [overlayY, setOverlayY] = useState(50);
  const [overlaySaved, setOverlaySaved] = useState(false);

  // Warframe version
  const [wfVersion, setWfVersion] = useState("…");

  useEffect(() => {
    invoke<number>("get_screenshot_delay").then(v => {
      setDelay(v);
      setDelayInput(String(v));
    });
    invoke<number>("get_overlay_y_percent").then(v => setOverlayY(v));
    invoke<string>("get_warframe_version").then(v => setWfVersion(v || "Unknown"));
  }, []);

  // Sync customPath when logPath prop changes (initial load from event)
  useEffect(() => {
    setCustomPath(logPath);
  }, [logPath]);

  function handleClose() {
    invoke("close_settings");
  }

  function handleSavePath() {
    const path = customPath.trim();
    if (!path) return;
    invoke("set_log_path", { path });
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  }

  function handleSaveDelay() {
    const ms = Math.round(Number(delayInput));
    if (isNaN(ms)) return;
    invoke("set_screenshot_delay", { ms });
    setDelay(ms);
    setDelaySaved(true);
    setTimeout(() => setDelaySaved(false), 2000);
  }

  function handleOverlayY(v: number) {
    setOverlayY(v);
    invoke("set_overlay_y_percent", { percent: v });
    setOverlaySaved(true);
    setTimeout(() => setOverlaySaved(false), 1500);
  }

  const delayChanged = delayInput !== String(delay);

  return (
    <div
      className="w-full h-full p-6 flex flex-col gap-4"
      style={{ background: "#0d0d14", border: "1px solid #38bdf822" }}
      onMouseDown={e => {
        if (e.target === e.currentTarget) {
          e.preventDefault();
          getCurrentWindow().startDragging();
        }
      }}
    >
      <div className="flex flex-col h-full">
        {/* Header */}
        <div
          className="flex items-center gap-3 pb-3 mb-4 cursor-grab active:cursor-grabbing select-none shrink-0"
          style={{ borderBottom: "1px solid #1a1a2e" }}
          onMouseDown={e => { e.preventDefault(); getCurrentWindow().startDragging(); }}
        >
          <svg width="18" height="20" viewBox="0 0 16 18" fill="none">
            <path d="M8 0L15.7 4.5V13.5L8 18L0.3 13.5V4.5L8 0Z" stroke="#38bdf8" strokeWidth="1" opacity="0.7" fill="none"/>
            <path d="M8 4L12.5 6.5V11.5L8 14L3.5 11.5V6.5L8 4Z" fill="#38bdf8" opacity="0.2"/>
          </svg>
          <div>
            <div className="text-[13px] tracking-[0.3em] uppercase font-mono" style={{ color: "#38bdf8" }}>
              TennoHelios
            </div>
            <div className="text-[10px] tracking-widest uppercase font-mono" style={{ color: "#38bdf888" }}>
              v0.1.0 · Linux Warframe Overlay
            </div>
          </div>
          <button
            onClick={handleClose}
            className="ml-auto w-7 h-7 flex items-center justify-center"
            style={{ color: "#38bdf855", background: "transparent", border: "none", cursor: "pointer" }}
            onMouseEnter={e => (e.currentTarget.style.color = "#38bdf8")}
            onMouseLeave={e => (e.currentTarget.style.color = "#38bdf855")}
          >
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
              <path d="M1 1L13 13M13 1L1 13" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round"/>
            </svg>
          </button>
        </div>

        {/* Scrollable content */}
        <div className="flex-1 overflow-y-auto flex flex-col gap-4 pr-1" style={{ scrollbarWidth: "thin", scrollbarColor: "#38bdf822 transparent" }}>

        {/* Status */}
        <div className="pb-2" style={{ borderBottom: "1px solid #1a1a2e" }}>
          <div className="text-[10px] tracking-widest uppercase font-mono mb-2" style={{ color: "#38bdf8bb" }}>
            Status
          </div>
          <Row label="Log watcher">
            <span style={{ color: "#4ade80" }}>● Active</span>
          </Row>
          <Row label="Warframe version" last>
            <span style={{ color: "#c0c0cc" }}>{wfVersion}</span>
          </Row>
        </div>

        {/* EE.log path */}
        <div className="pb-4" style={{ borderBottom: "1px solid #1a1a2e" }}>
          <div className="text-[10px] tracking-widest uppercase font-mono mb-2" style={{ color: "#38bdf8bb" }}>
            EE.log path
          </div>
          <input
            type="text"
            value={customPath}
            onChange={e => { setCustomPath(e.target.value); setSaved(false); }}
            className="w-full text-[11px] font-mono px-3 py-2 outline-none mb-2"
            style={{
              background: "#0a0a10",
              border: `1px solid ${changed ? "#38bdf855" : "#38bdf822"}`,
              color: changed ? "#c0c0cc" : "#c0c0cc88",
            }}
            onFocus={e => (e.currentTarget.style.borderColor = "#38bdf877")}
            onBlur={e => (e.currentTarget.style.borderColor = changed ? "#38bdf855" : "#38bdf822")}
            onKeyDown={e => { if (e.key === "Enter") handleSavePath(); }}
          />
          <div className="flex gap-2">
            <button
              onClick={handleSavePath}
              disabled={!changed}
              className="px-4 py-1.5 text-[11px] font-mono tracking-widest uppercase"
              style={{
                background: saved ? "#4ade8022" : "#38bdf811",
                border: `1px solid ${saved ? "#4ade8066" : "#38bdf844"}`,
                color: saved ? "#4ade80" : changed ? "#38bdf8" : "#38bdf833",
                cursor: changed ? "pointer" : "default",
                transition: "all 0.2s",
              }}
            >
              {saved ? "✓ Saved" : "Apply"}
            </button>
            <button
              onClick={() => { setCustomPath(logPath); setSaved(false); }}
              disabled={!changed}
              className="px-4 py-1.5 text-[11px] font-mono tracking-widest uppercase"
              style={{
                background: "transparent",
                border: "1px solid #38bdf822",
                color: changed ? "#38bdf855" : "#38bdf822",
                cursor: changed ? "pointer" : "default",
                transition: "all 0.2s",
              }}
            >
              Reset
            </button>
          </div>
        </div>

        {/* Screenshot delay */}
        <div className="pb-4" style={{ borderBottom: "1px solid #1a1a2e" }}>
          <div className="text-[10px] tracking-widest uppercase font-mono mb-2" style={{ color: "#38bdf8bb" }}>
            Screenshot delay
          </div>
          <div className="flex items-center gap-2">
            <input
              type="text"
              inputMode="numeric"
              value={delayInput}
              onChange={e => { setDelayInput(e.target.value.replace(/[^0-9]/g, "")); setDelaySaved(false); }}
              onKeyDown={e => { if (e.key === "Enter") handleSaveDelay(); }}
              className="w-24 text-[12px] font-mono px-3 py-2 outline-none text-center"
              style={{
                background: "#0a0a10",
                border: `1px solid ${delayChanged ? "#38bdf855" : "#38bdf822"}`,
                color: "#c0c0cc",
              }}
            />
            <span className="text-[11px] font-mono" style={{ color: "#38bdf855" }}>ms</span>
            <button
              onClick={handleSaveDelay}
              disabled={!delayChanged}
              className="px-4 py-1.5 text-[11px] font-mono tracking-widest uppercase"
              style={{
                background: delaySaved ? "#4ade8022" : "#38bdf811",
                border: `1px solid ${delaySaved ? "#4ade8066" : "#38bdf844"}`,
                color: delaySaved ? "#4ade80" : delayChanged ? "#38bdf8" : "#38bdf833",
                cursor: delayChanged ? "pointer" : "default",
                transition: "all 0.2s",
              }}
            >
              {delaySaved ? "✓ Saved" : "Apply"}
            </button>
            <button
              onClick={() => { setDelayInput(String(delay)); setDelaySaved(false); }}
              disabled={!delayChanged}
              className="px-4 py-1.5 text-[11px] font-mono tracking-widest uppercase"
              style={{
                background: "transparent",
                border: "1px solid #38bdf822",
                color: delayChanged ? "#38bdf855" : "#38bdf822",
                cursor: delayChanged ? "pointer" : "default",
                transition: "all 0.2s",
              }}
            >
              Reset
            </button>
          </div>
          <div className="text-[10px] font-mono mt-1" style={{ color: "#7a7a9a" }}>
            Time to wait after reward screen appears before capturing (500–5000 ms)
          </div>
        </div>

        {/* Overlay Y position */}
        <div>
          <div className="text-[10px] tracking-widest uppercase font-mono mb-2" style={{ color: "#38bdf8bb" }}>
            Overlay position
          </div>
          <div className="flex items-center gap-3">
            <input
              type="range"
              min={10}
              max={90}
              value={overlayY}
              onChange={e => handleOverlayY(Number(e.target.value))}
              className="flex-1"
              style={{ accentColor: "#38bdf8" }}
            />
            <span className="text-[12px] font-mono w-10 text-right" style={{ color: overlaySaved ? "#4ade80" : "#c0c0cc" }}>
              {overlayY}%
            </span>
          </div>
          <div className="text-[10px] font-mono mt-1" style={{ color: "#7a7a9a" }}>
            Vertical position of the reward overlay (% of screen height)
          </div>
        </div>

        </div>{/* end scrollable content */}

        {/* Footer — always visible at bottom */}
        <div className="shrink-0 text-[10px] font-mono text-center pt-4 mt-2" style={{ color: "#7a7a9a", borderTop: "1px solid #1a1a2e" }}>
          GPL-3.0 · github.com/rkmnt/TennoHelios
        </div>
      </div>
    </div>
  );
}
