import { describe, expect, it } from "@jest/globals";
import { Container, Injectable } from "@wirestate/core";
import { BoundAction, Observable } from "@wirestate/mobx";

import { defineKeybindCommand, EKeybindCommandCategory, IKeybindCommand } from "@/core/commands";
import { KeybindCommand } from "@/core/commands/lib/command.decorator";
import { KeybindCommandsService } from "@/core/commands/services/commands";
import { mockContainer } from "@/fixtures/utils/container";

const SHADOWED: IKeybindCommand = defineKeybindCommand({
  category: EKeybindCommandCategory.APPLICATION,
  chords: ["F6"],
  description: "Answered by whichever scope is innermost.",
  id: "fixture/shadowed",
  label: "Shadowed",
});

const GUARDED: IKeybindCommand = defineKeybindCommand({
  category: EKeybindCommandCategory.APPLICATION,
  description: "Refused until its owner is ready.",
  id: "fixture/guarded",
  label: "Guarded",
});

@Injectable()
class OuterHandlerService {
  public calls: number = 0;

  @KeybindCommand(SHADOWED)
  public act(): void {
    this.calls += 1;
  }
}

@Injectable()
class InnerHandlerService {
  public calls: number = 0;

  @KeybindCommand(SHADOWED)
  public act(): void {
    this.calls += 1;
  }
}

@Injectable()
class GuardedHandlerService {
  public calls: number = 0;

  @Observable()
  public isReady: boolean = false;

  @BoundAction()
  public setReady(isReady: boolean): void {
    this.isReady = isReady;
  }

  @KeybindCommand(GUARDED, { isEnabled: (service: GuardedHandlerService) => service.isReady })
  public act(): void {
    this.calls += 1;
  }
}

describe("KeybindCommandBindingPlugin", () => {
  it("binds a decorated method for the lifetime of its container, with no registration of its own", () => {
    const container: Container = mockContainer([OuterHandlerService]).provision();
    const commandsService: KeybindCommandsService = container.get(KeybindCommandsService);

    expect(commandsService.isAvailable(SHADOWED)).toBe(true);
    expect(commandsService.execute(SHADOWED)).toBe(true);
    expect(container.get(OuterHandlerService).calls).toBe(1);

    container.deprovision();

    expect(commandsService.isAvailable(SHADOWED)).toBe(false);
    expect(commandsService.execute(SHADOWED)).toBe(false);
  });

  it("force-activates a service nothing injected, so a command-only owner still answers", () => {
    const container: Container = mockContainer([OuterHandlerService]).provision();

    // Nothing resolved the service before dispatching; the plugin's participation is what wired it.
    container.get(KeybindCommandsService).execute(SHADOWED);

    expect(container.get(OuterHandlerService).calls).toBe(1);
  });

  it("lets an inner scope shadow an outer one, and restores the outer when it releases", () => {
    const root: Container = mockContainer([OuterHandlerService]).provision();
    const commandsService: KeybindCommandsService = root.get(KeybindCommandsService);
    const child: Container = new Container({ bindings: [InnerHandlerService], parent: root }).provision();

    commandsService.execute(SHADOWED);

    expect(child.get(InnerHandlerService).calls).toBe(1);
    expect(root.get(OuterHandlerService).calls).toBe(0);

    child.deprovision();
    commandsService.execute(SHADOWED);

    expect(child.get(InnerHandlerService).calls).toBe(1);
    expect(root.get(OuterHandlerService).calls).toBe(1);
  });

  it("refuses a command its own handler guards, without the caller knowing the condition", () => {
    const container: Container = mockContainer([GuardedHandlerService]).provision();
    const commandsService: KeybindCommandsService = container.get(KeybindCommandsService);
    const guarded: GuardedHandlerService = container.get(GuardedHandlerService);

    expect(commandsService.isAvailable(GUARDED)).toBe(false);
    expect(commandsService.execute(GUARDED)).toBe(false);
    expect(guarded.calls).toBe(0);

    guarded.setReady(true);

    expect(commandsService.isAvailable(GUARDED)).toBe(true);
    expect(commandsService.execute(GUARDED)).toBe(true);
    expect(guarded.calls).toBe(1);
  });
});
