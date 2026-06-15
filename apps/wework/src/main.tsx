import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { createBrowserRouter, RouterProvider } from "react-router-dom";

import "./index.css";
import "@supportflow/ui/design-system";
import "@supportflow/ui/design-system/flavors/wework";
import "@/features/wework/styles/wework-console.css";

import { App } from "./App";
import { WeworkAppPage } from "./features/wework/page";
import {
  AccountRoute,
  AiChatRoute,
  AiConfigRoute,
  InboxRoute,
  KnowledgeRoute,
  LicenseRoute,
  McpRoute,
  SkillsRoute
} from "./features/wework/app";

const router = createBrowserRouter([
  {
    element: <App />,
    children: [
      {
        element: <WeworkAppPage />,
        children: [
          { index: true, element: <InboxRoute /> },
          { path: "inbox", element: <InboxRoute /> },
          { path: "knowledge", element: <KnowledgeRoute /> },
          { path: "skills", element: <SkillsRoute /> },
          { path: "ai_chat", element: <AiChatRoute /> },
          { path: "account", element: <AccountRoute /> },
          { path: "mcp", element: <McpRoute /> },
          { path: "ai_config", element: <AiConfigRoute /> },
          { path: "license", element: <LicenseRoute /> }
        ]
      }
    ]
  }
]);

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <RouterProvider router={router} />
  </StrictMode>
);
