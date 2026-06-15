"use client";

import { IconLock } from "@douyinfe/semi-icons";
import { Button, Space, Typography } from "@douyinfe/semi-ui-19";
import { useState, type ReactNode } from "react";

import { cn } from "@supportflow/shared";
import { formatLicenseInvalidReason } from "@supportflow/shared/tauri-bridge/license-error";
import { LicenseActivationModal } from "@supportflow/ui/title-bar";

import { useLicenseGate } from "./license-gate-provider";

const { Text, Title } = Typography;

type LicenseLockOverlayProps = {
  children: ReactNode;
  className?: string;
  /** 为 false 时不展示锁（即使订阅无效），用于仅锁特定路由 */
  enabled?: boolean;
};

const SHELL_CLASS =
  "license-lock-shell relative flex h-full min-h-0 w-full min-w-0 flex-1 flex-col self-stretch overflow-hidden";

/**
 * 订阅无效时：局部半透明遮罩 + 居中锁，点击锁打开激活弹窗。
 * 由壳层按路由控制 {@link enabled}，例如仅「账号与通道」页启用。
 */
export function LicenseLockOverlay({
  children,
  className,
  enabled = true
}: LicenseLockOverlayProps) {
  const { loading, valid, reasonLabel } = useLicenseGate();
  const [activationOpen, setActivationOpen] = useState(false);

  const locked = !loading && !valid && enabled;
  const headline = formatLicenseInvalidReason(reasonLabel);

  return (
    <div className={cn(SHELL_CLASS, className)}>
      <div
        style={{
          display: "flex",
          minHeight: 0,
          flex: 1,
          flexDirection: "column",
          overflow: "hidden",
          opacity: locked ? 0.45 : 1,
          pointerEvents: locked ? "none" : undefined,
          userSelect: locked ? "none" : undefined
        }}
      >
        {children}
      </div>

      {locked ? (
        <div role="presentation" className="license-lock-overlay">
          <Button
            theme="light"
            type="tertiary"
            className="license-lock-trigger"
            onClick={() => setActivationOpen(true)}
            aria-label={`${headline}，点击激活订阅`}
          >
            <Space vertical align="center" spacing="medium">
              <span className="license-lock-trigger__icon">
                <IconLock size="extra-large" />
              </span>
              <Title heading={5} className="license-lock-trigger__title">
                {headline}
              </Title>
              <Text type="tertiary" className="license-lock-trigger__hint">
                点击锁图标激活订阅
              </Text>
            </Space>
          </Button>
        </div>
      ) : null}

      {locked ? (
        <LicenseActivationModal open={activationOpen} onOpenChange={setActivationOpen} />
      ) : null}
    </div>
  );
}
