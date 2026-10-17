import type { BodyAction } from '@nemu/contracts';
import { isRequestableMotion } from '../fsm/slots.js';
import type { PhysicalStateController } from '../fsm/state-controller.js';
import type { ActionExecutorOptions, ActionResultPayload } from './types.js';

export class BodyActionExecutor {
  private readonly stateController: PhysicalStateController;
  private readonly options?: ActionExecutorOptions | undefined;
  private currentActionId: string | null = null;

  constructor(stateController: PhysicalStateController, options?: ActionExecutorOptions | undefined) {
    this.stateController = stateController;
    this.options = options;
  }

  /**
   * Executes a body action request from stage.action.
   * Validates action kind and slot requestability.
   */
  executeAction(actionId: string, action: BodyAction): ActionResultPayload {
    // 1. If currently running another action, cancel it first
    if (this.currentActionId) {
      this.dispatchResult({
        actionId: this.currentActionId,
        status: 'cancelled',
        detail: 'Preempted by new action',
      });
    }

    this.currentActionId = actionId;

    switch (action.kind) {
      case 'motion': {
        const motionId = action.motionId;
        // Validate requestable flag from CONTRACTS.md 1.8
        if (!isRequestableMotion(motionId)) {
          const result: ActionResultPayload = {
            actionId,
            status: 'failed',
            detail: `Motion slot '${motionId}' is not requestable`,
          };
          this.currentActionId = null;
          this.dispatchResult(result);
          return result;
        }

        // Trigger motion execution
        this.options?.onPlayMotion?.(motionId);
        const result: ActionResultPayload = {
          actionId,
          status: 'succeeded',
        };
        this.currentActionId = null;
        this.dispatchResult(result);
        return result;
      }

      case 'goTo': {
        this.stateController.send({ type: 'ACTION_GOTO', anchor: action.anchor });
        const result: ActionResultPayload = {
          actionId,
          status: 'succeeded',
        };
        this.currentActionId = null;
        this.dispatchResult(result);
        return result;
      }

      case 'sit': {
        this.stateController.send({ type: 'ACTION_SIT', platformId: action.platformId });
        const result: ActionResultPayload = {
          actionId,
          status: 'succeeded',
        };
        this.currentActionId = null;
        this.dispatchResult(result);
        return result;
      }

      case 'sleep': {
        this.stateController.send({ type: 'ACTION_SLEEP' });
        const result: ActionResultPayload = {
          actionId,
          status: 'succeeded',
        };
        this.currentActionId = null;
        this.dispatchResult(result);
        return result;
      }

      case 'wake': {
        this.stateController.send({ type: 'ACTION_WAKE' });
        const result: ActionResultPayload = {
          actionId,
          status: 'succeeded',
        };
        this.currentActionId = null;
        this.dispatchResult(result);
        return result;
      }

      case 'peek': {
        this.stateController.send({ type: 'ACTION_PEEK', side: action.side });
        const result: ActionResultPayload = {
          actionId,
          status: 'succeeded',
        };
        this.currentActionId = null;
        this.dispatchResult(result);
        return result;
      }

      case 'gaze': {
        this.options?.onGaze?.(action.target, action.holdMs);
        const result: ActionResultPayload = {
          actionId,
          status: 'succeeded',
        };
        this.currentActionId = null;
        this.dispatchResult(result);
        return result;
      }

      default: {
        const result: ActionResultPayload = {
          actionId,
          status: 'failed',
          detail: `Unsupported action kind: ${(action as { kind?: string }).kind}`,
        };
        this.currentActionId = null;
        this.dispatchResult(result);
        return result;
      }
    }
  }

  cancelCurrentAction(detail = 'Cancelled'): void {
    if (this.currentActionId) {
      this.dispatchResult({
        actionId: this.currentActionId,
        status: 'cancelled',
        detail,
      });
      this.currentActionId = null;
    }
  }

  getCurrentActionId(): string | null {
    return this.currentActionId;
  }

  private dispatchResult(result: ActionResultPayload): void {
    this.options?.onActionResult?.(result);
  }
}
