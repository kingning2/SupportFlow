# `tools_manifest.json` 编写说明

本文描述启动器工具清单的格式与约定。清单是**单一数据源**：编辑后需重新构建应用，Rust 在编译时通过 `include_str!` 嵌入 `src-tauri/resources/tools_manifest.json`。

## 文件位置与构建

| 项目     | 说明                                                                                     |
| -------- | ---------------------------------------------------------------------------------------- |
| 路径     | `src-tauri/resources/tools_manifest.json`                                                |
| 打包     | 在 `tauri.conf.json` 的 `bundle.resources` 中声明，随应用分发                            |
| 解析     | `src-tauri/src/config/tools_manifest.rs`（`ToolManifestEntry` + `PlatformDownloadSpec`） |
| 前端类型 | `src/config/tools-manifest.ts`（字段名与 JSON **camelCase** 对齐）                       |

修改 JSON 后执行 `cargo build` / `tauri build` 等即可触发重新解析；解析失败会在**编译期** `panic`，错误信息会提示检查 `kind`、`variant` 等枚举值。

## 顶层结构

文件为 **JSON 数组**，每个元素描述一个工具卡片。

```json
[
  {
    "id": "system-repair",
    "downloadSpec": {},
    "hot": true,
    "variant": "hero-left"
  }
]
```

### 根级字段

| 字段           | 类型    | 必填 | 说明                                                                                                                                                                    |
| -------------- | ------- | ---- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `id`           | string  | 是   | 稳定唯一标识，建议 `kebab-case`（如 `phone-unlock`）。用于下载目录名、日志等；**文案键**由前端将 `-` 转为 `_`，对应 i18n：`tools:{id转下划线}.title` / `.description`。 |
| `downloadSpec` | object  | 是   | 下载地址、平台安装检测、启动路径解析等，见下文。                                                                                                                        |
| `hot`          | boolean | 否   | 默认 `false`，用于 UI 角标等展示。                                                                                                                                      |
| `variant`      | string  | 是   | 卡片布局档位，仅允许：`hero-left`、`medium`、`small`（与 `ToolVariant` 一致，**kebab-case**）。                                                                         |

## `downloadSpec`（`PlatformDownloadSpec`）

所有键名均为 **camelCase**。各子块可按需省略；未配置的平台在**发起下载**时会报「缺少该平台下载源」类错误。

### 平台下载源：`windows` / `macos`

类型为 `PlatformArtifacts`，可包含：

| 键          | 说明                                                                                   |
| ----------- | -------------------------------------------------------------------------------------- |
| `universal` | 任意架构通用包（x64 / arm64 在解析时都会优先用架构专用键，没有再回退到 `universal`）。 |
| `x64`       | 仅 x64。                                                                               |
| `arm64`     | 仅 arm64（Apple Silicon 等）。                                                         |

每个 artifact 为 `DownloadArtifact`：

| 字段          | 类型   | 说明                                                                                                                                                                            |
| ------------- | ------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `url`         | string | 直接下载地址（与 `downloadKey` 二选一）。                                                                                                                                       |
| `downloadKey` | string | 解析 API 的 `name` 参数；存在即表示可下载。请求 `GET {downloadResolveBaseUrl}?name={downloadKey}`，响应 JSON：`{ "version", "url", "lastUpdated" }`，实际下载使用返回的 `url`。 |
| `fileName`    | string | 可选。保存到应用数据目录时的文件名；未填时从最终 `url` 路径推导。                                                                                                               |
| `kind`        | string | **`zip`** 或 **`executable`**（小写 kebab-case）。`zip` 解压目录为 `{工具下载根}/{id}/`；可执行则落地为单文件。                                                                 |

可选 **`downloadResolveBaseUrl`**（在 `downloadSpec` 根级）：默认 `https://strapi.gbyte.com/api/v1/system-config/info`（见仓库根 `.env`）。

**前端「是否可点下载」**：`toolHasDownloadForPlatform` 要求**当前平台**的 `windows` 或 `macos` 下，`universal` / `x64` / `arm64` 任一槽位配置了非空的 `url` 或 `downloadKey`。

### Windows：安装检测 `windowsProductRegistry`

| 字段               | 说明                                                                                                  |
| ------------------ | ----------------------------------------------------------------------------------------------------- |
| `hklmSoftwarePath` | 相对 HKLM 的软件键路径，例如 `SOFTWARE\Gbyte\Repair`，需与安装器写入的「产品数据根」一致。            |
| `uninstallSubkey`  | `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\{此名称}` 下的子键名，例如 `gbyte_repair`。 |

**规则**：未配置时，Windows 上 **`installed` 恒为 `false`**（无法从注册表判断）。配置后：卸载项存在，且（若能读到）`InstallPath` 指向的目录存在则视为已安装；读不到 `InstallPath` 时仅以卸载项存在为准（与 `src-tauri/src/utils/platform/windows/download.rs` 一致）。

### Windows：启动主程序 `windowsMainExecutableRelative`

相对 **注册表 `InstallPath` 目录** 的主程序相对路径，例如 `repair.exe`、`Gbyte Data Transfer.exe`。用于从已安装目录拼出可执行文件路径；与 zip 流程里的 `windowsZipInstallSteps.mainExecutableRelative` **二选一即可**；若两者都配置，**以 zip 步骤中的为准**。

### Windows：zip 解压后收尾 `windowsZipInstallSteps`

当 **payload 为 zip** 且与 `windowsProductRegistry` **同时存在**时，解压后可执行注册表、语言/gclid、防火墙等步骤（不创建快捷方式）。字段包括：

- `mainExecutableRelative`、`uninstallerRelative`
- `displayName`、`publisher`、`displayVersion`
- `firewallMaxConcurrent`（默认 8；兼容旧键名 `firewallScanMaxExes`）
- `writeLangRegistry`、`writeGclidFromEnv`（默认 `true`）

具体语义以 Rust 结构体 `WindowsZipInstallSteps`（`src-tauri/src/utils/platform/download.rs`）为准。

### macOS：已安装判定 `macosInstalledBundlePath`

若填写（如 `/Applications/Gbyte Repair.app`），**「已安装」以该路径在磁盘上存在为准**。不填时回退为：在应用工具下载目录下根据当前架构解析出的 artifact，判断落地文件是否存在或可解压目录非空（见 `macos/download.rs` 的 `is_tool_download_installed`）。

## 与行为的关系（速查）

| 能力                       | 依赖配置                                                                                                                               |
| -------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| Windows 显示「已安装」     | 通常需要 `windowsProductRegistry` 与真实安装器写入的注册表一致。                                                                       |
| Windows 从安装目录启动     | `windowsProductRegistry` + `windowsMainExecutableRelative`（或 zip 步骤里的主程序相对路径）。                                          |
| macOS 显示「已安装」       | 优先 `macosInstalledBundlePath`；否则看下载目录落地/解压结果。                                                                         |
| 仅展示、不提供本应用内下载 | 可不配 `windows`/`macos` 的 artifact；此时 UI 上应表现为不可下载，但若机器上已通过其他渠道安装且注册表/bundle 匹配，仍可能显示已安装。 |

## 校验建议

1. **JSON**：合法 UTF-8，根类型为数组，键名 **camelCase**。
2. **枚举**：`variant` ∈ `hero-left` | `medium` | `small`；`kind` ∈ `zip` | `executable`。
3. **id**：全局唯一；新增工具时在 `resources/languages/*.json` 的 `tools` 命名空间下增加对应 `snake_case` 键的 `title` / `description`。
4. **注册表字符串**：与安装器约定完全一致（大小写、反斜杠路径）。
5. **改完即编**：依赖编译期嵌入，改 JSON 后务必本地 `cargo test` 或完整构建，避免上线后解析失败。

## 相关代码索引

- 清单结构：`src-tauri/src/config/tools_manifest.rs`
- 下载规格与枚举：`src-tauri/src/utils/platform/download.rs`
- Windows 安装检测：`src-tauri/src/utils/platform/windows/download.rs`
- macOS 安装检测：`src-tauri/src/utils/platform/macos/download.rs`
- 安装态汇总：`src-tauri/src/utils/tools.rs`（`gather_install_state`）
- 前端类型与辅助函数：`src/config/tools-manifest.ts`
