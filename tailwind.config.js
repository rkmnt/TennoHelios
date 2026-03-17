/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        // Warframe-inspired palette
        "wf-bg": "#0a0a0f",
        "wf-surface": "#12121a",
        "wf-border": "#1e1e2e",
        "wf-text": "#e0e0e8",
        "wf-plat": "#4ade80",   // green — best platinum
        "wf-ducat": "#facc15",  // yellow — best ducat
        "wf-accent": "#38bdf8", // sky blue — UI accent
      },
      keyframes: {
        "card-in": {
          "0%":   { opacity: "0", transform: "translateY(16px)" },
          "100%": { opacity: "1", transform: "translateY(0)" },
        },
        "scan-sweep": {
          "0%":   { transform: "translateX(-100%)", opacity: "0" },
          "20%":  { opacity: "1" },
          "80%":  { opacity: "1" },
          "100%": { transform: "translateX(100%)", opacity: "0" },
        },
        "glow-pulse": {
          "0%, 100%": { opacity: "1" },
          "50%":      { opacity: "0.5" },
        },
        "arrow-bounce": {
          "0%, 100%": { transform: "translateX(-50%) translateY(0)" },
          "50%":      { transform: "translateX(-50%) translateY(3px)" },
        },
      },
      animation: {
        "card-in":      "card-in 0.4s cubic-bezier(0.16, 1, 0.3, 1) both",
        "scan-sweep":   "scan-sweep 1.2s ease-in-out both",
        "glow-pulse":   "glow-pulse 2s ease-in-out infinite",
        "arrow-bounce": "arrow-bounce 1.5s ease-in-out infinite",
      },
    },
  },
  plugins: [],
};
