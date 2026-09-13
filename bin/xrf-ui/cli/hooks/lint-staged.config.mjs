/**
 * Tasks the commit hook runs over the staged frontend sources.
 */
const ESLINT = [
  "eslint",
  "--config cli/lint/eslint.config.mjs",
  "--max-warnings 0",
  "--cache",
  "--cache-location target/eslint/cache-staged.json",
  "--cache-strategy content",
  "--fix",
].join(" ");

export default {
  "*.{cjs,js,mjs,ts,tsx}": ["prettier --write", ESLINT],
};
