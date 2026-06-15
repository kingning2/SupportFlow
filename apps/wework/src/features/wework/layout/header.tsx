"use client";

import { Layout, Typography } from "@douyinfe/semi-ui-19";

import { WeworkConsoleRoute } from "@supportflow/shared/tauri-bridge/enums";

import { WEWORK_ROUTE_PAGE_LABEL } from "../constants/wework-nav";

const { Header: SemiHeader } = Layout;
const { Title, Text } = Typography;

/** 仅单列占位页显示顶栏；分栏页（收件箱、知识库等）自带页头 */
const SIMPLE_HEADER_ROUTES = new Set<WeworkConsoleRoute>([
  WeworkConsoleRoute.Mcp,
  WeworkConsoleRoute.AiConfig
]);

const ROUTE_HEADER_DESCRIPTION: Partial<Record<WeworkConsoleRoute, string>> = {
  [WeworkConsoleRoute.AiConfig]: "管理 API 供应商、Key 与对话模型"
};

export interface HeaderProps {
  activeRoute: WeworkConsoleRoute;
}

export function Header({ activeRoute }: HeaderProps) {
  if (activeRoute === WeworkConsoleRoute.Inbox || !SIMPLE_HEADER_ROUTES.has(activeRoute)) {
    return null;
  }

  const description = ROUTE_HEADER_DESCRIPTION[activeRoute];

  return (
    <SemiHeader className="wework-panel-header">
      <div>
        <Title heading={6} style={{ margin: 0 }}>
          {WEWORK_ROUTE_PAGE_LABEL[activeRoute]}
        </Title>
        {description ? (
          <Text type="tertiary" size="small" style={{ display: "block", marginTop: 4 }}>
            {description}
          </Text>
        ) : null}
      </div>
    </SemiHeader>
  );
}
