import { invokeWrapper } from "./invoke";
import { TauriCmd } from "../enums";
import type {
  AgentClearProviderRequest,
  AgentConsoleState,
  AgentSendMessageRequest,
  AgentSendMessageResponse,
  AgentSetChatModelRequest,
  AgentUpdateProviderRequest
} from "@supportflow/shared/contracts";

export type { AgentConsoleState, AgentSendMessageRequest, AgentSendMessageResponse };

export function getAgentConsoleState() {
  return invokeWrapper<AgentConsoleState>(TauriCmd.AgentGetConsoleState);
}

export function sendAgentMessage(body: AgentSendMessageRequest) {
  return invokeWrapper<AgentSendMessageResponse>(TauriCmd.AgentSendMessage, { body });
}

export function cancelAgentMessage(requestId: string) {
  return invokeWrapper<void>(TauriCmd.AgentCancel, { body: { requestId } });
}

export function clearAgentContext() {
  return invokeWrapper<void>(TauriCmd.AgentClearContext);
}

export function newAgentSession() {
  return invokeWrapper<string>(TauriCmd.AgentNewSession);
}

export function refreshAgentSkills() {
  return invokeWrapper<AgentConsoleState["skills"]>(TauriCmd.AgentRefreshSkills);
}

export function updateAgentProvider(body: AgentUpdateProviderRequest) {
  return invokeWrapper<void>(TauriCmd.AgentUpdateProvider, { body });
}

export function clearAgentProvider(body: AgentClearProviderRequest) {
  return invokeWrapper<void>(TauriCmd.AgentClearProvider, { body });
}

export function setAgentChatModel(body: AgentSetChatModelRequest) {
  return invokeWrapper<void>(TauriCmd.AgentSetChatModel, { body });
}

export interface AgentSessionSummary {
  id: string;
  title: string;
  updatedAt: string;
}

export interface AgentMemoryItem {
  filename: string;
  type: string;
  size: number;
  updatedAt: string;
}

export interface AgentMemoryReadResult {
  filename: string;
  content: string;
}

export interface AgentKnowledgeFile {
  path: string;
  title: string;
}

export interface AgentKnowledgeGraphNode {
  id: string;
  label: string;
  category: string;
}

export interface AgentKnowledgeGraphLink {
  source: string;
  target: string;
}

export interface AgentChannelSummary {
  name: string;
  active: boolean;
  label: string;
}

export interface AgentChannelField {
  key: string;
  label: string;
  type: string;
  value: string;
  defaultValue?: string;
  placeholder?: string;
}

export interface AgentChannelDetail {
  name: string;
  labelKey: string;
  active: boolean;
  fields: AgentChannelField[];
  hintKey?: string;
}

export interface AgentChannelActionRequest {
  action: "connect" | "disconnect" | "save";
  channel: string;
  config?: Record<string, string | number | boolean>;
}

export interface AgentChannelActionResponse {
  channelType: string;
}

export interface AgentTaskSummary {
  id: string;
  name: string;
  enabled: boolean;
  nextRunAt?: string;
}

export interface AgentLogsStatus {
  enabled: boolean;
  source: string;
}

export interface AgentReadLogsRequest {
  limit?: number;
}

export interface AgentReadLogsResult {
  source: string;
  content: string;
}

export function listAgentSessions() {
  return invokeWrapper<AgentSessionSummary[]>(TauriCmd.AgentListSessions);
}

export function listAgentMemory() {
  return invokeWrapper<AgentMemoryItem[]>(TauriCmd.AgentListMemory);
}

export function readAgentMemory(filename: string) {
  return invokeWrapper<AgentMemoryReadResult>(TauriCmd.AgentReadMemory, {
    body: { filename }
  });
}

export function listAgentKnowledge() {
  return invokeWrapper<AgentKnowledgeFile[]>(TauriCmd.AgentListKnowledge);
}

export function readAgentKnowledge(path: string) {
  return invokeWrapper<{ path: string; content: string }>(TauriCmd.AgentReadKnowledge, {
    body: { path }
  });
}

export function getAgentKnowledgeGraph() {
  return invokeWrapper<{ nodes: AgentKnowledgeGraphNode[]; links: AgentKnowledgeGraphLink[] }>(
    TauriCmd.AgentGetKnowledgeGraph
  );
}

export interface AgentKnowledgeUploadFile {
  filename: string;
  data: number[];
}

export interface AgentKnowledgeUploadResult {
  results: Array<{
    path: string;
    title: string;
    category: string;
    slug: string;
    originalName: string;
    truncated: boolean;
    charCount: number;
    archive: string;
  }>;
  errors: Array<{ file: string; message: string }>;
  count: number;
  memorySynced: boolean;
}

export function uploadAgentKnowledge(files: AgentKnowledgeUploadFile[], category?: string) {
  return invokeWrapper<AgentKnowledgeUploadResult>(TauriCmd.AgentUploadKnowledge, {
    body: { files, category }
  });
}

export function listAgentChannels() {
  return invokeWrapper<AgentChannelSummary[]>(TauriCmd.AgentListChannels);
}

/** @deprecated Use `fetchChannels` from `@/cmd/channel-python-channels` (Python proxy). */
export function getAgentChannelCatalog() {
  return invokeWrapper<{ status: string; channels: AgentChannelDetail[] }>(
    TauriCmd.AgentGetChannelCatalog
  );
}

/** @deprecated Use `channelAction` from `@/cmd/channel-python-channels` (Python proxy). */
export function agentChannelAction(body: AgentChannelActionRequest) {
  return invokeWrapper<AgentChannelActionResponse & { status: string }>(
    TauriCmd.AgentChannelAction,
    { body }
  );
}

export function listAgentTasks() {
  return invokeWrapper<AgentTaskSummary[]>(TauriCmd.AgentListTasks);
}

export function getAgentLogsStatus() {
  return invokeWrapper<AgentLogsStatus>(TauriCmd.AgentGetLogsStatus);
}

export function readAgentLogs(body?: AgentReadLogsRequest) {
  return invokeWrapper<AgentReadLogsResult>(TauriCmd.AgentReadLogs, { body });
}

export function startAgentLogStream() {
  return invokeWrapper<void>(TauriCmd.AgentStartLogStream);
}

export function stopAgentLogStream() {
  return invokeWrapper<void>(TauriCmd.AgentStopLogStream);
}
