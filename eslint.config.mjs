import { defineConfig, globalIgnores } from "eslint/config";
import nextVitals from "eslint-config-next/core-web-vitals";
import nextTs from "eslint-config-next/typescript";
import prettier from "eslint-config-prettier/flat";

const eslintConfig = defineConfig([
  ...nextVitals,
  ...nextTs,
  prettier,
  globalIgnores([
    ".next/**",
    "out/**",
    "build/**",
    "next-env.d.ts",
    "node_modules/**",
    "src-tauri/**",
    "src/components/ai-elements/**",
    ".cursor/**",
    ".agents/**"
  ]),
  {
    rules: {
      "@typescript-eslint/no-explicit-any": "off",
      "@typescript-eslint/no-unused-vars": "off",
      "@typescript-eslint/no-this-alias": "off",
      "react-hooks/exhaustive-deps": "off",
      "react/display-name": "off",
      "@typescript-eslint/no-empty-object-type": "off",
      "@next/next/no-img-element": "off",
      "react-hooks/immutability": "off"
    }
  }
]);

export default eslintConfig;
