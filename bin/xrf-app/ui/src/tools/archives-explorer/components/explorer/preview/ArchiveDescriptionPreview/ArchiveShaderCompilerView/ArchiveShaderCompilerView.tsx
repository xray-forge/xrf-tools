import { ArchiveShaderCompilerDescription, ArchiveShaderCompilerShader } from "@xrf/ipc/types/xrf-app-archives";
import { EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReadingColumn } from "@xrf/ui/layout/reading-column";
import { ReactElement } from "react";

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
    <ReadingColumn data-testid={dataTestId} id={id} className={className}>
      <EditorPanelSection
        title={`Compiler shaders (${shaders.length})`}
        caption={"Read when the level was built; the blender of the same name in shaders.xr is what draws it"}
      >
        {shaders.map((shader: ArchiveShaderCompilerShader) => (
          <EditorPanelProperty
            isInline
            key={shader.name}
            label={shader.name}
            value={describeCompilerFlags(shader)}
            caption={describeBaking(shader)}
          />
        ))}
      </EditorPanelSection>
    </ReadingColumn>
  );
}
