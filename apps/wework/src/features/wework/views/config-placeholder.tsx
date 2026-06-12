"use client";

export function ConfigPlaceholder({ title }: { title: string }) {
  return (
    <div className="flex h-full min-h-0 flex-col overflow-hidden">
      <div className="min-h-0 flex-1 overflow-auto p-6">
        <h2 className="text-lg font-semibold text-[#1A2B4A]">{title}</h2>
        <p className="mt-2 text-sm text-slate-500">
          该页面后续会接入相关配置能力，当前先保留占位。
        </p>
      </div>
    </div>
  );
}
