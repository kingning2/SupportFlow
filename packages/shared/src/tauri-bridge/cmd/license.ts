import { TauriCmd } from "../enums";
import { invokeWrapper } from "./invoke";

export type LicenseStatusDto = {
  machineCode: string;
  valid: boolean;
  reason?: string | null;
};

/** License snapshot from startup (machine code + validity). */
export const getLicenseStatus = () => invokeWrapper<LicenseStatusDto>(TauriCmd.LicenseGetStatus);

/** Verify activation token, persist locally, refresh license status. */
export const applyLicenseActivation = (token: string) =>
  invokeWrapper<LicenseStatusDto>(TauriCmd.LicenseApplyActivation, { token });
