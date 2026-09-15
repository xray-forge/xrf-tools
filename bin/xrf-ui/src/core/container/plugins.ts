import { CommandsPlugin, EventsPlugin, QueriesPlugin, WirestatePlugin } from "@wirestate/core";
import { DevToolsPlugin } from "@wirestate/core/devtools";
import { ObservablePlugin } from "@wirestate/mobx";

import { KeybindCommandBindingPlugin } from "@/core/commands";
import { FlowCancellationPlugin } from "@/lib/mobx/flow/cancellation.plugin";

/**
 * The plugins every container runs with, application and test alike.
 *
 * @param withDevtoolsPlugin - Whether devtools plugin should be attached.
 * @returns Plugin instances for one container.
 */
export function createContainerPlugins(withDevtoolsPlugin: boolean = false): Array<WirestatePlugin> {
  return [
    new CommandsPlugin(),
    new EventsPlugin(),
    new FlowCancellationPlugin(),
    new KeybindCommandBindingPlugin(),
    new ObservablePlugin(),
    new QueriesPlugin(),
    ...(withDevtoolsPlugin ? [new DevToolsPlugin()] : []),
  ];
}
