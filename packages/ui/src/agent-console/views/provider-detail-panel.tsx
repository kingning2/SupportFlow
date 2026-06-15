"use client";

import { useEffect, useMemo, useState } from "react";
import { Avatar, Button, Input, Modal, Select, Space, Tag, Typography } from "@douyinfe/semi-ui-19";
import { IconArrowLeft, IconSave } from "@douyinfe/semi-icons";

import {
  clearAgentProvider,
  setAgentChatModel,
  updateAgentProvider
} from "@supportflow/shared/tauri-bridge/cmd/agent";
import type { AgentConsoleState, ModelProviderDetail } from "@supportflow/shared/contracts";

import { providerLabel } from "../lib/agent-console/provider-labels";
import { MutedHint } from "../shared/console-brand";

const { Text, Title } = Typography;

function shouldUseCustomModel(modelOptions: string[], modelValue: string) {
  return (
    modelValue === "__custom__" ||
    (modelValue === "" && modelOptions.length === 0) ||
    (modelOptions.length > 0 && !modelOptions.includes(modelValue))
  );
}

interface ProviderDetailPanelProps {
  provider: ModelProviderDetail | null;
  pickableProviders: ModelProviderDetail[];
  state: AgentConsoleState | null;
  onBack: () => void;
  onSaved: () => void | Promise<void>;
}

export function ProviderDetailPanel({
  provider,
  pickableProviders,
  state,
  onBack,
  onSaved
}: ProviderDetailPanelProps) {
  const isAddMode = !provider;
  const [selectedId, setSelectedId] = useState(provider?.id ?? "");
  const [apiKey, setApiKey] = useState("");
  const [apiBaseDraft, setApiBaseDraft] = useState("");
  const [chatModel, setChatModel] = useState("");
  const [customModel, setCustomModel] = useState("");
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const active = provider ?? pickableProviders.find((item) => item.id === selectedId) ?? null;
  const modelOptions = active?.models ?? [];
  const resolvedModel =
    chatModel || (active?.isActive ? state?.modelName : "") || modelOptions[0] || "";
  const resolvedCustomModel = customModel || resolvedModel;
  const useCustomModel = shouldUseCustomModel(modelOptions, resolvedModel);

  const resolvedApiBase = useMemo(() => {
    if (!active) return "";
    return apiBaseDraft || active.apiBase || active.apiBaseDefault || "";
  }, [active, apiBaseDraft]);

  useEffect(() => {
    if (isAddMode && !selectedId && pickableProviders.length > 0) {
      setSelectedId(pickableProviders[0].id);
    }
  }, [isAddMode, pickableProviders, selectedId]);

  useEffect(() => {
    if (!isAddMode || !selectedId) {
      return;
    }
    const picked = pickableProviders.find((item) => item.id === selectedId);
    if (!picked) {
      return;
    }
    setApiBaseDraft(picked.apiBase ?? picked.apiBaseDefault ?? "");
    const initialModel = picked.models[0] ?? "";
    setChatModel(initialModel);
    setCustomModel(initialModel);
    setApiKey("");
    setError(null);
  }, [isAddMode, pickableProviders, selectedId]);

  useEffect(() => {
    if (!provider) {
      return;
    }
    setSelectedId(provider.id);
    setApiKey("");
    setApiBaseDraft(provider.apiBase ?? provider.apiBaseDefault ?? "");
    const initialModel =
      provider.isActive && state?.modelName ? state.modelName : (provider.models[0] ?? "");
    setChatModel(initialModel);
    setCustomModel(initialModel);
    setError(null);
  }, [provider, state?.modelName]);

  const handleModelChange = (value: string) => {
    setChatModel(value);
    if (value !== "__custom__") {
      setCustomModel(value);
    }
  };

  const handleSave = async () => {
    if (!active) return;

    const trimmedKey = apiKey.trim();
    const hasMasked = Boolean(active.apiKeyMasked);
    const isOllama = active.id.toLowerCase() === "ollama";
    if (!trimmedKey && !hasMasked && !isOllama) {
      setError("请填写 API Key，或确认该厂商已保存过凭据。");
      return;
    }

    const modelValue = (useCustomModel ? resolvedCustomModel : resolvedModel).trim();
    if (!modelValue) {
      setError("请填写或选择对话模型 ID。");
      return;
    }

    setSaving(true);
    setError(null);
    try {
      await updateAgentProvider({
        providerId: active.id,
        apiKey: trimmedKey || undefined,
        apiBase: active.hasApiBase ? resolvedApiBase.trim() : undefined,
        apiBaseSet: active.hasApiBase
      });
      await setAgentChatModel({ providerId: active.id, model: modelValue });
      await onSaved();
      onBack();
    } catch (err) {
      setError(err instanceof Error ? err.message : "保存失败，请检查 Key 或网络权限。");
    } finally {
      setSaving(false);
    }
  };

  const handleClear = () => {
    if (!active) return;
    Modal.confirm({
      title: "清除厂商凭据？",
      content: "这会删除当前厂商的 API Key 和 Base URL。",
      okType: "danger",
      onOk: async () => {
        setSaving(true);
        setError(null);
        try {
          await clearAgentProvider({ providerId: active.id });
          await onSaved();
          onBack();
        } catch (err) {
          setError(err instanceof Error ? err.message : "清除失败，请稍后再试。");
        } finally {
          setSaving(false);
        }
      }
    });
  };

  const displayName = active ? providerLabel(active.id) : "添加供应商";

  return (
    <div className="provider-settings-detail">
      <div className="provider-settings-toolbar">
        <Button
          icon={<IconArrowLeft />}
          theme="light"
          type="tertiary"
          disabled={saving}
          onClick={onBack}
        >
          返回列表
        </Button>
        <Button
          type="primary"
          icon={<IconSave />}
          loading={saving}
          disabled={!active}
          onClick={() => void handleSave()}
        >
          保存
        </Button>
      </div>

      <div className="provider-settings-detail__hero">
        <Space align="center">
          <Avatar size="large" style={{ background: "var(--semi-color-primary-light-default)" }}>
            {displayName.slice(0, 1)}
          </Avatar>
          <div>
            <Space align="center" spacing={8}>
              <Title heading={4} style={{ margin: 0 }}>
                {displayName}
              </Title>
              {active?.isActive ? (
                <Tag color="blue" size="small">
                  使用中
                </Tag>
              ) : null}
            </Space>
            <Text type="tertiary" size="small">
              配置 API Key、Base URL 与对话模型；保存后将设为当前对话供应商。
            </Text>
          </div>
        </Space>
      </div>

      <div className="provider-settings-detail__form">
        <Space vertical style={{ width: "100%" }} spacing="loose">
          {isAddMode ? (
            <div className="provider-settings-field">
              <Text strong className="provider-settings-field__label">
                选择厂商
              </Text>
              <Select
                value={selectedId}
                style={{ width: "100%" }}
                placeholder="选择要配置的厂商"
                onChange={(value) => setSelectedId(String(value))}
              >
                {pickableProviders.map((item) => (
                  <Select.Option key={item.id} value={item.id}>
                    {providerLabel(item.id)}
                  </Select.Option>
                ))}
              </Select>
            </div>
          ) : (
            <div className="provider-settings-field">
              <Text strong className="provider-settings-field__label">
                厂商 ID
              </Text>
              <Input value={active?.id ?? ""} disabled style={{ fontFamily: "monospace" }} />
            </div>
          )}

          <div className="provider-settings-field">
            <Text strong className="provider-settings-field__label">
              对话模型
            </Text>
            {modelOptions.length > 0 ? (
              <>
                <Select
                  value={useCustomModel ? "__custom__" : resolvedModel}
                  style={{ width: "100%" }}
                  onChange={(value) => handleModelChange(String(value))}
                >
                  {modelOptions.map((model) => (
                    <Select.Option key={model} value={model}>
                      {model}
                    </Select.Option>
                  ))}
                  <Select.Option value="__custom__">自定义模型名</Select.Option>
                </Select>
                {useCustomModel ? (
                  <Input
                    style={{ width: "100%", marginTop: 8, fontFamily: "monospace" }}
                    placeholder="自定义模型名"
                    value={resolvedCustomModel}
                    onChange={setCustomModel}
                  />
                ) : null}
              </>
            ) : (
              <Input
                style={{ width: "100%", fontFamily: "monospace" }}
                placeholder="例如 gpt-4o、deepseek-chat"
                value={resolvedCustomModel}
                onChange={setCustomModel}
              />
            )}
          </div>

          <div className="provider-settings-field">
            <Text strong className="provider-settings-field__label">
              {active?.id.toLowerCase() === "ollama" ? "API Key（可选）" : "API Key"}
            </Text>
            <Input
              mode="password"
              autoComplete="off"
              placeholder={
                active?.id.toLowerCase() === "ollama"
                  ? "本地 Ollama 通常无需 Key，可留空"
                  : active?.apiKeyMasked
                    ? active.apiKeyMasked
                    : "留空表示不修改已有 Key"
              }
              value={apiKey}
              onChange={(value) => setApiKey(String(value))}
            />
          </div>

          {active?.hasApiBase ? (
            <div className="provider-settings-field">
              <Text strong className="provider-settings-field__label">
                Base URL
              </Text>
              <Input
                style={{ fontFamily: "monospace" }}
                placeholder={active.apiBaseDefault ?? ""}
                value={resolvedApiBase}
                onChange={(value) => setApiBaseDraft(String(value))}
              />
            </div>
          ) : null}

          {error ? <Text type="danger">{error}</Text> : null}

          <Button
            type="danger"
            theme="light"
            disabled={saving || !active?.configured}
            onClick={handleClear}
          >
            清除凭据
          </Button>

          <MutedHint>凭据写入 `src-tauri/resources/config.json`，保存后立即生效。</MutedHint>
        </Space>
      </div>
    </div>
  );
}
