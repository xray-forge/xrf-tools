import { formatCount } from "@xrf/format";
import { ERenderDrawableKind, RenderDrawableKind, RenderDrawableReport } from "@xrf/ipc/types/xrf-renderer";
import { EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Fragment, ReactElement } from "react";

/** How each drawable kind is named, and what it draws counted in. */
const DRAWABLE_KIND_NAMES: Readonly<Record<RenderDrawableKind, { title: string; unit: string }>> = {
  [ERenderDrawableKind.GRASS]: { title: "Grass", unit: "tufts" },
};

interface ILevelRendererDrawablesSectionProps extends BaseComponentProps {
  drawables: ReadonlyArray<RenderDrawableReport>;
}

/**
 * What each drawable kind beside the static scene came to on a recent frame, a section a kind: what its cull kept,
 * against the most its lists hold.
 */
export function LevelRendererDrawablesSection({
  "data-testid": dataTestId = "level-renderer-drawables-section",
  id,
  className,
  drawables,
}: ILevelRendererDrawablesSectionProps): ReactElement {
  return (
    <Fragment>
      {drawables.map(({ kind, drawn, capacity }) => {
        const { title, unit } = DRAWABLE_KIND_NAMES[kind];

        return (
          <EditorPanelSection
            key={kind}
            data-testid={`${dataTestId}-${kind}`}
            id={id && `${id}-${kind}`}
            className={className}
            title={title}
          >
            <EditorPanelProperty
              label={"Drawn"}
              value={
                drawn > capacity
                  ? `${formatCount(capacity)} ${unit}, the lists full: ${formatCount(drawn)} kept`
                  : `${formatCount(drawn)} ${unit}`
              }
            />
            <EditorPanelProperty label={"Lists hold"} value={`${formatCount(capacity)} ${unit}`} />
          </EditorPanelSection>
        );
      })}
    </Fragment>
  );
}
