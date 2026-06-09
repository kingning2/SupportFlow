import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { BrowserRouter } from "react-router-dom";

import "./index.css";
import "antd/dist/reset.css";
import "@supportflow/ui/design-system";
import "@supportflow/ui/design-system/flavors/wework";
import "@/features/wework/styles/wework-console.css";

import { App } from "./App";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <BrowserRouter>
      <App />
    </BrowserRouter>
  </StrictMode>
);
