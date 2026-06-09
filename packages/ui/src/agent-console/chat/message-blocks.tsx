"use client";

import { Reasoning, ReasoningContent, ReasoningTrigger } from "../ai-elements/reasoning";
import { Tool, ToolContent, ToolHeader, ToolInput, ToolOutput } from "../ai-elements/tool";
import { Message, MessageContent, MessageResponse } from "../ai-elements/message";
import { mapToolStepState } from "../lib/agent-console/map-tool-state";
import type { AssistantChatMessage } from "../types/agent-chat";

interface AssistantMessageBlockProps {
  message: AssistantChatMessage;
}

export function AssistantMessageBlock({ message }: AssistantMessageBlockProps) {
  return (
    <Message from="assistant">
      <MessageContent>
        {message.reasoning ? (
          <Reasoning
            defaultOpen={message.reasoningStreaming}
            isStreaming={message.reasoningStreaming}
          >
            <ReasoningTrigger />
            <ReasoningContent>{message.reasoning}</ReasoningContent>
          </Reasoning>
        ) : null}

        {message.toolSteps.map((step) => (
          <Tool key={step.id} defaultOpen={step.status === "running"}>
            <ToolHeader
              type="dynamic-tool"
              state={mapToolStepState(step.status)}
              toolName={step.tool}
              title={step.tool}
            />
            <ToolContent>
              {step.arguments ? <ToolInput input={step.arguments} /> : null}
              {step.result ? (
                <ToolOutput
                  output={step.result}
                  errorText={step.status === "error" ? step.result : undefined}
                />
              ) : null}
            </ToolContent>
          </Tool>
        ))}

        {message.content ? (
          <MessageResponse isAnimating={message.streaming}>{message.content}</MessageResponse>
        ) : null}

        {message.cancelled ? (
          <p className="mt-1 text-xs text-amber-600 dark:text-amber-400">{"已中止"}</p>
        ) : null}
      </MessageContent>
    </Message>
  );
}

interface UserMessageBlockProps {
  text: string;
}

export function UserMessageBlock({ text }: UserMessageBlockProps) {
  return (
    <Message from="user">
      <MessageContent>
        <p className="whitespace-pre-wrap">{text}</p>
      </MessageContent>
    </Message>
  );
}
