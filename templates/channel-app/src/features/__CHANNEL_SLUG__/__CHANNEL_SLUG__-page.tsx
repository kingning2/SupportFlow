"use client";

import { MessageCircle } from "lucide-react";

export function __CHANNEL_PAGE_COMPONENT__() {
  return (
    <div className="flex h-full min-h-0 flex-col overflow-hidden">
      <div className="min-h-0 flex-1 overflow-y-auto p-6">
        <div className="mx-auto flex max-w-lg flex-col items-center py-20 text-center">
          <div className="mb-4 flex size-16 items-center justify-center rounded-2xl bg-slate-900/5">
            <MessageCircle className="size-8 text-slate-700" />
          </div>
          <h1 className="text-xl font-bold text-slate-800">__CHANNEL_LABEL__</h1>
          <p className="mt-2 text-sm text-slate-500">Platform scaffold created.</p>
          <p className="mt-6 rounded-lg border border-dashed border-slate-200 px-4 py-3 text-xs text-slate-400">
            Start implementation in `src/features/__CHANNEL_SLUG__`.
          </p>
        </div>
      </div>
    </div>
  );
}
