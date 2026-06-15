"use client";

import { IconCopy } from "@douyinfe/semi-icons";
import { Button, Card, Input, Space, Spin, Typography } from "@douyinfe/semi-ui-19";
import { useCallback, useState } from "react";

import { AppRoute } from "@supportflow/shared/tauri-bridge/enums";
import { formatLicenseInvalidReason } from "@supportflow/shared/tauri-bridge/license-error";

import { useLicenseGate } from "./license-gate-provider";

const { Title, Text, Paragraph } = Typography;

/** 订阅状态说明页（可正常路由进入；操作由 {@link LicenseLockOverlay} 上的锁触发）。 */
export function LicenseLockedPage() {
  const { loading, status, reasonLabel } = useLicenseGate();
  const [machineCopied, setMachineCopied] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const machineCode = status?.machineCode ?? "";
  const headline = formatLicenseInvalidReason(reasonLabel);

  const handleCopyMachineCode = useCallback(async () => {
    if (!machineCode) {
      return;
    }
    setMachineCopied(false);
    setError(null);
    try {
      if (typeof window === "undefined" || !navigator?.clipboard?.writeText) {
        setError("无法访问剪贴板");
        return;
      }
      await navigator.clipboard.writeText(machineCode);
      setMachineCopied(true);
      window.setTimeout(() => setMachineCopied(false), 2000);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(msg);
    }
  }, [machineCode]);

  if (loading) {
    return (
      <div style={{ display: "flex", flex: 1, alignItems: "center", justifyContent: "center" }}>
        <Spin size="large" />
      </div>
    );
  }

  return (
    <div
      style={{
        display: "flex",
        flex: 1,
        alignItems: "center",
        justifyContent: "center",
        overflowY: "auto",
        padding: "40px 24px"
      }}
    >
      <Card style={{ width: "100%", maxWidth: 512 }} bodyStyle={{ padding: 32 }} shadows="hover">
        <Space vertical align="center" spacing="medium" style={{ width: "100%" }}>
          <Title heading={3} style={{ margin: 0, textAlign: "center" }}>
            {headline}
          </Title>
          <Paragraph type="tertiary" style={{ textAlign: "center", margin: 0 }}>
            当前订阅无法使用完整功能。点击页面中央的锁图标可导入激活文件；也可复制机器码联系管理员续费。
          </Paragraph>

          <Card style={{ width: "100%" }} bodyStyle={{ padding: "12px 16px", textAlign: "center" }}>
            <Text type="tertiary" size="small" strong style={{ display: "block", marginBottom: 4 }}>
              状态详情
            </Text>
            <Text>{reasonLabel || "订阅未通过校验"}</Text>
          </Card>

          <Space vertical align="start" spacing="tight" style={{ width: "100%" }}>
            <Text strong>机器码</Text>
            <Space style={{ width: "100%" }}>
              <Input
                style={{ flex: 1, fontFamily: "monospace", fontSize: 12 }}
                value={machineCode}
                readonly
              />
              <Button
                icon={<IconCopy />}
                theme="light"
                disabled={!machineCode}
                onClick={() => void handleCopyMachineCode()}
              >
                {machineCopied ? "已复制" : "复制"}
              </Button>
            </Space>
          </Space>

          {error ? (
            <Text type="danger" role="alert">
              {error}
            </Text>
          ) : null}
        </Space>
      </Card>
    </div>
  );
}

/** 路由 path 段，与 {@link AppRoute.License} 一致。 */
export const LICENSE_ROUTE_PATH = AppRoute.License;
