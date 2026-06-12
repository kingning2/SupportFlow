export interface ExamplePrompt {
  id: string;
  title: string;
  text: string;
  prompt: string;
  iconClassName: string;
  iconColorClassName: string;
}

export const EXAMPLE_PROMPTS: ExamplePrompt[] = [
  {
    id: "sys",
    title: "项目与目录",
    text: "帮我快速梳理当前项目结构和关键入口。",
    prompt: "帮我快速梳理当前项目结构和关键入口。",
    iconClassName: "bg-blue-50 dark:bg-blue-900/30",
    iconColorClassName: "text-blue-500"
  },
  {
    id: "task",
    title: "任务拆解",
    text: "把这个需求拆成可执行步骤，并给出优先级。",
    prompt: "把这个需求拆成可执行步骤，并给出优先级。",
    iconClassName: "bg-amber-50 dark:bg-amber-900/30",
    iconColorClassName: "text-amber-500"
  },
  {
    id: "code",
    title: "代码修改",
    text: "直接帮我定位问题、修改代码并验证结果。",
    prompt: "直接帮我定位问题、修改代码并验证结果。",
    iconClassName: "bg-emerald-50 dark:bg-emerald-900/30",
    iconColorClassName: "text-emerald-500"
  },
  {
    id: "knowledge",
    title: "知识库整理",
    text: "把这批资料整理成便于检索的知识结构。",
    prompt: "把这批资料整理成便于检索的知识结构。",
    iconClassName: "bg-violet-50 dark:bg-violet-900/30",
    iconColorClassName: "text-violet-500"
  },
  {
    id: "skill",
    title: "技能与工具",
    text: "看看当前有哪些技能和工具可以直接使用。",
    prompt: "看看当前有哪些技能和工具可以直接使用。",
    iconClassName: "bg-rose-50 dark:bg-rose-900/30",
    iconColorClassName: "text-rose-500"
  },
  {
    id: "web",
    title: "联网查证",
    text: "需要最新信息时，帮我联网检索并给出来源。",
    prompt: "需要最新信息时，帮我联网检索并给出来源。",
    iconClassName: "bg-cyan-50 dark:bg-cyan-900/30",
    iconColorClassName: "text-cyan-500"
  }
];
