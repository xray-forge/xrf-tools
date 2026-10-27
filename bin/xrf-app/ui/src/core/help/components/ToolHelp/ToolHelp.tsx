import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { XrfMark } from "@xrf/ui/brand";
import { ReactElement, useEffect } from "react";
import { useLocation } from "react-router-dom";

import { getBuildKind, PRODUCT_NAME } from "@/core/configs";
import { ToolHelpButton, ToolHelpDialog } from "@/core/help/components";
import { HelpService } from "@/core/help/services/help.service";
import { LAUNCHER_HELP } from "@/core/launcher/help";
import { useCurrentTool } from "@/core/routing/current-tool.context";
import { IToolDescriptor, IToolHelp } from "@/core/routing/tool";

/**
 * The current screen's help: the rail control and the dialog.
 */
export function ToolHelp(): ReactElement {
  const helpService: HelpService = useInjection(HelpService);
  const tool: Nullable<IToolDescriptor> = useCurrentTool();
  const { pathname } = useLocation();

  const isLauncher: boolean = !tool && pathname === "/";
  const help: Nullable<IToolHelp> = (tool?.help ?? (isLauncher ? LAUNCHER_HELP : null)) || null;

  useEffect(() => helpService.setTool(tool?.id ?? null, Boolean(help)), [tool?.id, help, helpService]);

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
