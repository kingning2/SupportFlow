import { Route, Routes } from "react-router-dom";

import { DesktopAppLayout } from "@supportflow/ui/app-shell";

import { __CHANNEL_CAMEL__ShellAccent } from "@/config/shell-accent";
import { __CHANNEL_CONST___SHELL_CONTENT_CLASS } from "@/config/shell";
import { __CHANNEL_PAGE_COMPONENT__ } from "@/features/__CHANNEL_SLUG__/__CHANNEL_SLUG__-page";

export function App() {
  return (
    <DesktopAppLayout
      accent={__CHANNEL_CAMEL__ShellAccent}
      contentClassName={__CHANNEL_CONST___SHELL_CONTENT_CLASS}
    >
      <Routes>
        <Route path="*" element={<__CHANNEL_PAGE_COMPONENT__ />} />
      </Routes>
    </DesktopAppLayout>
  );
}
