"use client";

import { useMemo, useState } from "react";
import { Avatar, Button, Collapse, Space, Tag, Typography } from "@douyinfe/semi-ui-19";
import { IconChevronRight, IconPlus } from "@douyinfe/semi-icons";

import { getAgentConsoleState } from "@supportflow/shared/tauri-bridge/cmd/agent";
import type { AgentConsoleState, ModelProviderDetail } from "@supportflow/shared/contracts";

import { providerLabel } from "../lib/agent-console/provider-labels";
import { MutedHint, SectionCard, ViewShell } from "../shared/console-brand";
import { RuntimeConfigPanel } from "./config";
import { ProviderDetailPanel } from "./provider-detail-panel";

const { Text } = Typography;

type ProviderSettingsView = "list" | "detail" | "add";

interface ProviderSettingsProps {
  state: AgentConsoleState | null;
  onRefresh: (next: AgentConsoleState | null) => void;
  showRuntimePanel?: boolean;
  /** 嵌入企微等已有页头时，不再重复 ViewShell 标题 */
  embedded?: boolean;
}

function providerStatusTag(provider: ModelProviderDetail) {
  if (provider.id.toLowerCase() === "ollama") {
    return provider.configured ? "本地服务" : "未配置地址";
  }
  return provider.configured ? "已配置 API Key" : "未配置";
}

function providerMetaLine(provider: ModelProviderDetail, state: AgentConsoleState | null) {
  const base = provider.apiBase || provider.apiBaseDefault || "默认端点";
  if (provider.id.toLowerCase() === "ollama") {
    return `本地服务 · ${base}`;
  }
  const modelPart =
    provider.isActive && state?.modelName
      ? state.modelName
      : provider.configured
        ? "纯 API"
        : "待配置";
  return `${modelPart} · ${base}`;
}

export function ProviderSettings({
  state,
  onRefresh,
  showRuntimePanel = true,
  embedded = false
}: ProviderSettingsProps) {
  const details = state?.providerDetails ?? [];
  const editableProviders = useMemo(
    () => details.filter((provider) => provider.editable),
    [details]
  );
  const [view, setView] = useState<ProviderSettingsView>("list");
  const [selectedProvider, setSelectedProvider] = useState<ModelProviderDetail | null>(null);

  const reloadState = async () => {
    const next = await getAgentConsoleState();
    onRefresh(next);
  };

  const openDetail = (provider: ModelProviderDetail) => {
    setSelectedProvider(provider);
    setView("detail");
  };

  const openAdd = () => {
    setSelectedProvider(null);
    setView("add");
  };

  const backToList = () => {
    setView("list");
    setSelectedProvider(null);
  };

  const shellTitle = embedded ? undefined : "供应商配置";
  const shellDescription = embedded ? undefined : "管理 API 供应商、Key 与对话模型";

  if (view === "detail" || view === "add") {
    return (
      <ViewShell className="agent-console-interactive agent-console-page-enter provider-settings">
        <ProviderDetailPanel
          provider={view === "add" ? null : selectedProvider}
          pickableProviders={editableProviders}
          state={state}
          onBack={backToList}
          onSaved={reloadState}
        />
      </ViewShell>
    );
  }

  const configuredCount = details.filter((provider) => provider.configured).length;

  return (
    <ViewShell
      className="agent-console-interactive agent-console-page-enter provider-settings"
      title={shellTitle}
      description={shellDescription}
    >
      <SectionCard className="provider-settings-list-card">
        <div className="provider-settings-list-header">
          <Text type="tertiary" size="small" className="provider-settings-list-header__meta">
            {details.length} 个供应商 · 已配置 {configuredCount}/{details.length}
            {state?.botType
              ? ` · 当前对话 ${providerLabel(state.botType)} / ${state.modelName}`
              : ""}
          </Text>
          <Button icon={<IconPlus />} theme="light" type="tertiary" onClick={openAdd}>
            添加供应商
          </Button>
        </div>

        <Space vertical spacing={10} style={{ width: "100%" }}>
          {details.map((provider) => {
            const editable = provider.editable;
            return (
              <div
                key={provider.id}
                className={`provider-settings-card${editable ? "provider-settings-card--editable" : ""}`}
                role={editable ? "button" : undefined}
                tabIndex={editable ? 0 : undefined}
                onClick={() => {
                  if (editable) {
                    openDetail(provider);
                  }
                }}
                onKeyDown={(event) => {
                  if (!editable) return;
                  if (event.key === "Enter" || event.key === " ") {
                    event.preventDefault();
                    openDetail(provider);
                  }
                }}
              >
                <Avatar
                  size="small"
                  style={{
                    background: provider.isActive
                      ? "var(--semi-color-primary-light-default)"
                      : "var(--semi-color-fill-1)",
                    color: provider.isActive
                      ? "var(--semi-color-primary)"
                      : "var(--semi-color-text-2)"
                  }}
                >
                  {providerLabel(provider.id).slice(0, 1)}
                </Avatar>

                <div className="provider-settings-card__main">
                  <Text strong>{providerLabel(provider.id)}</Text>
                  <Text type="tertiary" size="small" className="provider-settings-card__meta">
                    {providerMetaLine(provider, state)}
                  </Text>
                </div>

                <Space>
                  <Tag color={provider.configured ? "green" : "grey"} size="small">
                    {providerStatusTag(provider)}
                  </Tag>
                  {provider.isActive ? (
                    <Tag color="blue" size="small">
                      使用中
                    </Tag>
                  ) : null}
                  {editable ? (
                    <IconChevronRight className="provider-settings-card__chevron" />
                  ) : null}
                </Space>
              </div>
            );
          })}
        </Space>
      </SectionCard>

      {showRuntimePanel ? (
        <Collapse
          className="provider-settings-runtime"
          defaultActiveKey={[]}
          style={{ marginTop: 16 }}
        >
          <Collapse.Panel header="运行参数" itemKey="runtime">
            <RuntimeConfigPanel state={state} />
          </Collapse.Panel>
        </Collapse>
      ) : null}

      <MutedHint>配置源：`src-tauri/resources/config.json`。修改运行参数后需重启应用。</MutedHint>
    </ViewShell>
  );
}
