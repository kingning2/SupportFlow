/** Localized string or map (SupportFlow channel catalog shape). */
export type ChannelLocalized = string | Record<string, string>;

export interface ChannelField {
  key: string;
  label: ChannelLocalized;
  type: string;
  value: unknown;
  default?: unknown;
  placeholder?: ChannelLocalized;
}

export interface ChannelCatalogEntry {
  name: string;
  label: ChannelLocalized;
  active: boolean;
  fields: ChannelField[];
  hint?: ChannelLocalized;
  icon?: string;
  color?: string;
  login_status?: string;
  loginStatus?: string;
}

export interface ChannelFieldDrafts {
  strings: Record<string, string>;
  bools: Record<string, boolean>;
  maskedCleared: Record<string, boolean>;
}
