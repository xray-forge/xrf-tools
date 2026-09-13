import { HostInfo } from "@/core/bindings/types/xrf-app";
import { formatBytes } from "@/lib/memory/format";

import { IAboutRow, statedRows } from "./about-row";

/**
 * @param host - What the backend reported about the machine and the runtime it is using.
 * @returns Every row worth stating about it, in the order they read.
 */
export function describeHost(host: HostInfo): Array<IAboutRow> {
  return statedRows([
    ["Tauri", host.tauriVersion],
    ["Webview", host.webviewVersion],
    ["Platform", [host.osName, host.osVersion].filter(Boolean).join(" ") || null],
    ["Kernel", host.kernelVersion],
    ["Architecture", host.arch],
    ["Processors", describeProcessors(host)],
    ["Memory", formatBytes(host.totalMemory)],
    ["Process", String(host.pid)],
  ]);
}

/** Logical processors, and the physical cores behind them where the platform separates the two. */
function describeProcessors(host: HostInfo): string {
  const logical: string = `${host.cpuCount} logical`;

  return host.physicalCoreCount ? `${logical}, ${host.physicalCoreCount} physical` : logical;
}
