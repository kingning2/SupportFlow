import { AppShellLayout, APP_SHELL_CONTENT_CLASS } from "@supportflow/ui/app-shell";

export default function MainWindowLayout({
  children
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <AppShellLayout modal bgGuard contentClassName={APP_SHELL_CONTENT_CLASS.console}>
      {children}
    </AppShellLayout>
  );
}
