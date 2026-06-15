"use client";

import { IconComment } from "@douyinfe/semi-icons";
import { Avatar, Card, Empty, Typography } from "@douyinfe/semi-ui-19";

const { Text } = Typography;

export function __CHANNEL_PAGE_COMPONENT__() {
  return (
    <div
      style={{
        display: "flex",
        height: "100%",
        minHeight: 0,
        flexDirection: "column",
        overflow: "hidden"
      }}
    >
      <div style={{ flex: 1, minHeight: 0, overflowY: "auto", padding: 24 }}>
        <Empty
          style={{ maxWidth: 512, margin: "80px auto" }}
          image={
            <Avatar
              size="large"
              style={{
                background: "var(--semi-color-primary-light-default)",
                color: "var(--semi-color-primary)"
              }}
            >
              <IconComment size="extra-large" />
            </Avatar>
          }
          title="__CHANNEL_LABEL__"
          description="Platform scaffold created."
        >
          <Card
            style={{ marginTop: 16, borderStyle: "dashed" }}
            bodyStyle={{ padding: "12px 16px" }}
          >
            <Text type="tertiary" size="small" style={{ display: "block", textAlign: "center" }}>
              Start implementation in `src/features/__CHANNEL_SLUG__`.
            </Text>
          </Card>
        </Empty>
      </div>
    </div>
  );
}
