import { createRequire } from "node:module";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import tsconfigPaths from "vite-tsconfig-paths";
import semiPlugin from "@douyinfe/semi-vite-plugin";

const FEISHU_DSM_THEME = "@semi-bot/semi-theme-feishu-dashboard";

const require = createRequire(import.meta.url);

function assertFeishuThemeInstalled(): void {
  try {
    require.resolve(`${FEISHU_DSM_THEME}/scss/index.scss`);
  } catch {
    throw new Error(
      `缺少飞书 DSM 主题包 ${FEISHU_DSM_THEME}。请在仓库根目录执行 pnpm install 后重启开发服务。`
    );
  }
}

assertFeishuThemeInstalled();

export default defineConfig({
  plugins: [
    react(),
    tailwindcss(),
    tsconfigPaths(),
    semiPlugin({
      theme: FEISHU_DSM_THEME
    })
  ],
  resolve: { dedupe: ["react", "react-dom"] },
  build: { outDir: "out", emptyOutDir: true },
  server: { port: 3000 }
});
