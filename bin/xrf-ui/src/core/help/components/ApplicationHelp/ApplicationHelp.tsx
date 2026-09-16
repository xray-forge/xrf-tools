import { useInjection } from "@wirestate/react";
import { ReactElement, useEffect } from "react";

import { ApplicationHelpButton } from "@/core/help/components/ApplicationHelp/ApplicationHelpButton";
import { ApplicationHelpDialog } from "@/core/help/components/ApplicationHelp/ApplicationHelpDialog";
import { HelpService } from "@/core/help/services/help";
import { IApplicationDescriptor } from "@/core/routing/application";
import { useCurrentApplication } from "@/core/routing/current-application.context";
import { Nullable } from "@/lib/types/general";

/**
 * The current application's help: the toolbar action and the dialog.
 *
 * `F1` is no longer handled here - it is the `help/open` command, dispatched by `core/keybinds` against the same
 * service this button calls, so the shortcut and the button cannot disagree about whether help exists.
 */
export function ApplicationHelp(): Nullable<ReactElement> {
  const helpService: HelpService = useInjection(HelpService);
  const application: Nullable<IApplicationDescriptor> = useCurrentApplication();
  const help = application?.help;

  useEffect(
    () => helpService.setApplication(application?.id ?? null, Boolean(help)),
    [application?.id, help, helpService]
  );

  if (!application || !help) {
    return null;
  }

  return (
    <>
      <ApplicationHelpButton onClick={helpService.open} />

      <ApplicationHelpDialog
        application={application}
        help={help}
        isOpen={helpService.isOpen}
        onClose={helpService.close}
      />
    </>
  );
}
