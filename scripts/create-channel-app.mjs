#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.resolve(__dirname, "..");
const templateDir = path.join(rootDir, "templates", "channel-app");
const appsDir = path.join(rootDir, "apps");

function toPascalCase(input) {
  return input
    .split(/[^a-zA-Z0-9]+/)
    .filter(Boolean)
    .map((part) => part[0].toUpperCase() + part.slice(1))
    .join("");
}

function toConstCase(input) {
  return input
    .split(/[^a-zA-Z0-9]+/)
    .filter(Boolean)
    .map((part) => part.toUpperCase())
    .join("_");
}

function toCamelCase(input) {
  const pascal = toPascalCase(input);
  return pascal ? pascal[0].toLowerCase() + pascal.slice(1) : "";
}

function ensureDir(dir) {
  fs.mkdirSync(dir, { recursive: true });
}

function readTemplate(filePath) {
  return fs.readFileSync(filePath, "utf8");
}

function writeFile(targetPath, content) {
  ensureDir(path.dirname(targetPath));
  fs.writeFileSync(targetPath, content, "utf8");
}

function replaceTokens(value, replacements) {
  return Object.entries(replacements).reduce(
    (result, [token, replacement]) => result.replaceAll(`__${token}__`, replacement),
    value
  );
}

function copyTemplateDir(sourceDir, targetDir, replacements) {
  for (const entry of fs.readdirSync(sourceDir, { withFileTypes: true })) {
    const sourcePath = path.join(sourceDir, entry.name);
    const targetName = replaceTokens(entry.name, replacements);
    const targetPath = path.join(targetDir, targetName);

    if (entry.isDirectory()) {
      ensureDir(targetPath);
      copyTemplateDir(sourcePath, targetPath, replacements);
      continue;
    }

    writeFile(targetPath, replaceTokens(readTemplate(sourcePath), replacements));
  }
}

function parseArgs(argv) {
  const options = {};

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (!arg.startsWith("--")) {
      continue;
    }

    const key = arg.slice(2);
    const value = argv[index + 1];
    if (!value || value.startsWith("--")) {
      throw new Error(`Missing value for --${key}`);
    }

    options[key] = value;
    index += 1;
  }

  return options;
}

function printUsage() {
  console.log(
    [
      "Usage:",
      "  pnpm run create:channel -- --name <slug> --title <title> [options]",
      "",
      "Required:",
      "  --name           App slug, e.g. dingtalk",
      "  --title          Window title, e.g. SupportFlow · 钉钉",
      "",
      "Optional:",
      "  --description    Metadata description",
      "  --label          Landing page heading",
      "  --logo-text      Title bar logo text",
      "  --gradient-from  Accent gradient start color",
      "  --gradient-to    Accent gradient end color",
      "  --bar-class      Title bar className",
      "  --title-class    Title text className",
      "  --control-class  Control button className",
      "  --content-class  Desktop app shell content className"
    ].join("\n")
  );
}

try {
  const args = parseArgs(process.argv.slice(2));

  const slug = args.name?.trim().toLowerCase();
  const title = args.title?.trim();

  if (!slug || !title) {
    printUsage();
    process.exit(1);
  }

  if (!/^[a-z0-9-]+$/.test(slug)) {
    throw new Error("--name must use lowercase letters, numbers, or hyphens");
  }

  const appDir = path.join(appsDir, slug);
  if (fs.existsSync(appDir)) {
    throw new Error(`Target app already exists: apps/${slug}`);
  }

  const pascalName = toPascalCase(slug);
  const constName = toConstCase(slug);
  const camelName = toCamelCase(slug);

  const replacements = {
    CHANNEL_SLUG: slug,
    CHANNEL_PASCAL: pascalName,
    CHANNEL_CONST: constName,
    CHANNEL_CAMEL: camelName,
    CHANNEL_TITLE: title,
    CHANNEL_DESCRIPTION: args.description?.trim() ?? `${title}通道`,
    CHANNEL_LABEL: args.label?.trim() ?? title,
    CHANNEL_PAGE_COMPONENT: `${pascalName}Page`,
    CHANNEL_LOGO_TEXT: args["logo-text"]?.trim() ?? pascalName.slice(0, 1),
    CHANNEL_GRADIENT_FROM: args["gradient-from"]?.trim() ?? "#4A9AFF",
    CHANNEL_GRADIENT_TO: args["gradient-to"]?.trim() ?? "#267EF0",
    CHANNEL_BAR_CLASS:
      args["bar-class"]?.trim() ?? "border-b border-black/10 bg-white/90 backdrop-blur",
    CHANNEL_TITLE_CLASS: args["title-class"]?.trim() ?? "text-slate-800",
    CHANNEL_CONTROL_CLASS:
      args["control-class"]?.trim() ?? "text-slate-600 hover:bg-slate-900/5 hover:text-slate-900",
    CHANNEL_CONTENT_CLASS:
      args["content-class"]?.trim() ?? "relative flex min-h-0 flex-1 flex-col overflow-hidden"
  };

  ensureDir(appDir);
  copyTemplateDir(templateDir, appDir, replacements);

  console.log(`Created apps/${slug}`);
  console.log(`Next steps:`);
  console.log(`  1. Implement platform logic in apps/${slug}/src/features/${slug}`);
  console.log(`  2. Add flavor-specific scripts if needed`);
  console.log(`  3. Run pnpm --filter @supportflow/app-${slug} typecheck`);
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
}
