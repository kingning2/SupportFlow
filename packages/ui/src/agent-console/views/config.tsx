"use client";

import { ViewShell } from "../shared/console-brand";
import type { AgentConsoleState } from "@supportflow/shared/contracts";

export function Config({ state }: { state: AgentConsoleState | null }) {
  return (
    <ViewShell
      title={"运行配置"}
      description={"工作区路径、采样参数与 MCP；模型厂商请在「模型」页查看。"}
    >
      <div className="grid gap-6 lg:grid-cols-2">
        <section className="rounded-xl border border-slate-200 p-4 dark:border-white/10">
          <h3 className="mb-3 text-sm font-semibold">{"路径"}</h3>
          <dl className="space-y-2 text-sm">
            <div>
              <dt className="text-muted-foreground">{"工作区"}</dt>
              <dd className="font-mono text-xs break-all">{state?.workspaceDir ?? "—"}</dd>
            </div>
            <div>
              <dt className="text-muted-foreground">{"配置源 (resources)"}</dt>
              <dd className="font-mono text-xs break-all">{state?.configPath ?? "—"}</dd>
            </div>
          </dl>
        </section>

        <section className="rounded-xl border border-slate-200 p-4 dark:border-white/10">
          <h3 className="mb-3 text-sm font-semibold">{"采样参数"}</h3>
          <dl className="grid grid-cols-2 gap-3 text-sm">
            <div>
              <dt className="text-muted-foreground">{"temperature"}</dt>
              <dd>{state?.temperature ?? "默认"}</dd>
            </div>
            <div>
              <dt className="text-muted-foreground">top_p</dt>
              <dd>{state?.topP ?? "默认"}</dd>
            </div>
            <div>
              <dt className="text-muted-foreground">{"请求超时"}</dt>
              <dd>{state?.requestTimeout ?? "默认"}</dd>
            </div>
          </dl>
        </section>
      </div>

      {state?.mcpStatus && Object.keys(state.mcpStatus).length > 0 ? (
        <section className="mt-6 rounded-xl border border-slate-200 p-4 dark:border-white/10">
          <h3 className="mb-3 text-sm font-semibold">MCP</h3>
          <ul className="space-y-1 text-sm">
            {Object.entries(state.mcpStatus).map(([name, status]) => (
              <li key={name} className="flex justify-between gap-4">
                <span className="font-mono text-xs">{name}</span>
                <span className="text-muted-foreground">{status}</span>
              </li>
            ))}
          </ul>
        </section>
      ) : null}

      <p className="text-muted-foreground mt-6 text-xs">
        {"配置源文件：src-tauri/resources/config.json（随 Tauri 打包）。修改后需重启应用。"}
      </p>
    </ViewShell>
  );
}
