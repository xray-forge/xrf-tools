const swcJest = require("@swc/jest");

/**
 * Compiles a workspace member's TypeScript for jest, the way its bundler would, with no application transforms.
 */
module.exports = swcJest.createTransformer({
  jsc: {
    parser: {
      syntax: "typescript",
      tsx: false,
    },
    target: "es2022",
  },
  module: { type: "commonjs" },
  sourceMaps: true,
});
