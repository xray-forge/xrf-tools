import { ReactElement } from "react";

import { PlannedTool } from "@/core/shell/editor/PlannedTool";

export function SoundEnvironmentsEditorTool(): ReactElement {
  return <PlannedTool description={"Edit acoustic environments and reverb presets."} />;
}
