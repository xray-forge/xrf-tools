import { RenderAppliedReport, RenderViewOptions } from "@xrf/ipc/types/xrf-renderer";
import { Nullable } from "@xrf/types";
import { EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import {
  ILevelAppliedReadout,
  ILevelAppliedSection,
  LEVEL_APPLIED_SECTIONS,
} from "@/tools/level-viewer/lib/panels/level-applied-readouts";

interface ILevelRendererAppliedSectionProps extends BaseComponentProps {
  /** What the renderer draws the level with, or null until it says. */
  applied: Nullable<RenderAppliedReport>;
  /** What the level asked it to draw with, which a changed value is told against. */
  requested: RenderViewOptions;
}

/**
 * What the renderer actually draws the level with, which the settings and the toolbar only ask for: read-only, each
 * line as `LEVEL_APPLIED_SECTIONS` reads it, a value the renderer changed saying what was asked.
 */
export function LevelRendererAppliedSection({
  "data-testid": dataTestId = "level-renderer-applied-section",
  id,
  className,
  applied,
  requested,
}: ILevelRendererAppliedSectionProps): ReactElement {
  if (!applied) {
    return (
      <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"Applied"}>
        <EditorPanelProperty label={"Renderer"} value={"Not drawing yet"} />
      </EditorPanelSection>
    );
  }

  return (
    <>
      {LEVEL_APPLIED_SECTIONS.filter((section: ILevelAppliedSection) => section.isShown?.(applied) ?? true).map(
        (section: ILevelAppliedSection, index: number) => (
          <EditorPanelSection
            key={section.title}
            data-testid={index === 0 ? dataTestId : undefined}
            id={index === 0 ? id : undefined}
            className={index === 0 ? className : undefined}
            title={section.title}
          >
            {section.readouts.map((readout: ILevelAppliedReadout) => {
              const value: Nullable<string> = readout.read(applied, requested);

              return value === null ? null : (
                <EditorPanelProperty
                  key={readout.label}
                  label={readout.label}
                  value={value}
                  isMonospace={readout.isMonospace}
                />
              );
            })}
          </EditorPanelSection>
        )
      )}
    </>
  );
}
