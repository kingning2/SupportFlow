import { Navigate, Route, Routes } from "react-router-dom";

import { AppShellLayout, APP_SHELL_CONTENT_CLASS, DesktopAppRoot } from "@supportflow/ui/app-shell";

import { Page as MainWindowPage } from "@/features/full/main-window/page";
import { Page as ModalWindowPage } from "@/features/full/modal-window/page";

export function App() {
  return (
    <DesktopAppRoot>
      <Routes>
        <Route path="/" element={<Navigate to="/main-window" replace />} />
        <Route
          path="/main-window"
          element={
            <AppShellLayout modal bgGuard contentClassName={APP_SHELL_CONTENT_CLASS.console}>
              <MainWindowPage />
            </AppShellLayout>
          }
        />
        <Route path="/modal-window" element={<ModalWindowPage />} />
      </Routes>
    </DesktopAppRoot>
  );
}
