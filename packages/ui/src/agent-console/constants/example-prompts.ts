export interface ExamplePrompt {
  id: string;
  title: string;
  text: string;
  prompt: string;
  iconBg: string;
  iconColor: string;
}

export const EXAMPLE_PROMPTS: ExamplePrompt[] = [
  {
    id: "sys",
    title: "项目与目录",
    text: "帮我快速梳理当前项目结构和关键入口。",
    prompt: "帮我快速梳理当前项目结构和关键入口。",
    iconBg: "var(--semi-color-info-light-default)",
    iconColor: "var(--semi-color-info)"
  },
  {
    id: "task",
    title: "任务拆解",
    text: "把这个需求拆成可执行步骤，并给出优先级。",
    prompt: "把这个需求拆成可执行步骤，并给出优先级。",
    iconBg: "var(--semi-color-warning-light-default)",
    iconColor: "var(--semi-color-warning)"
  },
  {
    id: "code",
    title: "代码修改",
    text: "直接帮我定位问题、修改代码并验证结果。",
    prompt: "直接帮我定位问题、修改代码并验证结果。",
    iconBg: "var(--semi-color-success-light-default)",
    iconColor: "var(--semi-color-success)"
  },
  {
    id: "knowledge",
    title: "知识库整理",
    text: "把这批资料整理成便于检索的知识结构。",
    prompt: "把这批资料整理成便于检索的知识结构。",
    iconBg: "var(--semi-color-primary-light-default)",
    iconColor: "var(--semi-color-primary)"
  },
  {
    id: "skill",
    title: "技能与工具",
    text: "看看当前有哪些技能和工具可以直接使用。",
    prompt: "看看当前有哪些技能和工具可以直接使用。",
    iconBg: "var(--semi-color-danger-light-default)",
    iconColor: "var(--semi-color-danger)"
  },
  {
    id: "web",
    title: "联网查证",
    text: "需要最新信息时，帮我联网检索并给出来源。",
    prompt: "需要最新信息时，帮我联网检索并给出来源。",
    iconBg: "var(--semi-color-info-light-default)",
    iconColor: "var(--semi-color-info)"
  }
];
