import { EventsPlugin, WirestatePlugin } from "@wirestate/core";
import { DevToolsPlugin } from "@wirestate/core/devtools";

import { ObservablePlugin } from "@/lib/mobx";

/**
 * The plugins every container runs with, application and test alike.
 *
 * @param withDevtoolsPlugin - Whether devtools plugin should be attached.
 * @returns Plugin instances for one container.
 */
export function createContainerPlugins(withDevtoolsPlugin: boolean = false): Array<WirestatePlugin> {
  return [new ObservablePlugin(), new EventsPlugin(), ...(withDevtoolsPlugin ? [new DevToolsPlugin()] : [])];
}
