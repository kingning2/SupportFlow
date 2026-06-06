"use client";

import { InputGroup, InputGroupTextarea } from "@supportflow/ui/input-group";
import { cn } from "@supportflow/shared";
import type { FileUIPart, SourceDocumentUIPart } from "ai";
import { nanoid } from "nanoid";
import type {
  ChangeEvent,
  ChangeEventHandler,
  ClipboardEventHandler,
  ComponentProps,
  FormEvent,
  FormEventHandler,
  HTMLAttributes,
  KeyboardEventHandler
} from "react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import {
  convertBlobUrlToDataUrl,
  LocalAttachmentsContext,
  LocalReferencedSourcesContext,
  type AttachmentsContext,
  type ReferencedSourcesContext,
  useOptionalPromptInputController,
  usePromptInputAttachments
} from "./prompt-input-context";

export interface PromptInputMessage {
  text: string;
  files: FileUIPart[];
}

export type PromptInputProps = Omit<HTMLAttributes<HTMLFormElement>, "onSubmit" | "onError"> & {
  accept?: string;
  multiple?: boolean;
  globalDrop?: boolean;
  syncHiddenInput?: boolean;
  maxFiles?: number;
  maxFileSize?: number;
  onError?: (err: { code: "max_files" | "max_file_size" | "accept"; message: string }) => void;
  onSubmit: (
    message: PromptInputMessage,
    event: FormEvent<HTMLFormElement>
  ) => void | Promise<void>;
};

function matchAccept(file: File, accept?: string) {
  if (!accept?.trim()) {
    return true;
  }

  return accept
    .split(",")
    .map((item) => item.trim())
    .filter(Boolean)
    .some((pattern) => {
      if (pattern.endsWith("/*")) {
        return file.type.startsWith(pattern.slice(0, -1));
      }
      return file.type === pattern;
    });
}

function validateIncomingFiles(params: {
  accept?: string;
  files: File[] | FileList;
  maxFileSize?: number;
  onError?: PromptInputProps["onError"];
}) {
  const { accept, files, maxFileSize, onError } = params;
  const incoming = [...files];
  const accepted = incoming.filter((file) => matchAccept(file, accept));
  if (incoming.length > 0 && accepted.length === 0) {
    onError?.({ code: "accept", message: "No files match the accepted types." });
    return [];
  }

  const sized = accepted.filter((file) => (maxFileSize ? file.size <= maxFileSize : true));
  if (accepted.length > 0 && sized.length === 0) {
    onError?.({ code: "max_file_size", message: "All files exceed the maximum size." });
    return [];
  }

  return sized;
}

function toUiFiles(files: File[]) {
  return files.map(
    (file) =>
      ({
        filename: file.name,
        id: nanoid(),
        mediaType: file.type,
        type: "file" as const,
        url: URL.createObjectURL(file)
      }) satisfies FileUIPart & { id: string }
  );
}

function revokeFiles(files: (FileUIPart & { id: string })[]) {
  for (const file of files) {
    if (file.url) {
      URL.revokeObjectURL(file.url);
    }
  }
}

function usePromptInputFiles(params: {
  accept?: string;
  controller: ReturnType<typeof useOptionalPromptInputController>;
  maxFiles?: number;
  maxFileSize?: number;
  onError?: PromptInputProps["onError"];
}) {
  const { accept, controller, maxFiles, maxFileSize, onError } = params;
  const usingProvider = Boolean(controller);
  const [items, setItems] = useState<(FileUIPart & { id: string })[]>([]);
  const providerAttachments = controller?.attachments;
  const files = providerAttachments?.files ?? items;
  const inputRef = useRef<HTMLInputElement | null>(null);
  const filesRef = useRef(files);

  useEffect(() => {
    filesRef.current = files;
  }, [files]);

  const addFiles = useCallback(
    (incomingFiles: File[] | FileList) => {
      const valid = validateIncomingFiles({
        accept,
        files: incomingFiles,
        maxFileSize,
        onError
      });
      if (valid.length === 0) {
        return;
      }

      const currentCount = usingProvider ? files.length : 0;
      const capacity =
        typeof maxFiles === "number" ? Math.max(0, maxFiles - currentCount) : undefined;
      const capped = typeof capacity === "number" ? valid.slice(0, capacity) : valid;
      if (typeof capacity === "number" && valid.length > capacity) {
        onError?.({ code: "max_files", message: "Too many files. Some were not added." });
      }

      if (usingProvider) {
        if (capped.length > 0) {
          providerAttachments?.add(capped);
        }
        return;
      }

      setItems((prev) => [...prev, ...toUiFiles(capped)]);
    },
    [accept, controller, files.length, maxFileSize, maxFiles, onError, usingProvider]
  );

  const removeLocal = useCallback((id: string) => {
    setItems((prev) => {
      const found = prev.find((file) => file.id === id);
      if (found?.url) {
        URL.revokeObjectURL(found.url);
      }
      return prev.filter((file) => file.id !== id);
    });
  }, []);

  const clearLocal = useCallback(() => {
    setItems((prev) => {
      revokeFiles(prev);
      return [];
    });
  }, []);

  useEffect(
    () => () => {
      if (!usingProvider) {
        revokeFiles(filesRef.current);
      }
    },
    [usingProvider]
  );

  const openFileDialog = useCallback(() => {
    if (usingProvider) {
      providerAttachments?.openFileDialog();
      return;
    }
    inputRef.current?.click();
  }, [providerAttachments, usingProvider]);

  const clear = useCallback(() => {
    if (usingProvider) {
      providerAttachments?.clear();
      return;
    }
    clearLocal();
  }, [clearLocal, providerAttachments, usingProvider]);

  const remove = providerAttachments?.remove ?? removeLocal;

  return {
    addFiles,
    attachmentsCtx: {
      add: addFiles,
      clear,
      fileInputRef: inputRef,
      files: files.map((file) => ({ ...file, id: file.id })),
      openFileDialog,
      remove
    } satisfies AttachmentsContext,
    files,
    inputRef
  };
}

function useReferencedSources() {
  const [referencedSources, setReferencedSources] = useState<
    (SourceDocumentUIPart & { id: string })[]
  >([]);

  return useMemo<ReferencedSourcesContext>(
    () => ({
      add: (incoming: SourceDocumentUIPart[] | SourceDocumentUIPart) => {
        const array = Array.isArray(incoming) ? incoming : [incoming];
        setReferencedSources((prev) => [
          ...prev,
          ...array.map((item) => ({ ...item, id: nanoid() }))
        ]);
      },
      clear: () => setReferencedSources([]),
      remove: (id: string) => {
        setReferencedSources((prev) => prev.filter((item) => item.id !== id));
      },
      sources: referencedSources
    }),
    [referencedSources]
  );
}

function useDropHandlers(params: {
  addFiles: (files: File[] | FileList) => void;
  formRef: React.RefObject<HTMLFormElement | null>;
  globalDrop?: boolean;
}) {
  const { addFiles, formRef, globalDrop } = params;

  useEffect(() => {
    const form = formRef.current;
    if (!form || globalDrop) {
      return;
    }

    const onDragOver = (event: DragEvent) => {
      if (event.dataTransfer?.types?.includes("Files")) {
        event.preventDefault();
      }
    };
    const onDrop = (event: DragEvent) => {
      if (event.dataTransfer?.types?.includes("Files")) {
        event.preventDefault();
      }
      if (event.dataTransfer?.files?.length) {
        addFiles(event.dataTransfer.files);
      }
    };

    form.addEventListener("dragover", onDragOver);
    form.addEventListener("drop", onDrop);
    return () => {
      form.removeEventListener("dragover", onDragOver);
      form.removeEventListener("drop", onDrop);
    };
  }, [addFiles, formRef, globalDrop]);

  useEffect(() => {
    if (!globalDrop) {
      return;
    }

    const onDragOver = (event: DragEvent) => {
      if (event.dataTransfer?.types?.includes("Files")) {
        event.preventDefault();
      }
    };
    const onDrop = (event: DragEvent) => {
      if (event.dataTransfer?.types?.includes("Files")) {
        event.preventDefault();
      }
      if (event.dataTransfer?.files?.length) {
        addFiles(event.dataTransfer.files);
      }
    };

    document.addEventListener("dragover", onDragOver);
    document.addEventListener("drop", onDrop);
    return () => {
      document.removeEventListener("dragover", onDragOver);
      document.removeEventListener("drop", onDrop);
    };
  }, [addFiles, globalDrop]);
}

export const PromptInput = ({
  className,
  accept,
  children,
  globalDrop,
  maxFileSize,
  maxFiles,
  multiple,
  onError,
  onSubmit,
  syncHiddenInput,
  ...props
}: PromptInputProps) => {
  const controller = useOptionalPromptInputController();
  const usingProvider = Boolean(controller);
  const formRef = useRef<HTMLFormElement | null>(null);
  const { addFiles, attachmentsCtx, files, inputRef } = usePromptInputFiles({
    accept,
    controller,
    maxFileSize,
    maxFiles,
    onError
  });
  const refsCtx = useReferencedSources();

  useEffect(() => {
    if (usingProvider && controller) {
      controller.__registerFileInput(inputRef, () => inputRef.current?.click());
    }
  }, [controller, inputRef, usingProvider]);

  useEffect(() => {
    if (syncHiddenInput && inputRef.current && files.length === 0) {
      inputRef.current.value = "";
    }
  }, [files.length, inputRef, syncHiddenInput]);

  useDropHandlers({ addFiles, formRef, globalDrop });

  const clearAfterSubmit = useCallback(() => {
    attachmentsCtx.clear();
    refsCtx.clear();
    if (usingProvider && controller) {
      controller.textInput.clear();
    }
  }, [attachmentsCtx, controller, refsCtx, usingProvider]);

  const handleChange: ChangeEventHandler<HTMLInputElement> = useCallback(
    (event) => {
      if (event.currentTarget.files) {
        addFiles(event.currentTarget.files);
      }
      event.currentTarget.value = "";
    },
    [addFiles]
  );

  const handleSubmit: FormEventHandler<HTMLFormElement> = useCallback(
    async (event) => {
      event.preventDefault();

      const form = event.currentTarget;
      const text = usingProvider
        ? (controller?.textInput.value ?? "")
        : (new FormData(form).get("message") as string) || "";

      if (!usingProvider) {
        form.reset();
      }

      try {
        const convertedFiles = await Promise.all(
          files.map(async ({ id: _id, ...item }) =>
            item.url?.startsWith("blob:")
              ? { ...item, url: (await convertBlobUrlToDataUrl(item.url)) ?? item.url }
              : item
          )
        );

        await onSubmit({ files: convertedFiles, text }, event);
        clearAfterSubmit();
      } catch {
        // keep current values for retry
      }
    },
    [clearAfterSubmit, controller, files, onSubmit, usingProvider]
  );

  return (
    <LocalAttachmentsContext.Provider value={attachmentsCtx}>
      <LocalReferencedSourcesContext.Provider value={refsCtx}>
        <input
          accept={accept}
          aria-label="Upload files"
          className="hidden"
          multiple={multiple}
          onChange={handleChange}
          ref={inputRef}
          title="Upload files"
          type="file"
        />
        <form className={cn("w-full", className)} onSubmit={handleSubmit} ref={formRef} {...props}>
          <InputGroup className="overflow-hidden">{children}</InputGroup>
        </form>
      </LocalReferencedSourcesContext.Provider>
    </LocalAttachmentsContext.Provider>
  );
};

export type PromptInputBodyProps = HTMLAttributes<HTMLDivElement>;

export const PromptInputBody = ({ className, ...props }: PromptInputBodyProps) => (
  <div className={cn("contents", className)} {...props} />
);

export type PromptInputTextareaProps = ComponentProps<typeof InputGroupTextarea>;

export const PromptInputTextarea = ({
  className,
  onChange,
  onKeyDown,
  placeholder = "What would you like to know?",
  ...props
}: PromptInputTextareaProps) => {
  const controller = useOptionalPromptInputController();
  const attachments = usePromptInputAttachments();
  const [isComposing, setIsComposing] = useState(false);

  const handleKeyDown: KeyboardEventHandler<HTMLTextAreaElement> = useCallback(
    (event) => {
      onKeyDown?.(event);
      if (event.defaultPrevented) {
        return;
      }

      if (
        event.key === "Enter" &&
        !isComposing &&
        !event.nativeEvent.isComposing &&
        !event.shiftKey
      ) {
        event.preventDefault();
        const submitButton = event.currentTarget.form?.querySelector(
          'button[type="submit"]'
        ) as HTMLButtonElement | null;
        if (!submitButton?.disabled) {
          event.currentTarget.form?.requestSubmit();
        }
      }

      if (
        event.key === "Backspace" &&
        event.currentTarget.value === "" &&
        attachments.files.length > 0
      ) {
        event.preventDefault();
        const lastAttachment = attachments.files.at(-1);
        if (lastAttachment) {
          attachments.remove(lastAttachment.id);
        }
      }
    },
    [attachments, isComposing, onKeyDown]
  );

  const handlePaste: ClipboardEventHandler<HTMLTextAreaElement> = useCallback(
    (event) => {
      const files = [...(event.clipboardData?.items ?? [])]
        .filter((item) => item.kind === "file")
        .map((item) => item.getAsFile())
        .filter((file): file is File => Boolean(file));

      if (files.length > 0) {
        event.preventDefault();
        attachments.add(files);
      }
    },
    [attachments]
  );

  const controlledProps = controller
    ? {
        onChange: (event: ChangeEvent<HTMLTextAreaElement>) => {
          controller.textInput.setInput(event.currentTarget.value);
          onChange?.(event);
        },
        value: controller.textInput.value
      }
    : { onChange };

  return (
    <InputGroupTextarea
      className={cn("field-sizing-content max-h-48 min-h-16", className)}
      name="message"
      onCompositionEnd={() => setIsComposing(false)}
      onCompositionStart={() => setIsComposing(true)}
      onKeyDown={handleKeyDown}
      onPaste={handlePaste}
      placeholder={placeholder}
      {...props}
      {...controlledProps}
    />
  );
};
