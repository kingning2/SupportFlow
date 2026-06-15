import { Banner } from "@douyinfe/semi-ui-19";

import { localizeChannelText, type ChannelLocalized } from "@supportflow/shared";

export function ChannelHint({ hint, lang }: { hint: ChannelLocalized; lang: string }) {
  const text = localizeChannelText(hint, lang);
  if (!text) {
    return null;
  }

  return (
    <Banner
      type="warning"
      fullMode={false}
      bordered
      closeIcon={null}
      description={text}
      style={{ marginBottom: 16 }}
    />
  );
}
