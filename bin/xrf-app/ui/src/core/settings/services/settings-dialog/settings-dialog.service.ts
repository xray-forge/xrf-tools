import { Injectable } from "@wirestate/core";
import { BoundAction, Observable } from "@wirestate/mobx";

import { KeybindCommand } from "@/core/commands";
import { OPEN_SETTINGS_KEYBIND_COMMAND } from "@/core/settings/commands";
import { ESettingsSection } from "@/core/settings/lib/settings-section";

/**
 * Owns whether the settings dialog is on screen, and which of its sections it shows.
 */
@Injectable()
export class SettingsDialogService {
  @Observable()
  public isOpen: boolean = false;

  /** Kept between openings, so the dialog comes back where it was left. */
  @Observable()
  public section: ESettingsSection = ESettingsSection.GENERAL;

  /** Shows settings where they were left. */
  @KeybindCommand(OPEN_SETTINGS_KEYBIND_COMMAND)
  @BoundAction()
  public open(): void {
    this.isOpen = true;
  }

  /**
   * Shows settings at one section.
   *
   * @param section - Section to show.
   */
  @BoundAction()
  public openSection(section: ESettingsSection): void {
    this.section = section;
    this.isOpen = true;
  }

  /**
   * Switches the open dialog to another section.
   *
   * @param section - Section to show.
   */
  @BoundAction()
  public setSection(section: ESettingsSection): void {
    this.section = section;
  }

  @BoundAction()
  public close(): void {
    this.isOpen = false;
  }
}
