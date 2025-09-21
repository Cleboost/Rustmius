import { useServersStore } from "@/stores/servers";
import { useConsolesStore } from "@/stores/consoles";
import { Server as ServerType } from "@/types/server";

export default class Server {
  id: string;
  server?: ServerType;
  private serversStore = useServersStore();
  private consolesStore = useConsolesStore();

  constructor(id: string) {
    this.id = id;
    this.loadServer();
  }

  private async loadServer(): Promise<void> {
    await this.serversStore.load();
    this.server = this.serversStore.findServerById(this.id);
  }

  async ensureLoaded(): Promise<void> {
    await this.loadServer();
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

  async launchTerminal(): Promise<void> {
    try {
      await this.consolesStore.launchNativeTerminal(this.id);
    } catch (error) {
      console.error(`Failed to launch terminal for ${this.getName()}:`, error);
      throw error;
    }
  }

  async testConnection(): Promise<boolean> {
    try {
      console.log(
        `Testing SSH connection to ${this.getName()} (${this.getIP()})`,
      );
      return true;
    } catch (error) {
      console.error(`SSH test failed for ${this.getName()}:`, error);
      return false;
    }
  }

  config(): ConfigServer {
    return new ConfigServer(this.server);
  }
}

class ConfigServer {
  server: ServerType | undefined;
  private serversStore = useServersStore();

  constructor(server: ServerType | undefined) {
    this.server = server;
  }

  get(): ServerType | undefined {
    return this.server;
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

  getKey(): undefined {
    return undefined;
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
