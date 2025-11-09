import { Client, ClientChannel, ConnectConfig } from "ssh2";
import CommandQueue from "./CommandQueue";
import EventBus from "./EventBus";
import type {
  CommandCancelReason,
  CommandResult,
  CommandSpec,
  SSHEventMap,
  SSHEventName,
  SSHLogLevel,
  SSHSessionOptions,
  SSHSessionStatus,
  VPSId,
} from "./types";
import { SSH_EVENT_NAMES } from "./types";

const DEFAULT_KEEPALIVE_INTERVAL_MS = 15_000;
const DEFAULT_KEEPALIVE_COUNT_MAX = 5;
const DEFAULT_RECONNECT_BACKOFF = {
  base: 500,
  max: 15_000,
  factor: 2,
};
const DEFAULT_COMMAND_TIMEOUT_MS = 60_000;

const toError = (error: unknown): Error =>
  error instanceof Error ? error : new Error(String(error));

const generateCommandId = (): string => {
  if (typeof crypto !== "undefined" && "randomUUID" in crypto) {
    return crypto.randomUUID();
  }
  return `cmd_${Date.now().toString(36)}_${Math.random().toString(36).slice(2, 8)}`;
};

export class CommandCancelledError extends Error {
  readonly commandId: string;
  readonly reason: CommandCancelReason;
  readonly result?: CommandResult;

  constructor(
    commandId: string,
    reason: CommandCancelReason,
    result?: CommandResult,
  ) {
    super(`Command ${commandId} cancelled (${reason})`);
    this.name = "CommandCancelledError";
    this.commandId = commandId;
    this.reason = reason;
    this.result = result;
  }
}

type InternalCommandSpec = CommandSpec & { id: string };

type PendingCommand = {
  spec: InternalCommandSpec;
  resolve: (result: CommandResult) => void;
  reject: (reason: unknown) => void;
  queuedAt: Date;
  abortCleanup?: () => void;
};

type ActiveCommand = PendingCommand & {
  stdout: string[];
  stderr: string[];
  startedAt: Date;
  endedAt?: Date;
  channel?: ClientChannel;
  timeoutTimer?: ReturnType<typeof setTimeout>;
  cancellationReason?: CommandCancelReason;
  settleCalled?: boolean;
};

const DEFAULT_CANCEL_REASON: CommandCancelReason = "other";

export class SSHSession {
  readonly vpsId: VPSId;
  readonly options: SSHSessionOptions;

  private readonly eventBus: EventBus;
  private readonly queue: CommandQueue<PendingCommand>;
  private readonly activeCommands = new Map<string, ActiveCommand>();
  private readonly concurrency: number;

  private client: Client | null = null;
  private connectPromise: Promise<void> | null = null;
  private reconnectAttempts = 0;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private disposed = false;
  private currentStatus: SSHSessionStatus = "closed";

  constructor(
    vpsId: VPSId,
    options: SSHSessionOptions,
    eventBus: EventBus,
  ) {
    this.vpsId = vpsId;
    this.options = options;
    this.eventBus = eventBus;
    this.queue = new CommandQueue();
    this.concurrency = Math.max(1, options.queueConcurrency ?? 1);
  }

  get status(): SSHSessionStatus {
    return this.currentStatus;
  }

  async connect(): Promise<void> {
    if (this.disposed) throw new Error("SSH session already disposed");
    if (this.currentStatus === "ready" && this.client) return;
    if (this.connectPromise) return this.connectPromise;

    this.updateStatus("connecting");

    const connectConfig: ConnectConfig = {
      host: this.options.host,
      port: this.options.port ?? 22,
      username: this.options.username,
      privateKey: this.options.privateKey,
      password: this.options.password,
      keepaliveInterval:
        this.options.keepaliveIntervalMs ?? DEFAULT_KEEPALIVE_INTERVAL_MS,
      keepaliveCountMax:
        this.options.keepaliveCountMax ?? DEFAULT_KEEPALIVE_COUNT_MAX,
    };

    this.emitLog("debug", "Attempting SSH connection", {
      host: connectConfig.host,
      port: connectConfig.port,
      username: connectConfig.username,
    });

    this.connectPromise = new Promise<void>((resolve, reject) => {
      const client = new Client();

      const clearConnectPromise = () => {
        this.connectPromise = null;
      };

      const cleanup = () => {
        client.removeListener("ready", onReady);
        client.removeListener("error", onError);
        client.removeListener("close", onClose);
        client.removeListener("end", onEnd);
      };

      const onReady = () => {
        clearTimeout(connectTimeout);
        this.emitLog("info", "SSH session ready", { vpsId: this.vpsId });
        this.client = client;
        this.reconnectAttempts = 0;
        this.updateStatus("ready");
        clearConnectPromise();
        resolve();
        this.drainQueue();
      };

      const onError = (error: Error) => {
        this.emitLog("error", "SSH session error", {
          vpsId: this.vpsId,
          error: error.message,
        });

        if (this.currentStatus === "connecting" && this.connectPromise) {
          cleanup();
          client.destroy();
          clearConnectPromise();
          this.updateStatus("error");
          reject(error);
          this.scheduleReconnect();
          return;
        }

        if (!this.disposed) {
          this.handleConnectionLost(error);
        }
      };

      const handleCloseOrEnd = (hadError: boolean) => {
        cleanup();
        this.emitLog(
          hadError ? "error" : "warn",
          "SSH connection closed",
          { hadError },
        );
        if (this.disposed) {
          this.updateStatus("closed");
          this.cleanupClient(client);
          return;
        }

        this.cleanupClient(client);
        this.updateStatus("degraded");
        this.handleConnectionLost();
        this.scheduleReconnect();
      };

      const onClose = (hadError: boolean) => {
        handleCloseOrEnd(hadError);
      };

      const onEnd = () => {
        handleCloseOrEnd(false);
      };

      const connectTimeout = setTimeout(() => {
        const timeoutErr = new Error("SSH connection timeout");
        onError(timeoutErr);
      }, Math.max(30_000, this.options.defaultTimeoutMs ?? DEFAULT_COMMAND_TIMEOUT_MS));

      client.once("ready", onReady);
      client.on("error", onError);
      client.once("close", onClose);
      client.once("end", onEnd);

      try {
        client.connect(connectConfig);
      } catch (error) {
        cleanup();
        clearTimeout(connectTimeout);
        clearConnectPromise();
        const err = toError(error);
        this.emitLog("error", "Failed to initiate SSH connection", {
          error: err.message,
        });
        reject(err);
        this.scheduleReconnect();
      }
    });

    return this.connectPromise;
  }

  enqueue(spec: CommandSpec): Promise<CommandResult> {
    if (this.disposed) {
      return Promise.reject(new Error("SSH session disposed"));
    }

    const id = spec.id ?? generateCommandId();
    const internalSpec: InternalCommandSpec = { ...spec, id };
    let queued = false;
    let cancelledBeforeQueue: CommandCancelReason | null = null;

    const promise = new Promise<CommandResult>((resolve, reject) => {
      const pending: PendingCommand = {
        spec: internalSpec,
        resolve,
        reject,
        queuedAt: new Date(),
      };

      if (internalSpec.abortSignal) {
        if (internalSpec.abortSignal.aborted) {
          cancelledBeforeQueue = "abort";
          pending.reject(
            new CommandCancelledError(internalSpec.id, "abort"),
          );
          return;
        }
        const abortListener = () => {
          void this.cancel(internalSpec.id, "abort");
        };
        internalSpec.abortSignal.addEventListener("abort", abortListener, {
          once: true,
        });
        pending.abortCleanup = () => {
          internalSpec.abortSignal?.removeEventListener(
            "abort",
            abortListener,
          );
        };
      }

      this.queue.enqueue(pending);
      queued = true;
    });

    if (cancelledBeforeQueue) {
      this.eventBus.emit("command:cancelled", {
        vpsId: this.vpsId,
        commandId: id,
        reason: cancelledBeforeQueue,
      });
      return promise;
    }

    if (queued) {
      this.eventBus.emit("command:queued", {
        vpsId: this.vpsId,
        commandId: id,
        command: internalSpec.command,
      });

      this.ensureConnection();
      this.drainQueue();
    }

    return promise;
  }

  async cancel(
    commandId: string,
    reason: CommandCancelReason = DEFAULT_CANCEL_REASON,
  ): Promise<void> {
    const pending = this.queue.remove((entry) => entry.spec.id === commandId);
    if (pending) {
      pending.abortCleanup?.();
      const error = new CommandCancelledError(commandId, reason);
      this.eventBus.emit("command:cancelled", {
        vpsId: this.vpsId,
        commandId,
        reason,
      });
      pending.reject(error);
      return;
    }

    const active = this.activeCommands.get(commandId);
    if (!active) {
      return;
    }

    active.cancellationReason = reason;
    active.stderr.push(`Command cancelled (${reason}).`);
    this.eventBus.emit("command:cancelled", {
      vpsId: this.vpsId,
      commandId,
      reason,
    });
    this.emitLog("info", "Cancelling running command", {
      commandId,
      reason,
    });

    this.applyCancellationSignal(active);
  }

  async dispose(): Promise<void> {
    if (this.disposed) return;
    this.disposed = true;

    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }

    this.emitLog("info", "Disposing SSH session", { vpsId: this.vpsId });

    const pending = this.queue.clear();
    for (const task of pending) {
      task.abortCleanup?.();
      const error = new CommandCancelledError(
        task.spec.id,
        "session-disposed",
      );
      this.eventBus.emit("command:cancelled", {
        vpsId: this.vpsId,
        commandId: task.spec.id,
        reason: "session-disposed",
      });
      task.reject(error);
    }

    const running = Array.from(this.activeCommands.values());
    for (const command of running) {
      command.cancellationReason = "session-disposed";
      command.stderr.push("Session disposed.");
      this.eventBus.emit("command:cancelled", {
        vpsId: this.vpsId,
        commandId: command.spec.id,
        reason: "session-disposed",
      });
      this.finalizeCommand(command, { code: null });
    }

    await new Promise<void>((resolve) => {
      if (!this.client) {
        resolve();
        return;
      }

      const client = this.client;
      const done = () => {
        this.cleanupClient(client);
        resolve();
      };

      client.once("close", done);
      try {
        client.end();
      } catch {
        client.destroy();
      }
    });

    this.updateStatus("closed");
  }

  on<E extends SSHEventName>(
    eventName: E,
    handler: (payload: SSHEventMap[E]) => void,
  ): () => void {
    if (!SSH_EVENT_NAMES.includes(eventName)) {
      throw new Error(`Unsupported SSH event: ${eventName}`);
    }
    const wrapped = (payload: unknown) => {
      const eventPayload = payload as { vpsId?: VPSId };
      if (!eventPayload || eventPayload.vpsId !== this.vpsId) return;
      handler(payload as SSHEventMap[E]);
    };
    this.eventBus.on(eventName, wrapped as (...args: unknown[]) => void);
    return () => {
      this.eventBus.off(eventName, wrapped as (...args: unknown[]) => void);
    };
  }

  private ensureConnection(): void {
    if (this.client || this.connectPromise || this.disposed) return;
    void this.connect().catch((error) => {
      this.emitLog("error", "Initial connection failed", {
        error: toError(error).message,
      });
    });
  }

  private drainQueue(): void {
    if (this.disposed) return;
    if (this.currentStatus !== "ready") return;
    if (!this.client) return;

    while (
      this.activeCommands.size < this.concurrency &&
      !this.queue.isEmpty()
    ) {
      const pending = this.queue.dequeue();
      if (!pending) break;
      this.startCommand(pending);
    }
  }

  private startCommand(pending: PendingCommand): void {
    const client = this.client;
    if (!client) {
      this.queue.enqueue(pending);
      this.ensureConnection();
      return;
    }

    const active: ActiveCommand = {
      ...pending,
      stdout: [],
      stderr: [],
      startedAt: new Date(),
    };

    this.activeCommands.set(active.spec.id, active);

    this.eventBus.emit("command:start", {
      vpsId: this.vpsId,
      commandId: active.spec.id,
      command: active.spec.command,
    });

    const commandToExecute = this.buildCommand(active.spec);
    const timeoutMs =
      active.spec.timeoutMs ?? this.options.defaultTimeoutMs ?? DEFAULT_COMMAND_TIMEOUT_MS;

    if (timeoutMs > 0 && Number.isFinite(timeoutMs)) {
      active.timeoutTimer = setTimeout(() => {
        void this.cancel(active.spec.id, "timeout");
      }, timeoutMs);
    }

    const runCommand = () => {
      try {
        client.exec(
          commandToExecute,
          { env: active.spec.env },
          (err, channel) => {
            if (err) {
              this.activeCommands.delete(active.spec.id);
              active.abortCleanup?.();
              if (active.timeoutTimer) clearTimeout(active.timeoutTimer);
              const error = toError(err);
              this.emitLog("error", "Failed to execute command", {
                commandId: active.spec.id,
                error: error.message,
              });

              const result: CommandResult = {
                id: active.spec.id,
                code: null,
                stdout: "",
                stderr: error.message,
                startedAt: active.startedAt,
                endedAt: new Date(),
              };

              this.eventBus.emit("command:end", {
                vpsId: this.vpsId,
                commandId: active.spec.id,
                result,
              });
              active.reject(error);
              this.drainQueue();
              return;
            }

            active.channel = channel;
            this.attachCommandHandlers(active, channel);
            if (active.cancellationReason) {
              this.applyCancellationSignal(active);
            }
          },
        );
      } catch (error) {
        this.activeCommands.delete(active.spec.id);
        active.abortCleanup?.();
        if (active.timeoutTimer) clearTimeout(active.timeoutTimer);
        const err = toError(error);
        this.emitLog("error", "Unexpected error executing command", {
          commandId: active.spec.id,
          error: err.message,
        });
        active.reject(err);
        this.drainQueue();
      }
    };

    runCommand();
  }

  private attachCommandHandlers(
    active: ActiveCommand,
    channel: ClientChannel,
  ): void {
    channel.setEncoding?.("utf8");
    channel.stderr?.setEncoding?.("utf8");

    channel.on("data", (chunk: Buffer | string) => {
      const text = typeof chunk === "string" ? chunk : chunk.toString("utf8");
      active.stdout.push(text);
      this.eventBus.emit("command:stdout", {
        vpsId: this.vpsId,
        commandId: active.spec.id,
        chunk: text,
      });
    });

    channel.stderr.on("data", (chunk: Buffer | string) => {
      const text = typeof chunk === "string" ? chunk : chunk.toString("utf8");
      active.stderr.push(text);
      this.eventBus.emit("command:stderr", {
        vpsId: this.vpsId,
        commandId: active.spec.id,
        chunk: text,
      });
    });

    channel.on("error", (error) => {
      const err = toError(error);
      active.stderr.push(err.message);
      this.emitLog("error", "Command channel error", {
        commandId: active.spec.id,
        error: err.message,
      });
    });

    let exitCode: number | null = null;
    let exitSignal: string | null = null;

    channel.on("exit", (code: number | null, signal: string | null) => {
      exitCode = code;
      exitSignal = signal;
    });

    channel.on("close", () => {
      this.finalizeCommand(active, { code: exitCode, signal: exitSignal });
    });
  }

  private buildCommand(spec: InternalCommandSpec): string {
    if (spec.cwd) {
      return `cd ${JSON.stringify(spec.cwd)} && ${spec.command}`;
    }
    return spec.command;
  }

  private finalizeCommand(
    active: ActiveCommand,
    outcome: {
      code: number | null;
      signal?: string | null;
      error?: Error;
    },
  ): void {
    if (active.settleCalled) return;
    active.settleCalled = true;

    this.activeCommands.delete(active.spec.id);

    active.abortCleanup?.();
    if (active.timeoutTimer) {
      clearTimeout(active.timeoutTimer);
      active.timeoutTimer = undefined;
    }

    active.endedAt = new Date();

    const result: CommandResult = {
      id: active.spec.id,
      code: outcome.code ?? null,
      stdout: active.stdout.join(""),
      stderr: active.stderr.join(""),
      startedAt: active.startedAt,
      endedAt: active.endedAt,
    };

    if (active.cancellationReason) {
      if (!result.stderr) {
        result.stderr = `Command cancelled (${active.cancellationReason}).`;
      }
      const error = new CommandCancelledError(
        active.spec.id,
        active.cancellationReason,
        result,
      );
      this.eventBus.emit("command:end", {
        vpsId: this.vpsId,
        commandId: active.spec.id,
        result,
      });
      active.reject(error);
    } else if (outcome.error) {
      const err = toError(outcome.error);
      result.stderr += err.message;
      this.eventBus.emit("command:end", {
        vpsId: this.vpsId,
        commandId: active.spec.id,
        result,
      });
      active.reject(err);
    } else {
      this.eventBus.emit("command:end", {
        vpsId: this.vpsId,
        commandId: active.spec.id,
        result,
      });
      active.resolve(result);
    }

    this.drainQueue();
  }

  private applyCancellationSignal(active: ActiveCommand): void {
    if (!active.channel) return;

    try {
      active.channel.signal?.("INT");
    } catch (error) {
      this.emitLog("debug", "Failed to send SIGINT to command", {
        commandId: active.spec.id,
        error: toError(error).message,
      });
    }

    setTimeout(() => {
      try {
        if (!active.channel?.destroyed) {
          active.channel?.close();
        }
      } catch (error) {
        this.emitLog("debug", "Failed to close command channel", {
          commandId: active.spec.id,
          error: toError(error).message,
        });
      }
    }, 250);
  }

  private scheduleReconnect(): void {
    if (this.disposed) return;
    if (this.reconnectTimer) return;

    const backoff = this.options.reconnectBackoffMs ?? DEFAULT_RECONNECT_BACKOFF;
    const delay = Math.min(
      backoff.base * Math.pow(backoff.factor, this.reconnectAttempts),
      backoff.max,
    );

    this.reconnectAttempts += 1;

    this.emitLog("warn", "Scheduling reconnect attempt", {
      attempt: this.reconnectAttempts,
      delay,
    });

    this.reconnectTimer = setTimeout(() => {
      this.reconnectTimer = null;
      if (this.disposed) return;
      this.emitLog("info", "Attempting SSH reconnect", {
        attempt: this.reconnectAttempts,
      });

      void this.connect().catch((error) => {
        this.emitLog("error", "Reconnect attempt failed", {
          attempt: this.reconnectAttempts,
          error: toError(error).message,
        });
        this.updateStatus("error");
        this.scheduleReconnect();
      });
    }, delay);
  }

  private handleConnectionLost(error?: Error): void {
    const message = error ? error.message : "Connection lost";
    for (const active of Array.from(this.activeCommands.values())) {
      active.cancellationReason = "connection-lost";
      active.stderr.push(message);
      this.eventBus.emit("command:cancelled", {
        vpsId: this.vpsId,
        commandId: active.spec.id,
        reason: "connection-lost",
      });
      this.finalizeCommand(active, { code: null, error });
    }
  }

  private cleanupClient(client: Client): void {
    if (this.client === client) {
      this.client = null;
    }
    client.removeAllListeners();
  }

  private updateStatus(status: SSHSessionStatus): void {
    if (this.currentStatus === status) return;
    this.currentStatus = status;
    this.eventBus.emit("session:status", { vpsId: this.vpsId, status });
  }

  private emitLog(
    level: SSHLogLevel,
    message: string,
    meta?: Record<string, unknown>,
  ): void {
    this.eventBus.emit("session:log", {
      vpsId: this.vpsId,
      level,
      message,
      meta,
    });
  }
}

export default SSHSession;
