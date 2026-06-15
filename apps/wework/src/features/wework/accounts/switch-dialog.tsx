"use client";

import { Modal, Typography } from "@douyinfe/semi-ui-19";

import type { WeworkSavedAccount } from "@/features/wework/accounts/types";

const { Text } = Typography;

interface AccountSwitchDialogProps {
  activeAccountLabel?: string;
  onConfirm: () => void;
  onOpenChange: (open: boolean) => void;
  switching: boolean;
  switchTarget: WeworkSavedAccount | null;
}

export function AccountSwitchDialog({
  activeAccountLabel,
  onConfirm,
  onOpenChange,
  switching,
  switchTarget
}: AccountSwitchDialogProps) {
  return (
    <Modal
      visible={switchTarget !== null}
      title="切换连接账号"
      width={448}
      onCancel={() => onOpenChange(false)}
      onOk={onConfirm}
      confirmLoading={switching}
      okText={switching ? "正在连接" : "确认切换"}
      cancelText="取消"
      maskClosable={!switching}
      closable={!switching}
    >
      <Text>
        {`当前账号：${activeAccountLabel ?? "未连接"}，切换后将连接到 ${switchTarget?.label ?? "目标账号"}。`}
      </Text>
    </Modal>
  );
}
