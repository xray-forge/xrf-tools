import { DetailSection } from "@xrf/ui/layout/detail-section";
import { ReactElement } from "react";

/**
 * Where the third-party licence notices ship: the release archive carries them as files beside the executable rather
 * than inside it.
 */
export function SettingsLicensesSection(): ReactElement {
  return (
    <DetailSection
      data-testid={"settings-licenses-section"}
      title={"Licences"}
      description={
        "xrf-app is MIT licensed. The notices of the third-party code compiled into it are in the licenses folder " +
        "beside xrf-app.exe, and each bundled library's own licence sits beside it under resources."
      }
    />
  );
}
