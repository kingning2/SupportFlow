import MainProvider from "./main-provider";

export default function MainLayout({
  children
}: Readonly<{
  children: React.ReactNode;
}>) {
  return <MainProvider>{children}</MainProvider>;
}
