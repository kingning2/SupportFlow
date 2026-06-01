import { Info } from "lucide-react";

import { localizeCowText, type CowLocalized } from "@/cmd/cow-python-channels";

export function ChannelHint({ hint, lang }: { hint: CowLocalized; lang: string }) {
  const text = localizeCowText(hint, lang);
  if (!text) {
    return null;
  }
  return (
    <div className="mb-4 rounded-lg border border-amber-200 bg-amber-50 px-4 py-3 dark:border-amber-800/40 dark:bg-amber-900/20">
      <p className="text-xs leading-relaxed text-amber-800 dark:text-amber-200/90">
        <Info className="mr-1.5 inline size-3.5 align-text-bottom opacity-80" />
        {text}
      </p>
    </div>
  );
}
