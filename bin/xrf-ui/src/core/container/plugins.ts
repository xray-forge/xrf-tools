import { CommandsPlugin, EventsPlugin, QueriesPlugin, WirestatePlugin } from "@wirestate/core";
import { DevToolsPlugin } from "@wirestate/core/devtools";
import { ObservablePlugin } from "@wirestate/mobx";

import { CommandBindingPlugin } from "@/core/commands";
import { FlowCancellationPlugin } from "@/lib/mobx/flow/cancellation.plugin";

/**
 * The plugins every container runs with, application and test alike.
 *
 * @param withDevtoolsPlugin - Whether devtools plugin should be attached.
 * @returns Plugin instances for one container.
 */
export function createContainerPlugins(withDevtoolsPlugin: boolean = false): Array<WirestatePlugin> {
  return [
    new EventsPlugin(),
    new CommandsPlugin(),
    new QueriesPlugin(),
    new CommandBindingPlugin(),
    new FlowCancellationPlugin(),
    new ObservablePlugin(),
    ...(withDevtoolsPlugin ? [new DevToolsPlugin()] : []),
  ];
}
