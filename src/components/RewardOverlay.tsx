import { ItemCard } from "./ItemCard";
import { BestPickIndicator } from "./BestPickIndicator";

interface RewardItem {
  name: string;
  platValue: number;
  ducatValue: number;
}

interface RewardOverlayProps {
  items: RewardItem[];
}

export function RewardOverlay({ items: allItems }: RewardOverlayProps) {
  const items = allItems.filter(item => item.name !== "");
  const known = items;
  const allZeroPlat  = known.every(item => item.platValue  === 0);
  const allZeroDucat = known.every(item => item.ducatValue === 0);
  const bestPlatIdx    = allZeroPlat  ? -1 : items.reduce((b, item, i) => item.name && item.platValue  > (items[b]?.platValue  ?? -1) ? i : b, -1);
  const bestDucatIdx   = allZeroDucat ? -1 : items.reduce((b, item, i) => item.name && item.ducatValue > (items[b]?.ducatValue ?? -1) ? i : b, -1);
  const bestOverallIdx = bestPlatIdx;

  // Rank by plat value descending (1 = best)
  const sorted = [...items].map((item, i) => ({ i, plat: item.platValue }))
    .sort((a, b) => b.plat - a.plat);
  const rankMap = Object.fromEntries(sorted.map((entry, rank) => [entry.i, rank + 1]));

  return (
    <div className="flex flex-col gap-2 w-full max-w-[1440px] px-8">

      {/* Header */}
      <div
        className="flex items-center gap-3"
        style={{ animation: "header-in 1.5s cubic-bezier(0.16, 1, 0.3, 1) both" }}
      >
        {/* Left line with ticks */}
        <div className="relative flex-1 flex items-center">
          <div className="flex-1 h-px bg-gradient-to-r from-transparent to-wf-accent opacity-30" />
          <div className="w-px h-2 bg-wf-accent opacity-40 mx-1" />
          <div className="w-px h-1 bg-wf-accent opacity-25 mx-0.5" />
        </div>

        <div className="flex items-center gap-2 px-1">
          <svg width="16" height="18" viewBox="0 0 16 18" fill="none">
            <path d="M8 0L15.7 4.5V13.5L8 18L0.3 13.5V4.5L8 0Z" fill="none" stroke="#38bdf8" strokeWidth="1" opacity="0.7" />
            <path d="M8 4L12.5 6.5V11.5L8 14L3.5 11.5V6.5L8 4Z" fill="#38bdf8" opacity="0.2" />
          </svg>
          <span className="text-wf-accent text-[10px] tracking-[0.35em] uppercase font-mono opacity-75">
            Tenno Helios Scan
          </span>
        </div>

        {/* Right line with ticks */}
        <div className="relative flex-1 flex items-center">
          <div className="w-px h-1 bg-wf-accent opacity-25 mx-0.5" />
          <div className="w-px h-2 bg-wf-accent opacity-40 mx-1" />
          <div className="flex-1 h-px bg-gradient-to-l from-transparent to-wf-accent opacity-30" />
        </div>
      </div>

      {/* Cards row */}
      <div className="flex gap-2">
        {items.map((item, i) => (
          <div key={i} className="relative flex flex-col flex-1" style={{ minWidth: 0 }}>

            {/* Arrow above best card */}
            {i === bestOverallIdx ? (
              <BestPickIndicator />
            ) : (
              <div style={{ height: 52 }} />
            )}

            <ItemCard
              index={i}
              rank={rankMap[i]}
              name={item.name}
              platValue={item.platValue}
              ducatValue={item.ducatValue}
              isBestPlat={i === bestPlatIdx}
              isBestDucat={i === bestDucatIdx}
              isBestOverall={i === bestOverallIdx}
            />
          </div>
        ))}
      </div>
    </div>
  );
}
