"use client";

/**
 * @deprecated Import from `@douyinfe/semi-ui-19` instead.
 *             This `@supportflow/ui/*` path is a compatibility shim only.
 */
import * as React from "react";
import { Button, Input, TextArea } from "@douyinfe/semi-ui-19";

type Align = "inline-start" | "inline-end" | "block-start" | "block-end";

const groupStyle: React.CSSProperties = {
  display: "flex",
  width: "100%",
  alignItems: "center",
  border: "1px solid var(--semi-color-border)",
  borderRadius: 6,
  overflow: "hidden",
  background: "var(--semi-color-bg-0)"
};

function addonStyle(align: Align): React.CSSProperties {
  if (align === "inline-end") {
    return { order: 2, padding: "0 12px" };
  }
  if (align === "block-start") {
    return { order: 0, width: "100%", padding: "12px 12px 0" };
  }
  if (align === "block-end") {
    return { order: 2, width: "100%", padding: "0 12px 12px" };
  }
  return { order: 0, padding: "0 12px" };
}

function InputGroup({ className, style, ...props }: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="input-group"
      role="group"
      className={className}
      style={{ ...groupStyle, ...style }}
      {...props}
    />
  );
}

function InputGroupAddon({
  className,
  align = "inline-start",
  style,
  ...props
}: React.ComponentProps<"div"> & { align?: Align }) {
  return (
    <div
      role="group"
      data-slot="input-group-addon"
      data-align={align}
      className={className}
      style={{
        display: "flex",
        alignItems: "center",
        color: "var(--semi-color-text-2)",
        fontSize: 14,
        ...addonStyle(align),
        ...style
      }}
      onClick={(e) => {
        if ((e.target as HTMLElement).closest("button")) {
          return;
        }
        (
          e.currentTarget.parentElement?.querySelector("input, textarea") as HTMLElement | null
        )?.focus?.();
      }}
      {...props}
    />
  );
}

function InputGroupButton(props: React.ComponentProps<typeof Button>) {
  return <Button theme="borderless" type="tertiary" size="small" {...props} />;
}

function InputGroupText({ className, style, ...props }: React.ComponentProps<"span">) {
  return (
    <span
      className={className}
      style={{
        display: "inline-flex",
        alignItems: "center",
        gap: 8,
        color: "var(--semi-color-text-2)",
        fontSize: 14,
        ...style
      }}
      {...props}
    />
  );
}

function InputGroupInput({
  className,
  style,
  ...props
}: Omit<React.ComponentProps<typeof Input>, "prefix" | "suffix">) {
  return (
    <Input
      data-slot="input-group-control"
      className={className}
      style={{ flex: 1, border: "none", boxShadow: "none", ...style }}
      {...props}
    />
  );
}

function InputGroupTextarea({ className, style, ...props }: React.ComponentProps<typeof TextArea>) {
  return (
    <TextArea
      data-slot="input-group-control"
      className={className}
      style={{ flex: 1, border: "none", boxShadow: "none", resize: "none", ...style }}
      {...props}
    />
  );
}

/** @deprecated Use Semi components from `@douyinfe/semi-ui-19` instead. */

export {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupText,
  InputGroupInput,
  InputGroupTextarea
};
