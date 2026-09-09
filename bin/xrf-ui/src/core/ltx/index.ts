// Everything outside this domain reaches it here; modules inside it import siblings by path, which is what keeps the
// barrel from becoming a load-time cycle.
export * from "./lib/verify";
