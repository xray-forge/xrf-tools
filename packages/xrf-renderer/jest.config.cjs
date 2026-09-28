const { createJestConfig } = require("@xrf/toolchain/jest");

/**
 * The renderer's own tests: maths and processing with pure results, in node, with no GPU.
 *
 * @type {import('jest').Config}
 */
module.exports = createJestConfig({ rootDir: __dirname });
