import { Navigate, Route, Routes } from "react-router-dom";

import { AppShellLayout, APP_SHELL_CONTENT_CLASS, DesktopAppRoot } from "@supportflow/ui/app-shell";

import { FullMainWindowPage } from "@/features/full/main-window/main-window-page";
import { FullModalWindowPage } from "@/features/full/modal-window/modal-window-page";

export function App() {
  return (
    <DesktopAppRoot>
      <Routes>
        <Route path="/" element={<Navigate to="/main-window" replace />} />
        <Route
          path="/main-window"
          element={
            <AppShellLayout modal bgGuard contentClassName={APP_SHELL_CONTENT_CLASS.console}>
              <FullMainWindowPage />
            </AppShellLayout>
          }
        />
        <Route path="/modal-window" element={<FullModalWindowPage />} />
      </Routes>
    </DesktopAppRoot>
  );
}
