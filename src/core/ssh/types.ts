export type VPSId = string;

export type CommandSpec = {
  id?: string;
  command: string;
  cwd?: string;
  env?: Record<string, string>;
  timeoutMs?: number;
  abortSignal?: AbortSignal;
};

export type CommandResult = {
  id: string;
  code: number | null;
  stdout: string;
  stderr: string;
  startedAt: Date;
  endedAt: Date;
};

export type SSHSessionStatus =
  | "connecting"
  | "ready"
  | "degraded"
  | "closed"
  | "error";

export type SSHLogLevel = "info" | "warn" | "error" | "debug";

export type ReconnectBackoffOptions = {
  base: number;
  max: number;
  factor: number;
};

export type SSHSessionOptions = {
  host: string;
  port?: number;
  username: string;
  privateKey?: string | Buffer;
  password?: string;
  keepaliveIntervalMs?: number;
  keepaliveCountMax?: number;
  reconnectBackoffMs?: ReconnectBackoffOptions;
  queueConcurrency?: number;
  defaultTimeoutMs?: number;
};

export type SessionStatusEvent = {
  vpsId: VPSId;
  status: SSHSessionStatus;
};

export type SessionLogEvent = {
  vpsId: VPSId;
  level: SSHLogLevel;
  message: string;
  meta?: Record<string, unknown>;
};

export type CommandQueuedEvent = {
  vpsId: VPSId;
  commandId: string;
  command: string;
};

export type CommandStartEvent = {
  vpsId: VPSId;
  commandId: string;
  command: string;
};

export type CommandOutputEvent = {
  vpsId: VPSId;
  commandId: string;
  chunk: string;
};

export type CommandEndEvent = {
  vpsId: VPSId;
  commandId: string;
  result: CommandResult;
};

export type CommandCancelledEvent = {
  vpsId: VPSId;
  commandId: string;
  reason: string;
};

export type SSHEventMap = {
  "session:status": SessionStatusEvent;
  "session:log": SessionLogEvent;
  "command:queued": CommandQueuedEvent;
  "command:start": CommandStartEvent;
  "command:stdout": CommandOutputEvent;
  "command:stderr": CommandOutputEvent;
  "command:end": CommandEndEvent;
  "command:cancelled": CommandCancelledEvent;
};

export type SSHEventName = keyof SSHEventMap;

export const SSH_EVENT_NAMES: ReadonlyArray<SSHEventName> = [
  "session:status",
  "session:log",
  "command:queued",
  "command:start",
  "command:stdout",
  "command:stderr",
  "command:end",
  "command:cancelled",
];

export type SSHSubscribeFilter = {
  vpsId?: VPSId;
  commandId?: string;
  event?: string;
};

export type SSHEventEnvelope<E extends SSHEventName = SSHEventName> = {
  type: E;
  payload: SSHEventMap[E];
};

export type CommandCancelReason =
  | "user"
  | "timeout"
  | "abort"
  | "session-disposed"
  | "connection-lost"
  | "other";
