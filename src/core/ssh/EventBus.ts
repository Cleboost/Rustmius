import { EventEmitter } from "events";
import type { SSHEventMap, SSHEventName } from "./types";

type Listener<E extends SSHEventName> = (payload: SSHEventMap[E]) => void;

export class EventBus extends EventEmitter {
  constructor() {
    super();
    this.setMaxListeners(0);
  }

  emit<E extends SSHEventName>(eventName: E, payload: SSHEventMap[E]): boolean {
    return super.emit(eventName, payload);
  }

  on<E extends SSHEventName>(eventName: E, listener: Listener<E>): this {
    return super.on(eventName, listener as (...args: unknown[]) => void);
  }

  once<E extends SSHEventName>(eventName: E, listener: Listener<E>): this {
    return super.once(eventName, listener as (...args: unknown[]) => void);
  }

  off<E extends SSHEventName>(eventName: E, listener: Listener<E>): this {
    return super.off(eventName, listener as (...args: unknown[]) => void);
  }

  addListener<E extends SSHEventName>(
    eventName: E,
    listener: Listener<E>,
  ): this {
    return super.addListener(eventName, listener as (...args: unknown[]) => void);
  }

  removeListener<E extends SSHEventName>(
    eventName: E,
    listener: Listener<E>,
  ): this {
    return super.removeListener(
      eventName,
      listener as (...args: unknown[]) => void,
    );
  }

  subscribe<E extends SSHEventName>(
    eventName: E,
    listener: Listener<E>,
  ): () => void {
    this.on(eventName, listener);
    return () => this.off(eventName, listener);
  }
}

export default EventBus;
