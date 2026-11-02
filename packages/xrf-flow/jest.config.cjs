const { createJestConfig } = require("@xrf/toolchain/jest");

/**
 * The workspace base, compiled with decorators for the flow decorators under test.
 */
module.exports = createJestConfig({ rootDir: __dirname, hasDecorators: true });
