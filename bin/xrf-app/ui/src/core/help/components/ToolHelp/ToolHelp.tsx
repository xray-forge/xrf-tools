import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { XrfMark } from "@xrf/ui/brand";
import { ReactElement } from "react";

import { getBuildKind, PRODUCT_NAME } from "@/core/configs";
import { ToolHelpButton, ToolHelpDialog } from "@/core/help/components";
import { HelpService } from "@/core/help/services/help.service";
import { IToolDescriptor, IToolHelp } from "@/core/routing/tool";
import { ShellToolService } from "@/core/shell/services/shell-tool";

/**
 * The current screen's help: the rail control and the dialog.
 */
export function ToolHelp(): ReactElement {
  const helpService: HelpService = useInjection(HelpService);
  const shellToolService: ShellToolService = useInjection(ShellToolService);

  const tool: Nullable<IToolDescriptor> = shellToolService.tool;
  const help: Nullable<IToolHelp> = helpService.help;

  return (
    <>
      <ToolHelpButton isDisabled={!help} onClick={helpService.open} />

      {help ? (
        <ToolHelpDialog
          title={tool?.label ?? PRODUCT_NAME}
          icon={tool?.icon ?? <XrfMark core={getBuildKind()} />}
          help={help}
          isOpen={helpService.isOpen}
          onClose={helpService.close}
        />
      ) : null}
    </>
  );
}
