/** Modal 面板注册名（对应 `src/components/modal/panels`） */
export enum ModalPanel {
  Demo = "demo"
}

export function isModalPanel(value: string): value is ModalPanel {
  return Object.values(ModalPanel).includes(value as ModalPanel);
}
