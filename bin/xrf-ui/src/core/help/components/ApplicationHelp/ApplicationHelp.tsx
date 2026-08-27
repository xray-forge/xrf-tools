import { ReactElement, useCallback, useEffect, useState } from "react";

import { ApplicationHelpButton } from "@/core/help/components/ApplicationHelp/ApplicationHelpButton";
import { ApplicationHelpDialog } from "@/core/help/components/ApplicationHelp/ApplicationHelpDialog";
import { IApplicationDescriptor } from "@/core/routing/application";
import { useCurrentApplication } from "@/core/routing/current-application.context";
import { Nullable } from "@/lib/types/general";

/**
 * The current application's help: the caption-row button, the `F1` shortcut, and the dialog.
 *
 * Help always describes the tool that is open, so outside an application with authored help there is
 * no affordance at all rather than a disabled one.
 */
export function ApplicationHelp(): Nullable<ReactElement> {
  const application: Nullable<IApplicationDescriptor> = useCurrentApplication();
  const help = application?.help;

  const [isOpen, setIsOpen] = useState(false);

  const close = useCallback(() => setIsOpen(false), []);

  // Whatever was being read belonged to the application it was read in.
  useEffect(() => close(), [application?.id, close]);

  useEffect(() => {
    if (!help) {
      return;
    }

    function onKeyDown(event: KeyboardEvent): void {
      if (event.key === "F1") {
        event.preventDefault();
        setIsOpen(true);
      }
    }

    window.addEventListener("keydown", onKeyDown);

    return () => window.removeEventListener("keydown", onKeyDown);
  }, [help]);

  if (!application || !help) {
    return null;
  }

  return (
    <>
      <ApplicationHelpButton onClick={() => setIsOpen(true)} />

      <ApplicationHelpDialog application={application} help={help} isOpen={isOpen} onClose={close} />
    </>
  );
}
