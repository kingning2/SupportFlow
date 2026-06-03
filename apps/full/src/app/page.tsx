import { redirect } from "next/navigation";

/** 完整控制台入口：直接重定向，避免客户端初始化阶段白屏。 */
export default function Root() {
  redirect("/main-window");
}
