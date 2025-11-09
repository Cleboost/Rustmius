import EventBus from "./EventBus";
import SSHSession from "./SSHSession";
import type {
  CommandResult,
  CommandSpec,
  SSHEventEnvelope,
  SSHEventMap,
  SSHEventName,
  SSHSessionOptions,
  SSHSubscribeFilter,
  VPSId,
} from "./types";
import { SSH_EVENT_NAMES } from "./types";

const wildcardToRegExp = (pattern: string): RegExp => {
  const escaped = pattern.replace(/[.+?^${}()|[\]\\]/g, "\\$&").replace(/\*/g, ".*");
  return new RegExp(`^${escaped}$`);
};

const resolveEventNames = (pattern?: string): SSHEventName[] => {
  if (!pattern || pattern === "*") {
    return [...SSH_EVENT_NAMES];
  }
  if (pattern.includes("*")) {
    const regex = wildcardToRegExp(pattern);
    return SSH_EVENT_NAMES.filter((event) => regex.test(event));
  }
  return SSH_EVENT_NAMES.filter((event) => event === pattern);
};

const shouldDeliverEvent = (
  filter: SSHSubscribeFilter,
  eventName: SSHEventName,
  payload: SSHEventMap[SSHEventName],
): boolean => {
  if (filter.vpsId && payload.vpsId !== filter.vpsId) return false;
  if (filter.commandId) {
    if (
      typeof (payload as { commandId?: string }).commandId === "string" &&
      (payload as { commandId?: string }).commandId !== filter.commandId
    ) {
      return false;
    }
  }

  if (filter.event) {
    const names = resolveEventNames(filter.event);
    if (!names.includes(eventName)) return false;
  }

  return true;
};

export class SSHManager {
  private static instance: SSHManager | null = null;

  private readonly sessions = new Map<VPSId, SSHSession>();
  private readonly eventBus = new EventBus();

  static getInstance(): SSHManager {
    if (!SSHManager.instance) {
      SSHManager.instance = new SSHManager();
    }
    return SSHManager.instance;
  }

  private constructor() {
    // singleton
  }

  async ensureSession(
    vpsId: VPSId,
    options: SSHSessionOptions,
  ): Promise<SSHSession> {
    let session = this.sessions.get(vpsId);

    if (!session) {
      const newSession = new SSHSession(vpsId, options, this.eventBus);
      this.sessions.set(vpsId, newSession);
      try {
        await newSession.connect();
      } catch (error) {
        this.sessions.delete(vpsId);
        throw error;
      }
      return newSession;
    }

    try {
      await session.connect();
    } catch (error) {
      throw error;
    }

    return session;
  }

  getSession(vpsId: VPSId): SSHSession | undefined {
    return this.sessions.get(vpsId);
  }

  async close(vpsId: VPSId): Promise<void> {
    const session = this.sessions.get(vpsId);
    if (!session) return;
    this.sessions.delete(vpsId);
    await session.dispose();
  }

  subscribe(
    filter: SSHSubscribeFilter,
    handler: (event: SSHEventEnvelope) => void,
  ): { unsubscribe: () => void } {
    const eventNames = resolveEventNames(filter.event);
    const subscriptions = eventNames.map((eventName) => {
      const listener = (payload: SSHEventMap[typeof eventName]) => {
        if (!shouldDeliverEvent(filter, eventName, payload)) return;
        handler({ type: eventName, payload });
      };
      this.eventBus.on(eventName, listener as (...args: unknown[]) => void);
      return { eventName, listener };
    });

    return {
      unsubscribe: () => {
        for (const { eventName, listener } of subscriptions) {
          this.eventBus.off(eventName, listener as (...args: unknown[]) => void);
        }
      },
    };
  }

  async runCommand(
    vpsId: VPSId,
    options: SSHSessionOptions,
    spec: CommandSpec,
  ): Promise<CommandResult> {
    const session = await this.ensureSession(vpsId, options);
    return session.enqueue(spec);
  }
}

export default SSHManager;
