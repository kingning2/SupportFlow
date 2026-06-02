import type { Config } from "tailwindcss";

export default {
  content: [
    "./src/**/*.{js,ts,jsx,tsx,mdx}",
    "../../packages/ui/src/**/*.{js,ts,jsx,tsx}",
    "../../packages/agent-console/src/**/*.{js,ts,jsx,tsx}",
    "../../packages/channel-core/src/**/*.{js,ts,jsx,tsx}",
    "../../packages/channel-wework/src/**/*.{js,ts,jsx,tsx}"
  ]
} satisfies Config;
