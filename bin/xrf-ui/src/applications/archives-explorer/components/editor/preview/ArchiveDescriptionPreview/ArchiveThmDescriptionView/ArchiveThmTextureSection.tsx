import { ReactElement } from "react";

import { ArchiveDescribeScope, ArchiveThmTexture } from "@/core/ipc/types/xrf-app";
import { EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { ArchiveDescriptionReference } from "../ArchiveDescriptionReference";
import { ArchiveDescriptionRow } from "../ArchiveDescriptionRow";
import { describeTextureShape } from "./ArchiveThmDescriptionView.utils";

interface IArchiveThmTextureSectionProps extends BaseComponentProps {
  texture: ArchiveThmTexture;
  scope: ArchiveDescribeScope;
}

/**
 * What the descriptor describes, which is the file beside it rather than a name it carries.
 *
 * The declared size appears only when the texture measures something else, and a cube map's six-face source strip is
 * explained rather than presented as a disagreement: 53 of the 54 vanilla descriptors whose declaration differs are
 * that, so a surface that cannot say it reports the whole `sky\` directory as inconsistent.
 */
export function ArchiveThmTextureSection({
  "data-testid": dataTestId = "archive-thm-texture-section",
  id,
  className,
  texture,
  scope,
}: IArchiveThmTextureSectionProps): ReactElement {
  return (
    <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"Texture"} isFirst>
      <ArchiveDescriptionReference label={"File"} reference={texture.reference} scope={scope} />

      <ArchiveDescriptionRow label={"Size"} value={describeTextureShape(texture)} />

      {texture.declared ? (
        <ArchiveDescriptionRow
          label={"Declared size"}
          value={`${texture.declared.width} × ${texture.declared.height}`}
          caption={
            texture.declared.isCubeStrip
              ? "The six cube faces the descriptor was converted from, laid out in a row"
              : "What the descriptor records; the file beside it is the authority"
          }
        />
      ) : null}
    </EditorPanelSection>
  );
}
