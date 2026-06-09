import { defineConfig, globalIgnores } from "eslint/config";
import prettier from "eslint-config-prettier/flat";
import sonarjs from "eslint-plugin-sonarjs";

const eslintConfig = defineConfig([
  prettier,
  globalIgnores([
    "out/**",
    "apps/**/out/**",
    "build/**",
    "node_modules/**",
    "src-tauri/**",
    "apps/full/src/components/ai-elements/**",
    ".cursor/**",
    ".agents/**"
  ]),
  {
    plugins: {
      sonarjs
    },
    rules: {
      "@typescript-eslint/no-explicit-any": "off",
      "@typescript-eslint/no-unused-vars": "off",
      "@typescript-eslint/no-this-alias": "off",
      "react-hooks/exhaustive-deps": "off",
      "react/display-name": "off",
      "@typescript-eslint/no-empty-object-type": "off",
      "react-hooks/immutability": "off",
      // 1. 限制单个文件最大行数为 500 行
      "max-lines": [
        "error",
        {
          max: 500,
          skipBlankLines: true, // 忽略空行
          skipComments: true // 忽略注释（只算纯代码）
        }
      ],

      // 2. 进阶推荐：限制单个函数/方法最大不能超过 300 行（函数太长是模块不清晰的元凶）
      "max-lines-per-function": [
        "error",
        {
          max: 300,
          skipBlankLines: true,
          skipComments: true
        }
      ],

      complexity: ["error", { max: 20 }],
      "sonarjs/cognitive-complexity": ["error", 20],
      "max-depth": ["error", { max: 3 }]
    }
  }
]);

export default eslintConfig;
