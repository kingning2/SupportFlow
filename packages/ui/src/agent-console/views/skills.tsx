"use client";

import { useState } from "react";
import {
  Button,
  Card,
  Col,
  Empty,
  Input,
  List,
  Modal,
  Row,
  Space,
  Spin,
  Tag,
  Typography
} from "@douyinfe/semi-ui-19";
import {
  IconDownload,
  IconEyeOpened,
  IconFolder,
  IconRefresh,
  IconFile
} from "@douyinfe/semi-icons";

import {
  getAgentSkillDetail,
  installAgentSkill,
  refreshAgentSkills
} from "@supportflow/shared/tauri-bridge/cmd/agent";
import type { AgentConsoleState, SkillDetail } from "@supportflow/shared/contracts";

import { SectionCard, ViewShell } from "../shared/console-brand";

const { Text, Title } = Typography;

interface SkillsProps {
  state: AgentConsoleState | null;
  onRefresh: (next: AgentConsoleState | null) => void;
}

const COPY = {
  baseDirLabel: "基础目录",
  detailDesc: "查看技能来源、注册位置和当前状态。",
  detailLabel: "查看详情",
  detailLoadingText: "正在加载技能详情...",
  detailTitle: "技能详情",
  filePathLabel: "技能文件",
  installButtonLabel: "安装",
  installDesc: "支持 Skill Hub 名称、GitHub owner/repo、zip 链接和本地路径。",
  installPlaceholder: "例如：supportflow/notion-skill 或 https://example.com/skill.zip",
  installTitle: "安装外部技能",
  installSuccessPrefix: "安装成功：",
  modelDisabledLabel: "已禁用模型调用",
  modelEnabledLabel: "允许模型调用",
  sourceLabel: "来源"
} as const;

export function Skills({ state, onRefresh }: SkillsProps) {
  const [detailOpen, setDetailOpen] = useState(false);
  const [detailLoading, setDetailLoading] = useState(false);
  const [detailError, setDetailError] = useState<string | null>(null);
  const [selectedSkill, setSelectedSkill] = useState<SkillDetail | null>(null);
  const [installSource, setInstallSource] = useState("");
  const [installing, setInstalling] = useState(false);
  const [installError, setInstallError] = useState<string | null>(null);
  const [installSuccess, setInstallSuccess] = useState<string | null>(null);

  const refreshSkillsState = async () => {
    try {
      const skills = await refreshAgentSkills();
      if (state) {
        onRefresh({ ...state, skills });
      }
    } catch {
      // keep current state
    }
  };

  const handleOpenDetail = async (name: string) => {
    setDetailOpen(true);
    setDetailLoading(true);
    setDetailError(null);
    try {
      setSelectedSkill(await getAgentSkillDetail(name));
    } catch (error) {
      setSelectedSkill(null);
      setDetailError(error instanceof Error ? error.message : String(error));
    } finally {
      setDetailLoading(false);
    }
  };

  const handleInstall = async () => {
    const source = installSource.trim();
    if (!source) return;

    setInstalling(true);
    setInstallError(null);
    setInstallSuccess(null);
    try {
      const result = await installAgentSkill({ source });
      await refreshSkillsState();
      setInstallSuccess(
        `${COPY.installSuccessPrefix}${result.installed_names.join(", ") || source}`
      );
      setInstallSource("");
    } catch (error) {
      setInstallError(error instanceof Error ? error.message : String(error));
    } finally {
      setInstalling(false);
    }
  };

  const skills = state?.skills ?? [];
  const tools = state?.tools ?? [];

  return (
    <ViewShell
      title="技能与工具"
      description="查看当前已注册的工具与技能，并支持安装新的外部技能。"
      extra={
        <Button
          icon={<IconRefresh />}
          theme="light"
          type="tertiary"
          onClick={() => void refreshSkillsState()}
        >
          刷新
        </Button>
      }
    >
      <SectionCard title={COPY.installTitle} style={{ marginBottom: 24 }}>
        <Text type="tertiary" size="small" style={{ display: "block", marginBottom: 12 }}>
          {COPY.installDesc}
        </Text>
        <Space style={{ width: "100%" }}>
          <Input
            value={installSource}
            onChange={setInstallSource}
            placeholder={COPY.installPlaceholder}
            style={{ flex: 1, fontFamily: "monospace" }}
          />
          <Button
            icon={<IconDownload />}
            type="primary"
            loading={installing}
            disabled={installSource.trim().length === 0}
            onClick={() => void handleInstall()}
          >
            {COPY.installButtonLabel}
          </Button>
        </Space>
        {installError ? (
          <Text type="danger" size="small" style={{ display: "block", marginTop: 12 }}>
            {installError}
          </Text>
        ) : null}
        {installSuccess ? (
          <Text type="success" size="small" style={{ display: "block", marginTop: 12 }}>
            {installSuccess}
          </Text>
        ) : null}
      </SectionCard>

      <Row gutter={24}>
        <Col span={12}>
          <Title heading={6} style={{ marginBottom: 12 }}>
            技能
          </Title>
          <List
            split
            dataSource={skills}
            emptyContent={
              <Empty description="暂无技能。内置技能将自动加载，也可在工作区 skills/ 目录添加。" />
            }
            renderItem={(skill) => (
              <List.Item
                main={
                  <Space vertical align="start" spacing={4} style={{ width: "100%" }}>
                    <Space>
                      <Text strong>{skill.name}</Text>
                      <Tag color={skill.enabled ? "green" : "grey"} size="small">
                        {skill.enabled ? "已启用" : "已禁用"}
                      </Tag>
                    </Space>
                    <Text type="tertiary" size="small">
                      {skill.description}
                    </Text>
                    <Text type="tertiary" size="small" code ellipsis style={{ maxWidth: "100%" }}>
                      {skill.source === "builtin" ? "内置" : skill.source}
                    </Text>
                  </Space>
                }
                extra={
                  <Button
                    icon={<IconEyeOpened />}
                    theme="borderless"
                    type="tertiary"
                    size="small"
                    onClick={() => void handleOpenDetail(skill.name)}
                  >
                    {COPY.detailLabel}
                  </Button>
                }
              />
            )}
          />
        </Col>
        <Col span={12}>
          <Title heading={6} style={{ marginBottom: 12 }}>
            工具
          </Title>
          <List
            split
            dataSource={tools}
            emptyContent={<Empty description="暂无工具" />}
            renderItem={(tool) => (
              <List.Item
                main={
                  <Space vertical align="start" spacing={4}>
                    <Space>
                      <Text strong>{tool.label || tool.name}</Text>
                      {tool.label && tool.label !== tool.name ? (
                        <Tag size="small" color="grey">
                          {tool.name}
                        </Tag>
                      ) : null}
                      {tool.isMcp ? (
                        <Tag size="small" color="blue">
                          MCP
                        </Tag>
                      ) : null}
                    </Space>
                    <Text type="tertiary" size="small">
                      {tool.description}
                    </Text>
                  </Space>
                }
              />
            )}
          />
        </Col>
      </Row>

      <Modal
        visible={detailOpen}
        title={selectedSkill?.name ?? COPY.detailTitle}
        width={720}
        onCancel={() => {
          setDetailOpen(false);
          setSelectedSkill(null);
          setDetailError(null);
        }}
        footer={null}
      >
        {detailLoading ? <Spin tip={COPY.detailLoadingText} /> : null}
        {!detailLoading && detailError ? <Text type="danger">{detailError}</Text> : null}
        {!detailLoading && !detailError && selectedSkill ? (
          <Space vertical align="start" spacing="medium" style={{ width: "100%" }}>
            <Text type="tertiary">{selectedSkill.description || COPY.detailDesc}</Text>
            <Space wrap>
              <Tag color={selectedSkill.enabled ? "green" : "grey"}>
                {selectedSkill.enabled ? "已启用" : "已禁用"}
              </Tag>
              <Tag color="blue">
                {selectedSkill.disableModelInvocation
                  ? COPY.modelDisabledLabel
                  : COPY.modelEnabledLabel}
              </Tag>
            </Space>
            <Card
              title={
                <Space>
                  <IconFolder />
                  {COPY.baseDirLabel}
                </Space>
              }
            >
              <Text code style={{ wordBreak: "break-all" }}>
                {selectedSkill.baseDir}
              </Text>
            </Card>
            <Card
              title={
                <Space>
                  <IconFile />
                  {COPY.filePathLabel}
                </Space>
              }
            >
              <Text code style={{ wordBreak: "break-all" }}>
                {selectedSkill.filePath}
              </Text>
            </Card>
            <Card title={COPY.sourceLabel}>
              <Text code>{selectedSkill.source}</Text>
            </Card>
          </Space>
        ) : null}
      </Modal>
    </ViewShell>
  );
}
