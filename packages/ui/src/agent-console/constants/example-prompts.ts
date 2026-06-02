/** Welcome screen example cards (i18n keys under `console`). */
export interface ExamplePrompt {
  id: string;
  titleKey: string;
  textKey: string;
  promptKey: string;
  iconClassName: string;
  iconColorClassName: string;
}

export const EXAMPLE_PROMPTS: ExamplePrompt[] = [
  {
    id: "sys",
    titleKey: "example_sys_title",
    textKey: "example_sys_text",
    promptKey: "example_sys_text",
    iconClassName: "bg-blue-50 dark:bg-blue-900/30",
    iconColorClassName: "text-blue-500"
  },
  {
    id: "task",
    titleKey: "example_task_title",
    textKey: "example_task_text",
    promptKey: "example_task_text",
    iconClassName: "bg-amber-50 dark:bg-amber-900/30",
    iconColorClassName: "text-amber-500"
  },
  {
    id: "code",
    titleKey: "example_code_title",
    textKey: "example_code_text",
    promptKey: "example_code_text",
    iconClassName: "bg-emerald-50 dark:bg-emerald-900/30",
    iconColorClassName: "text-emerald-500"
  },
  {
    id: "knowledge",
    titleKey: "example_knowledge_title",
    textKey: "example_knowledge_text",
    promptKey: "example_knowledge_text",
    iconClassName: "bg-violet-50 dark:bg-violet-900/30",
    iconColorClassName: "text-violet-500"
  },
  {
    id: "skill",
    titleKey: "example_skill_title",
    textKey: "example_skill_text",
    promptKey: "example_skill_text",
    iconClassName: "bg-rose-50 dark:bg-rose-900/30",
    iconColorClassName: "text-rose-500"
  },
  {
    id: "web",
    titleKey: "example_web_title",
    textKey: "example_web_text",
    promptKey: "example_web_text",
    iconClassName: "bg-cyan-50 dark:bg-cyan-900/30",
    iconColorClassName: "text-cyan-500"
  }
];
