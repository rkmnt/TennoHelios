interface SettingsOverlayProps {
  logPath: string;
}

function Row({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex items-center justify-between gap-4 py-2.5"
      style={{ borderBottom: "1px solid #1a1a2e" }}>
      <span className="text-[11px] tracking-widest uppercase font-mono" style={{ color: "#38bdf866" }}>
        {label}
      </span>
      <span className="text-[12px] font-mono" style={{ color: "#c0c0cc" }}>
        {children}
      </span>
    </div>
  );
}

function Kbd({ keys }: { keys: string }) {
  return (
    <span className="px-2 py-0.5 font-mono text-[11px] tracking-wide"
      style={{ border: "1px solid #38bdf844", color: "#38bdf8", background: "#38bdf810" }}>
      {keys}
    </span>
  );
}

export function SettingsOverlay({ logPath }: SettingsOverlayProps) {
  return (
    <div className="fixed inset-0 flex items-center justify-center"
      style={{ background: "rgba(0,0,0,0.6)", zIndex: 50 }}>
      <div
        className="relative w-full max-w-lg p-6 flex flex-col gap-4"
        style={{
          background: "#0d0d14",
          border: "1px solid #38bdf833",
          clipPath: "polygon(0 0, calc(100% - 18px) 0, 100% 18px, 100% 100%, 18px 100%, 0 calc(100% - 18px))",
          boxShadow: "0 0 60px rgba(56,189,248,0.06)",
        }}
      >
        {/* Header */}
        <div className="flex items-center gap-3 pb-3" style={{ borderBottom: "1px solid #1a1a2e" }}>
          <svg width="18" height="20" viewBox="0 0 16 18" fill="none">
            <path d="M8 0L15.7 4.5V13.5L8 18L0.3 13.5V4.5L8 0Z" stroke="#38bdf8" strokeWidth="1" opacity="0.7" fill="none"/>
            <path d="M8 4L12.5 6.5V11.5L8 14L3.5 11.5V6.5L8 4Z" fill="#38bdf8" opacity="0.2"/>
          </svg>
          <div>
            <div className="text-[13px] tracking-[0.3em] uppercase font-mono" style={{ color: "#38bdf8" }}>
              TennoHelios
            </div>
            <div className="text-[10px] tracking-widest uppercase font-mono" style={{ color: "#38bdf855" }}>
              v0.1.0 · Linux Warframe Overlay
            </div>
          </div>
          <div className="ml-auto font-mono text-[10px] tracking-widest" style={{ color: "#38bdf833" }}>
            Ctrl+Shift+H to close
          </div>
        </div>

        {/* Status */}
        <div>
          <div className="text-[10px] tracking-widest uppercase font-mono mb-2" style={{ color: "#38bdf844" }}>
            Status
          </div>
          <Row label="Log watcher">
            <span style={{ color: "#4ade80" }}>● Active</span>
          </Row>
          <Row label="EE.log path">
            <span className="text-[10px] opacity-60 truncate max-w-[260px]">{logPath || "default"}</span>
          </Row>
        </div>

        {/* Shortcuts */}
        <div>
          <div className="text-[10px] tracking-widest uppercase font-mono mb-2" style={{ color: "#38bdf844" }}>
            Keyboard shortcuts
          </div>
          <Row label="Test overlay"><Kbd keys="F12" /></Row>
          <Row label="Settings"><Kbd keys="Ctrl+Shift+H" /></Row>
        </div>

        {/* Footer */}
        <div className="text-[10px] font-mono text-center pt-1" style={{ color: "#38bdf833" }}>
          GPL-3.0 · github.com/rkmnt/TennoHelios
        </div>
      </div>
    </div>
  );
}
