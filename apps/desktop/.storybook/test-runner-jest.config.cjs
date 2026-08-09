const path = require("node:path");
const { getJestConfig } = require("@storybook/test-runner");

const base = getJestConfig();
const playwright = base.testEnvironmentOptions?.["jest-playwright"] ?? {};
const channel = process.env.PLAYWRIGHT_CHANNEL || "chrome";

module.exports = {
  ...base,
  rootDir: path.resolve(__dirname, ".."),
  testEnvironmentOptions: {
    ...base.testEnvironmentOptions,
    "jest-playwright": {
      ...playwright,
      launchOptions: {
        ...(playwright.launchOptions ?? {}),
        channel,
      },
    },
  },
};
