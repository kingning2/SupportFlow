"use client";

import { Empty } from "@douyinfe/semi-ui-19";

export function ConfigPlaceholder({ title }: { title: string }) {
  return (
    <Empty
      style={{ margin: "auto", padding: 24 }}
      title={title}
      description="该页面后续会接入相关配置能力，当前先保留占位。"
    />
  );
}
