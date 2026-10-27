const { createJestConfig } = require("@xrf/toolchain/jest");

module.exports = createJestConfig({
  rootDir: __dirname,
  hasJsx: true,
  moduleNameMapper: {
    "\\.css$": require.resolve("@xrf/toolchain/jest-asset-stub"),
  },
  testEnvironment: "jsdom",
});
