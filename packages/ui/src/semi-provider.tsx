"use client";

import { LocaleProvider } from "@douyinfe/semi-ui-19";
import zh_CN from "@douyinfe/semi-ui-19/lib/es/locale/source/zh_CN";
import type { ReactNode } from "react";

type SemiProviderProps = {
  children: ReactNode;
};

/** Semi Design 根配置（飞书 DSM 官方主题由 @douyinfe/semi-vite-plugin 在构建时注入）。 */
export function SemiProvider({ children }: SemiProviderProps) {
  return <LocaleProvider locale={zh_CN}>{children}</LocaleProvider>;
}
