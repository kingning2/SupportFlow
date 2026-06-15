"use client";

import { Layout } from "@douyinfe/semi-ui-19";
import type { CSSProperties, ReactNode } from "react";

import { cn } from "@supportflow/shared";

const { Sider, Content, Header } = Layout;

const panelLayoutStyle: CSSProperties = {
  display: "flex",
  flexDirection: "column",
  height: "100%",
  minHeight: 0
};

/** 工作区根：贴边铺满（飞书式） */
export function WeworkWorkspace({
  className,
  children,
  style
}: {
  className?: string;
  children: ReactNode;
  style?: CSSProperties;
}) {
  return (
    <Layout className={cn("wework-workspace", className)} style={{ ...panelLayoutStyle, ...style }}>
      {children}
    </Layout>
  );
}

/** 横向分栏：固定宽列表 + 弹性主区 */
export function WeworkWorkspaceSplit({ children }: { children: ReactNode }) {
  return (
    <Layout
      className="wework-workspace-split"
      style={{ flex: 1, minHeight: 0, overflow: "hidden" }}
    >
      {children}
    </Layout>
  );
}

/** 左侧列表面板（固定宽度） */
export function WeworkListPanel({
  className,
  children
}: {
  className?: string;
  children: ReactNode;
}) {
  return (
    <Sider className={cn("wework-list-panel", className)} style={panelLayoutStyle}>
      {children}
    </Sider>
  );
}

/** 右侧主内容区 */
export function WeworkMainPanel({
  className,
  children,
  style
}: {
  className?: string;
  children: ReactNode;
  style?: CSSProperties;
}) {
  return (
    <Content
      className={cn("wework-main-panel", className)}
      style={{ ...panelLayoutStyle, ...style }}
    >
      {children}
    </Content>
  );
}

/** 面板顶栏 */
export function WeworkPanelHeader({
  className,
  children,
  style
}: {
  className?: string;
  children: ReactNode;
  style?: CSSProperties;
}) {
  return (
    <Header className={cn("wework-panel-header", className)} style={style}>
      {children}
    </Header>
  );
}

/** 面板可滚动内容区 */
export function WeworkPanelBody({
  className,
  children,
  style
}: {
  className?: string;
  children: ReactNode;
  style?: CSSProperties;
}) {
  return (
    <Content
      className={cn("wework-panel-body", className)}
      style={{ flex: 1, minHeight: 0, overflow: "auto", ...style }}
    >
      {children}
    </Content>
  );
}

/** 单列全页（账号、技能等） */
export function WeworkPageSingle({ children }: { children: ReactNode }) {
  return (
    <Layout className="wework-page-single" style={panelLayoutStyle}>
      {children}
    </Layout>
  );
}

/** 单列全页可滚动主体 */
export function WeworkPageSingleBody({
  children,
  style
}: {
  children: ReactNode;
  style?: CSSProperties;
}) {
  return (
    <Content
      className="wework-page-single__body"
      style={{ flex: 1, minHeight: 0, overflow: "auto", padding: "1rem 1.25rem", ...style }}
    >
      {children}
    </Content>
  );
}
