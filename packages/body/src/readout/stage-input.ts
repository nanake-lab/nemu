import { controlCommandSchema } from '@nemu/contracts';
import type { CoreConnectionStatus } from './types.js';
import { ReadoutApplicator } from './applicator.js';

/** Validated stage wiring. Legacy bytes decode, but never become v2 animation inputs. */
export class StageReadoutInput {
  private status: CoreConnectionStatus = 'CONNECTED';

  constructor(readonly applicator: ReadoutApplicator) {}

  setConnectionStatus(status: CoreConnectionStatus): void {
    this.status = status;
    if (status !== 'CONNECTED') this.applicator.applyReadout(null, status);
  }

  handle(message: unknown): boolean {
    const result = controlCommandSchema.safeParse(message);
    if (!result.success) return false;
    const command = result.data;
    if (command.type === 'stage.readout.v2') {
      this.applicator.applyReadout(command.payload.readout, this.status);
      return true;
    }
    if (command.type === 'stage.readout') {
      this.applicator.resetToNeutral();
      return true;
    }
    if (command.type === 'stage.event' && command.payload.event.type === 'core.runState') {
      const states = command.payload.event.payload.states;
      this.setConnectionStatus(
        states.includes('CORE_UNMODULATED')
          ? 'CORE_UNMODULATED'
          : states.includes('MODULATION_NOT_ENABLED')
            ? 'MODULATION_NOT_ENABLED'
            : states.includes('DISCONNECTED')
              ? 'DISCONNECTED'
              : 'CONNECTED',
      );
      return true;
    }
    return false;
  }
}
