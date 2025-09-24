import { useServerConfigStore } from "@/stores/servers";
import { Server as ServerType } from "@/types/server";
import { Command } from "@tauri-apps/plugin-shell";
import Key from "./Class";

export default class Server {
  public readonly id: ServerType["id"];
  configClass: ConfigServer | undefined;
  console: ServerConsole | undefined;
  readonly serversStore = useServerConfigStore();

  constructor(id: string) {
    this.id = id;
    this.configClass = new ConfigServer(this.serversStore.getServer(id));
    this.console = new ServerConsole(this);
  }

  config(): ConfigServer {
    return this.configClass;
  }
}

export class ConfigServer {
  readonly server: ServerType;
  readonly serversStore = useServerConfigStore();

  constructor(server: ServerType | undefined) {
    this.server = server;
  }

  get(): ServerType | undefined {
    return this.server;
  }

  getID(): string {
    return this.server.id;
  }

  getName(): string {
    return this.server?.name || "Unknown";
  }

  getIP(): string {
    return this.server?.ip || "Unknown";
  }

  getKeyID(): number {
    return this.server?.keyID || 0;
  }

  getKey(): Key {
    return new Key(this.getKeyID());
  }

  setName(name: string): void {
    if (this.server) {
      this.server.name = name;
      this.serversStore.updateServer(this.server.id, this.server);
    }
  }

  setIP(ip: string): void {
    if (this.server) {
      this.server.ip = ip;
      this.serversStore.updateServer(this.server.id, this.server);
    }
  }

  setKeyID(keyID: number): void {
    if (this.server) {
      this.server.keyID = keyID;
      this.serversStore.updateServer(this.server.id, this.server);
    }
  }

  isValid(): boolean {
    return !!(this.server?.id && this.server?.name && this.server?.ip);
  }

  getSSHHostname(): string {
    return this.server?.ip || "localhost";
  }

  getSSHUser(): string {
    return "root";
  }

  getSSHPort(): number {
    return 22;
  }

  getSSHIdentityFile(): string | undefined {
    return undefined;
  }
}

export class ServerConsole {
  readonly server: Server;
    
  constructor(server: Server) {
    this.server = server;
  }

  async create(): Promise<void> {
    const sshArgs = ["ssh", "-tt", "-o", "StrictHostKeyChecking=accept-new", "-i", await this.server.config().getKey().getPath(), `root@${this.server.config().getIP()}`];
    const candidates: Array<{ bin: string; args: string[] }> = [
      { bin: "foot", args: ["-e", ...sshArgs] },
      { bin: "alacritty", args: ["-e", ...sshArgs] },
      { bin: "kitty", args: ["-e", ...sshArgs] },
      { bin: "wezterm", args: ["start", "--", ...sshArgs] },
      { bin: "gnome-terminal", args: ["--", ...sshArgs] },
      { bin: "xfce4-terminal", args: ["-e", sshArgs.join(" ")] },
      { bin: "konsole", args: ["-e", ...sshArgs] },
      { bin: "tilix", args: ["-e", ...sshArgs] },
      { bin: "lxterminal", args: ["-e", ...sshArgs] },
      { bin: "xterm", args: ["-e", ...sshArgs] },
      { bin: "footclient", args: ["-e", ...sshArgs] },
    ];

    let lastErr: unknown;
    for (const c of candidates) {
      try {
        console.log(`Trying terminal: ${c.bin} with args:`, c.args);
        await Command.create(c.bin, c.args).spawn();
        return;
      }
      catch (err) {
        lastErr = err;
        console.log(`❌ Failed to open ${c.bin}:`, err);
      }
    }
    console.error("❌ No terminal emulator available. Last error:", lastErr);
    throw lastErr ?? new Error("No terminal emulator available");
  }
}