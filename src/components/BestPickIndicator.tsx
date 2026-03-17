// Animované "Best Pick" Warframe-style označení nad kartou

export function BestPickIndicator() {
  return (
    <div
      className="flex flex-col items-center gap-1 mb-2"
      style={{
        opacity: 0,
        animation: "best-pick-in 0.6s ease-out 900ms forwards",
      }}
    >
      {/* Bracket row: [  OPTIMAL PICK  ] */}
      <div className="flex items-center gap-2">
        {/* Left bracket */}
        <div className="flex items-center gap-0.5" style={{ animation: "bracket-left 0.4s ease-out 1000ms both" }}>
          <div style={{
            width: 6, height: 14,
            borderTop: "1.5px solid #38bdf8",
            borderLeft: "1.5px solid #38bdf8",
            borderBottom: "1.5px solid #38bdf8",
          }} />
        </div>

        {/* Text */}
        <span
          className="text-[9px] tracking-[0.4em] uppercase font-mono"
          style={{
            color: "#38bdf8",
            animation: "best-pick-text 2s ease-in-out 1400ms infinite",
          }}
        >
          optimal pick
        </span>

        {/* Right bracket */}
        <div style={{ animation: "bracket-right 0.4s ease-out 1000ms both" }}>
          <div style={{
            width: 6, height: 14,
            borderTop: "1.5px solid #38bdf8",
            borderRight: "1.5px solid #38bdf8",
            borderBottom: "1.5px solid #38bdf8",
          }} />
        </div>
      </div>

      {/* Double chevron arrow */}
      <div
        className="flex flex-col items-center"
        style={{ gap: 2, animation: "arrow-bounce 1.8s ease-in-out 1500ms infinite" }}
      >
        <svg width="16" height="8" viewBox="0 0 16 8" fill="none">
          <path d="M1 1L8 7L15 1" stroke="#38bdf8" strokeWidth="1.5" strokeLinecap="round" />
        </svg>
        <svg width="16" height="8" viewBox="0 0 16 8" fill="none">
          <path d="M1 1L8 7L15 1" stroke="#38bdf8" strokeWidth="1.5" strokeLinecap="round" opacity="0.4" />
        </svg>
      </div>
    </div>
  );
}
