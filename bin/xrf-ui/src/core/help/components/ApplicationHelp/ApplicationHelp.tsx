import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ReactElement, useEffect } from "react";
import { useLocation } from "react-router-dom";

import { XrfMark } from "@/core/brand/XrfMark";
import { ApplicationHelpButton, ApplicationHelpDialog } from "@/core/help/components";
import { HelpService } from "@/core/help/services/help";
import { LAUNCHER_HELP } from "@/core/launcher/help";
import { IApplicationDescriptor, IApplicationHelp } from "@/core/routing/application";
import { useCurrentApplication } from "@/core/routing/current-application.context";

/**
 * The current screen's help: the rail control and the dialog.
 */
export function ApplicationHelp(): ReactElement {
  const helpService: HelpService = useInjection(HelpService);
  const application: Nullable<IApplicationDescriptor> = useCurrentApplication();
  const { pathname } = useLocation();

  const isLauncher: boolean = !application && pathname === "/";
  const help: Nullable<IApplicationHelp> = (application?.help ?? (isLauncher ? LAUNCHER_HELP : null)) || null;

  useEffect(
    () => helpService.setApplication(application?.id ?? null, Boolean(help)),
    [application?.id, help, helpService]
  );

  return (
    <>
      <ApplicationHelpButton isDisabled={!help} onClick={helpService.open} />

      {help ? (
        <ApplicationHelpDialog
          title={application?.label ?? "XRF Tools"}
          icon={application?.icon ?? <XrfMark />}
          help={help}
          isOpen={helpService.isOpen}
          onClose={helpService.close}
        />
      ) : null}
    </>
  );
}
