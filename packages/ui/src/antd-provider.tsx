"use client";

import { ConfigProvider, theme } from "antd";
import type { ReactNode } from "react";

type AntdProviderProps = {
  children: ReactNode;
  /** When true, use antd dark algorithm (matches `.dark` shell). */
  dark?: boolean;
};

/** Root ConfigProvider for antd components across desktop apps. */
export function AntdProvider({ children, dark = false }: AntdProviderProps) {
  return (
    <ConfigProvider
      theme={{
        algorithm: dark ? theme.darkAlgorithm : theme.defaultAlgorithm,
        token: {
          borderRadius: 6,
          fontFamily: "inherit"
        }
      }}
    >
      {children}
    </ConfigProvider>
  );
}
