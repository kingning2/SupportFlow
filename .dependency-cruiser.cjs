/** @type {import("dependency-cruiser").IConfiguration} */
module.exports = {
  forbidden: [
    {
      name: "no-circular",
      severity: "error",
      comment: "TypeScript 模块之间禁止形成循环依赖。",
      from: {},
      to: {
        circular: true
      }
    },
    {
      name: "shared-no-ui-or-apps",
      severity: "error",
      comment: "@supportflow/shared 只承载共享类型与 IPC 薄桥接，不能反向依赖 UI 或具体应用。",
      from: {
        path: "^packages/shared/src"
      },
      to: {
        path: "^(packages/ui/src|apps/[^/]+/src)"
      }
    },
    {
      name: "ui-no-apps",
      severity: "error",
      comment: "@supportflow/ui 只放通用视图层，不能依赖具体 app 页面实现。",
      from: {
        path: "^packages/ui/src"
      },
      to: {
        path: "^apps/[^/]+/src"
      }
    },
    {
      name: "full-no-other-apps",
      severity: "error",
      comment: "apps/full 不能依赖其他 app 实现代码，应用层共享逻辑应沉到 packages/*。",
      from: {
        path: "^apps/full/src"
      },
      to: {
        path: "^apps/(wechat|wework)/src"
      }
    },
    {
      name: "wechat-no-other-apps",
      severity: "error",
      comment: "apps/wechat 不能依赖其他 app 实现代码，应用层共享逻辑应沉到 packages/*。",
      from: {
        path: "^apps/wechat/src"
      },
      to: {
        path: "^apps/(full|wework)/src"
      }
    },
    {
      name: "wework-no-other-apps",
      severity: "error",
      comment: "apps/wework 不能依赖其他 app 实现代码，应用层共享逻辑应沉到 packages/*。",
      from: {
        path: "^apps/wework/src"
      },
      to: {
        path: "^apps/(full|wechat)/src"
      }
    }
  ],
  options: {
    tsPreCompilationDeps: true,
    combinedDependencies: true,
    doNotFollow: {
      path: "node_modules"
    },
    exclude: {
      path: "(^|/)(node_modules|\\.next|out|dist|coverage)(/|$)"
    },
    enhancedResolveOptions: {
      conditionNames: ["import", "require", "node", "default"],
      extensions: [".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs", ".json"]
    }
  }
};
