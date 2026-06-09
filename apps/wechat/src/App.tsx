import { Route, Routes } from "react-router-dom";

import { DesktopAppLayout } from "@supportflow/ui/app-shell";

import { wechatShellAccent } from "@/shell-accent";
import { WechatPage } from "@/wechat-page";

export function App() {
  return (
    <DesktopAppLayout accent={wechatShellAccent}>
      <Routes>
        <Route path="*" element={<WechatPage />} />
      </Routes>
    </DesktopAppLayout>
  );
}
