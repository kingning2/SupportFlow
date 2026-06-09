import { Outlet } from "react-router-dom";

import { DesktopAppLayout } from "@supportflow/ui/app-shell";

import { weworkShellAccent } from "./config/shell-accent";
import { WEWORK_SHELL_CONTENT_CLASS } from "./config/shell";

/** 根布局：DesktopAppLayout 包裹子路由通过 Outlet 渲染 */
export function App() {
  return (
    <DesktopAppLayout accent={weworkShellAccent} contentClassName={WEWORK_SHELL_CONTENT_CLASS}>
      <Outlet />
    </DesktopAppLayout>
  );
}
