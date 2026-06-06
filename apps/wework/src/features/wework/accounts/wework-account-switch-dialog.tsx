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

type Translate = (key: string, options?: Record<string, unknown>) => string;

interface WeworkAccountSwitchDialogProps {
  activeAccountLabel?: string;
  onConfirm: () => void;
  onOpenChange: (open: boolean) => void;
  switching: boolean;
  switchTarget: WeworkSavedAccount | null;
  t: Translate;
}

export function WeworkAccountSwitchDialog({
  activeAccountLabel,
  onConfirm,
  onOpenChange,
  switching,
  switchTarget,
  t
}: WeworkAccountSwitchDialogProps) {
  return (
    <Dialog open={switchTarget !== null} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{t("wework_account_switch_title")}</DialogTitle>
          <DialogDescription>
            {t("wework_account_switch_message", {
              current: activeAccountLabel ?? "鈥?",
              target: switchTarget?.label ?? "鈥?"
            })}
          </DialogDescription>
        </DialogHeader>
        <DialogFooter className="gap-2 sm:gap-0">
          <Button
            type="button"
            variant="outline"
            disabled={switching}
            onClick={() => onOpenChange(false)}
          >
            {t("wework_account_switch_cancel")}
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
                {t("channels_connecting")}
              </>
            ) : (
              t("wework_account_switch_confirm")
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
