/** @type {import('@commitlint/types').UserConfig} */
const commitlintConfig = {
  extends: ["@commitlint/config-conventional"],
  rules: {
    "type-enum": [
      2,
      "always",
      ["feat", "fix", "docs", "chore", "refactor", "perf", "test", "build", "style", "ci", "revert"]
    ],
    "subject-empty": [2, "never"],
    "subject-full-stop": [0],
    "header-max-length": [2, "always", 100]
  }
};

export default commitlintConfig;
