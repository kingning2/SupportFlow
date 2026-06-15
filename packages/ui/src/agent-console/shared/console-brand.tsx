"use client";

import { IconSemiLogo } from "@douyinfe/semi-icons";
import { Avatar, Card, Empty, Layout, Typography } from "@douyinfe/semi-ui-19";
import type { ReactNode } from "react";

const { Header, Content } = Layout;
const { Title, Text, Paragraph } = Typography;

export function ConsoleBrandMark({ size = 48 }: { size?: number; className?: string }) {
  return (
    <Avatar
      size="large"
      style={{
        width: size,
        height: size,
        background: "#35A85B",
        color: "#fff",
        boxShadow: "0 10px 24px rgb(53 168 91 / 0.2)"
      }}
    >
      <IconSemiLogo />
    </Avatar>
  );
}

export function ConsoleBrandMarkSmall({ size = 32 }: { size?: number; className?: string }) {
  return (
    <Avatar
      size="small"
      style={{ width: size, height: size, background: "#35A85B", color: "#fff" }}
    >
      <IconSemiLogo />
    </Avatar>
  );
}

export function ViewShell({
  title,
  description,
  children,
  extra,
  className
}: {
  title?: string;
  description?: string;
  children: ReactNode;
  extra?: ReactNode;
  className?: string;
}) {
  const showHeader = Boolean(title || description || extra);

  return (
    <Layout
      className={["agent-console-view-shell", className].filter(Boolean).join(" ")}
      style={{ height: "100%", minHeight: 0, overflow: "hidden" }}
    >
      {showHeader ? (
        <Header
          className="agent-console-view-header"
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            height: "auto",
            lineHeight: "inherit",
            padding: "12px 24px",
            borderBottom: "1px solid var(--semi-color-border)"
          }}
        >
          <div style={{ minWidth: 0 }}>
            {title ? (
              <Title heading={5} style={{ margin: 0 }}>
                {title}
              </Title>
            ) : null}
            {description ? (
              <Text
                type="tertiary"
                size="small"
                style={{ display: "block", marginTop: title ? 4 : 0 }}
              >
                {description}
              </Text>
            ) : null}
          </div>
          {extra}
        </Header>
      ) : null}
      <Content
        style={{
          flex: 1,
          minHeight: 0,
          overflow: "auto",
          padding: showHeader ? 24 : "16px 24px 24px"
        }}
      >
        {children}
      </Content>
    </Layout>
  );
}

export function PlaceholderView({ title, description }: { title: string; description?: string }) {
  const text = description ?? "该功能暂未接入 Rust Agent，请先在对话、配置和技能页面中使用。";

  return (
    <ViewShell title={title} description={text}>
      <Empty description={text} />
    </ViewShell>
  );
}

export function SectionCard({
  title,
  children,
  style,
  className
}: {
  title?: string;
  children: ReactNode;
  style?: React.CSSProperties;
  className?: string;
}) {
  return (
    <Card className={className} style={style} title={title}>
      {children}
    </Card>
  );
}

export function MutedHint({ children }: { children: ReactNode }) {
  return (
    <Paragraph type="tertiary" size="small" style={{ marginTop: 24 }}>
      {children}
    </Paragraph>
  );
}
