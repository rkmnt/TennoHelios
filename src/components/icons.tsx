import platImg from "../assets/plat.png";
import ducatImg from "../assets/ducat.png";

export function PlatIcon({ size = 16 }: { size?: number }) {
  return (
    <img src={platImg} width={size} height={size} alt="platinum" />
  );
}

export function DucatIcon({ size = 16 }: { size?: number }) {
  // Ducat image has more whitespace — render larger inside a fixed container
  return (
    <span
      style={{
        display: "inline-flex",
        alignItems: "center",
        justifyContent: "center",
        width: size,
        height: size,
        flexShrink: 0,
      }}
    >
      <img src={ducatImg} width={size * 1.5} height={size * 1.5} alt="ducats" />
    </span>
  );
}
