"use client";

import { Loader2 } from "lucide-react";

import { Button } from "@supportflow/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle
} from "@supportflow/ui/dialog";

import type { WeworkSavedAccount } from "@/features/wework/accounts/types";

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
    <Dialog open={switchTarget !== null} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>切换连接账号</DialogTitle>
          <DialogDescription>
            {`当前账号：${activeAccountLabel ?? "未连接"}，切换后将连接到 ${switchTarget?.label ?? "目标账号"}。`}
          </DialogDescription>
        </DialogHeader>
        <DialogFooter className="gap-2 sm:gap-0">
          <Button
            type="button"
            variant="outline"
            disabled={switching}
            onClick={() => onOpenChange(false)}
          >
            取消
          </Button>
          <Button
            type="button"
            disabled={switching}
            className="bg-[var(--wework-blue)] text-white hover:opacity-90"
            onClick={onConfirm}
          >
            {switching ? (
              <>
                <Loader2 className="size-4 animate-spin" />
                正在连接
              </>
            ) : (
              "确认切换"
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
