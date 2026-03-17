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
    },
  },
  plugins: [],
};
