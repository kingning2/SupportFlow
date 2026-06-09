import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { createBrowserRouter, RouterProvider } from "react-router-dom";

import "./index.css";
import "antd/dist/reset.css";
import "@supportflow/ui/design-system";
import "@supportflow/ui/design-system/flavors/wework";
import "@/features/wework/styles/wework-console.css";

import { App } from "./App";
import { WeworkAppPage } from "./features/wework/page";
import {
  AccountRoute,
  AiConfigRoute,
  InboxRoute,
  KnowledgeRoute,
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
          { path: "account", element: <AccountRoute /> },
          { path: "mcp", element: <McpRoute /> },
          { path: "ai_config", element: <AiConfigRoute /> }
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
