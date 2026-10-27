import { ReactElement } from "react";

import { PlannedTool } from "@/core/shell/chrome/PlannedTool";

export function SoundEnvironmentsEditorTool(): ReactElement {
  return <PlannedTool description={"Edit acoustic environments and reverb presets."} />;
}
