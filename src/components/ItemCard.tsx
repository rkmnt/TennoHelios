import { PlatIcon, DucatIcon } from "./icons";
import { useCountUp } from "../hooks/useCountUp";

interface ItemCardProps {
  name: string;
  platValue: number;
  ducatValue: number;
  isBestPlat: boolean;
  isBestDucat: boolean;
  isBestOverall: boolean;
  index: number;
  rank: number; // 1 = best plat, 4 = worst
}

// L-shaped corner bracket
function Corner({ position }: { position: "tl" | "tr" | "bl" | "br" }) {
  const size = 14;
  const thickness = 2;
  const style: React.CSSProperties = { position: "absolute", width: size, height: size, color: "currentColor" };

  const pos: React.CSSProperties =
    position === "tl" ? { top: 5, left: 5, borderTop: `${thickness}px solid`, borderLeft: `${thickness}px solid` }
    : position === "tr" ? { top: 5, right: 5, borderTop: `${thickness}px solid`, borderRight: `${thickness}px solid` }
    : position === "bl" ? { bottom: 5, left: 5, borderBottom: `${thickness}px solid`, borderLeft: `${thickness}px solid` }
    : { bottom: 5, right: 5, borderBottom: `${thickness}px solid`, borderRight: `${thickness}px solid` };

  return <div style={{ ...style, ...pos }} />;
}

export function ItemCard({ name, platValue, ducatValue, isBestPlat, isBestDucat, isBestOverall, index, rank }: ItemCardProps) {
  const stagger      = index * 100;
  const platDisplay  = useCountUp(platValue,  stagger + 350);
  const ducatDisplay = useCountUp(ducatValue, stagger + 450);

  const accentColor = isBestPlat ? "#4ade80" : isBestDucat ? "#facc15" : "#38bdf855";
  const glowColor   = isBestPlat ? "rgba(74,222,128,0.1)" : isBestDucat ? "rgba(250,204,21,0.1)" : "transparent";
  const scanColor   = isBestOverall ? "#38bdf8" : isBestPlat ? "#4ade80" : isBestDucat ? "#facc15" : "#2a2a3a";

  if (!name) return null;

  return (
    <div
      className="relative flex flex-col"
      style={{
        clipPath: "polygon(0 0, calc(100% - 12px) 0, 100% 12px, 100% 100%, 12px 100%, 0 calc(100% - 12px))",
        border: `1px solid ${accentColor}`,
        boxShadow: isBestPlat || isBestDucat
          ? `inset 0 0 60px ${glowColor}, 0 0 30px ${glowColor}, 0 0 2px ${accentColor}44`
          : "none",
        background: isBestPlat || isBestDucat
          ? `linear-gradient(160deg, #12121a 60%, ${accentColor}08 100%)`
          : "#12121a",
        flex: 1,
        padding: "18px 16px",
        animation: `card-in 0.7s cubic-bezier(0.16, 1, 0.3, 1) ${stagger}ms both`,
        minWidth: 0,
      }}
    >
      {/* Hex pattern background */}
      <div className="absolute inset-0 pointer-events-none opacity-[0.03]"
        style={{
          backgroundImage: `url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='28' height='32'%3E%3Cpolygon points='14,2 26,9 26,23 14,30 2,23 2,9' fill='none' stroke='%2338bdf8' stroke-width='0.8'/%3E%3C/svg%3E")`,
          backgroundSize: "28px 32px",
        }}
      />

      {/* Glow pulse overlay */}
      {(isBestPlat || isBestDucat) && (
        <div className="absolute inset-0 pointer-events-none"
          style={{
            background: `radial-gradient(ellipse at 50% 0%, ${glowColor.replace("0.1", "0.2")} 0%, transparent 65%)`,
            animation: `glow-pulse 2.5s ease-in-out ${stagger + 700}ms infinite`,
          }}
        />
      )}

      {/* Animated scan sweep */}
      <div className="absolute top-0 left-0 right-0 h-px overflow-hidden pointer-events-none">
        <div style={{
          position: "absolute", top: 0, left: 0, width: "45%", height: "100%",
          background: `linear-gradient(90deg, transparent, ${scanColor}dd, transparent)`,
          animation: `scan-sweep 1.4s ease-in-out ${stagger + 150}ms both`,
        }} />
      </div>

      {/* Static top line */}
      <div className="absolute top-0 left-0 right-0 h-px"
        style={{ background: `linear-gradient(90deg, transparent, ${accentColor}, transparent)` }}
      />

      {/* 4-corner brackets */}
      <div style={{ color: accentColor, opacity: isBestPlat || isBestDucat ? 1 : 0.55 }}>
        <Corner position="tl" />
        <Corner position="tr" />
        <Corner position="bl" />
        <Corner position="br" />
      </div>

      {/* Rank number */}
      <div className="absolute top-1.5 right-8 font-mono text-xs font-bold tracking-widest"
        style={{ color: accentColor, opacity: 0.55 }}>
        #{rank}
      </div>

      {/* Item name */}
      <p className="text-wf-text text-[11px] uppercase tracking-widest leading-snug font-semibold mb-4 mt-1"
        style={{ minHeight: "2.8rem" }}>
        {name}
      </p>

      {/* Diamond separator */}
      <div className="flex items-center gap-2 mb-4">
        <div className="flex-1 h-px" style={{ background: `linear-gradient(90deg, ${accentColor}44, transparent)` }} />
        <svg width="6" height="6" viewBox="0 0 6 6">
          <rect x="1" y="1" width="4" height="4" transform="rotate(45 3 3)"
            fill="none" stroke={accentColor} strokeWidth="0.8" opacity="0.6" />
        </svg>
        <div className="flex-1 h-px" style={{ background: `linear-gradient(90deg, transparent, ${accentColor}44)` }} />
      </div>

      {/* Values */}
      <div className="flex flex-col gap-2.5">
        {/* Plat row */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <span className="text-[11px] tracking-widest uppercase font-mono" style={{ color: `${accentColor}bb` }}>plat</span>
            {isBestPlat && (
              <span className="text-[10px] tracking-widest uppercase font-mono px-2 py-0.5"
                style={{ color: "#4ade80", border: "1px solid #4ade8066", background: "#4ade8018" }}>
                best
              </span>
            )}
          </div>
          <div className="flex items-center gap-1.5">
            <span className="text-xl font-bold font-mono leading-none tabular-nums"
              style={{ color: isBestPlat ? "#4ade80" : "#c0c0cc" }}>
              {platDisplay}
            </span>
            <PlatIcon size={17} />
          </div>
        </div>

        {/* Ducat row */}
        {ducatValue > 0 && (
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <span className="text-[11px] tracking-widest uppercase font-mono" style={{ color: `${accentColor}bb` }}>ducat</span>
              {isBestDucat && (
                <span className="text-[10px] tracking-widest uppercase font-mono px-2 py-0.5"
                  style={{ color: "#facc15", border: "1px solid #facc1566", background: "#facc1518" }}>
                  best
                </span>
              )}
            </div>
            <div className="flex items-center gap-1.5">
              <span className="text-xl font-bold font-mono leading-none tabular-nums"
                style={{ color: isBestDucat ? "#facc15" : "#c0c0cc" }}>
                {ducatDisplay}
              </span>
              <DucatIcon size={17} />
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
