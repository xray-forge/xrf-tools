import { ArchiveShaderCompilerDescription, ArchiveShaderCompilerShader } from "@xrf/ipc/types/xrf-app-archives";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { EditorPanelSection } from "@xrf/ui/shell/editor/EditorPanel/EditorPanelSection";
import { ReactElement } from "react";

import { ArchiveDescriptionLayout } from "../ArchiveDescriptionLayout";
import { ArchiveDescriptionRow } from "../ArchiveDescriptionRow";
import { describeBaking, describeCompilerFlags } from "./ArchiveShaderCompilerView.utils";

interface IArchiveShaderCompilerViewProps extends BaseComponentProps {
  description: ArchiveShaderCompilerDescription;
}

/**
 * The compiler shader library: what the level compiler was told about each surface, which is not how it is drawn.
 */
export function ArchiveShaderCompilerView({
  "data-testid": dataTestId = "archive-shader-compiler-view",
  id,
  className,
  description,
}: IArchiveShaderCompilerViewProps): ReactElement {
  const { shaders } = description;

  return (
    <ArchiveDescriptionLayout data-testid={dataTestId} id={id} className={className}>
      <EditorPanelSection
        title={`Compiler shaders (${shaders.length})`}
        caption={"Read when the level was built; the blender of the same name in shaders.xr is what draws it"}
        isFirst
      >
        {shaders.map((shader: ArchiveShaderCompilerShader) => (
          <ArchiveDescriptionRow
            key={shader.name}
            label={shader.name}
            value={describeCompilerFlags(shader)}
            caption={describeBaking(shader)}
          />
        ))}
      </EditorPanelSection>
    </ArchiveDescriptionLayout>
  );
}
