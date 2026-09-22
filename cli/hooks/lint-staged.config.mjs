/**
 * Tasks the commit hook runs over the staged TypeScript sources of every workspace member.
 */
const ESLINT = [
  "eslint",
  "--max-warnings 0",
  "--cache",
  "--cache-location target/eslint/cache-staged.json",
  "--cache-strategy content",
  "--fix",
].join(" ");

export default {
  "{bin/xrf-ui,cli,packages}/**/*.{cjs,js,mjs,ts,tsx}": ["prettier --write", ESLINT],
};
