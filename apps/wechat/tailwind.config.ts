import type { Config } from "tailwindcss";

export default {
  content: [
    "./src/**/*.{js,ts,jsx,tsx,mdx}",
    "../../packages/app-shell/src/**/*.{js,ts,jsx,tsx}",
    "../../packages/channel-wechat/src/**/*.{js,ts,jsx,tsx}",
    "../../packages/channel-core/src/**/*.{js,ts,jsx,tsx}"
  ]
} satisfies Config;
