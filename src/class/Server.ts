import { useServerConfigStore } from "@/stores/servers";
import { Server as ServerType } from "@/types/server";
import { Command } from "@tauri-apps/plugin-shell";
import Key from "./Class";

export default class Server {
  public readonly id: ServerType["id"];
  configClass: ConfigServer;
  console: ServerConsole;
  docker: Docker;
  readonly serversStore = useServerConfigStore();

  constructor(id: string) {
    this.id = id;
    this.configClass = new ConfigServer(this.serversStore.getServer(id));
    this.console = new ServerConsole(this);
    this.docker = new Docker(this);
  }

  config(): ConfigServer {
    return this.configClass;
  }
}

class ConfigServer {
  server: ServerType;
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

  update(server: ServerType): void {
    this.server = { ...this.server, ...server };
    this.serversStore.updateServer(this.server.id, this.server);
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
    const sshArgs = [
      "ssh",
      "-tt",
      "-o",
      "StrictHostKeyChecking=accept-new",
      "-i",
      await this.server.config().getKey().getPath(),
      `root@${this.server.config().getIP()}`,
    ];
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
      } catch (err) {
        lastErr = err;
        console.log(`❌ Failed to open ${c.bin}:`, err);
      }
    }
    console.error("❌ No terminal emulator available. Last error:", lastErr);
    throw lastErr ?? new Error("No terminal emulator available");
  }

  async execute(command: string): Promise<string> {
    const child = Command.create("ssh", ["-tt", "-o", "StrictHostKeyChecking=accept-new", "-i", await this.server.config().getKey().getPath(), `root@${this.server.config().getIP()}`, command]);
    
    return new Promise((resolve, reject) => {
      let stdout = "";
      let stderr = "";
      
      child.on("close", (data) => {
        if (data.code === 0) {
          resolve(stdout);
        } else {
          if (stderr.includes("No route to host")) {
            reject(new Error("Cannot connect to server: No route to host"));
          } else if (stderr.includes("Connection refused")) {
            reject(new Error("Cannot connect to server: Connection refused"));
          } else if (stderr.includes("Permission denied")) {
            reject(new Error("Cannot connect to server: Permission denied"));
          } else if (stderr.includes("Host key verification failed")) {
            reject(new Error("Cannot connect to server: Host key verification failed"));
          } else {
            reject(new Error(`Command failed with code ${data.code}: ${stderr}`));
          }
        }
      });
      
      child.on("error", (error) => {
        reject(new Error(`Error executing command: ${error}`));
      });
      
      child.stdout.on("data", (data) => {
        stdout += data;
      });
      
      child.stderr.on("data", (data) => {
        stderr += data;
      });
      
      child.spawn();
    });
  }
}

class Docker {
  readonly server: Server;

  constructor(server: Server) {
    this.server = server;
  }
  
  async getVersion(): Promise<string> {
    try {
      const output = await this.server.console.execute("docker version");
      const lines = output.split('\n');
      let foundCommunity = false;
      let version = null;

      for (let i = 0; i < lines.length; i++) {
        if (!foundCommunity && lines[i].includes("Docker Engine - Community")) {
          foundCommunity = true;
          for (let j = i + 1; j < lines.length; j++) {
            const versionMatch = lines[j].match(/Version:\s*([^\s]+)/);
            if (versionMatch) {
              version = versionMatch[1];
              break;
            }
          }
          break;
        }
      }

      if (foundCommunity && version) {
        return `Community Version ${version}`;
      } else {
        return "Unknown version";
      }
    } catch (error) {
      console.error("Error retrieving Docker version:", error);
      if (error instanceof Error) {
        return `Error: ${error.message}`;
      }
      return "Connection error";
    }
  }

  async getAllDockerData(): Promise<{
    containers: {
      running: number;
      stopped: number;
      total: number;
    };
    images: {
      local: number;
      size: string;
      dangling: number;
    };
  }> {
    try {
      const command = `echo "===CONTAINERS===" && docker ps -a --format "{{.Status}}" && echo "===IMAGES===" && docker images --format "{{.Repository}}" && echo "===DANGLING===" && docker images -f dangling=true --format "{{.Repository}}" && echo "===SIZE===" && docker system df`;
      
      const output = await this.server.console.execute(command);
      console.log("Docker command output:", output); // Debug log
      
      const lines = output.split('\n');
      let currentSection = '';
      const containers: string[] = [];
      const images: string[] = [];
      const dangling: string[] = [];
      let size = "0B";
      
      for (const line of lines) {
        const trimmedLine = line.trim();
        
        if (trimmedLine === '===CONTAINERS===') {
          currentSection = 'containers';
          continue;
        } else if (trimmedLine === '===IMAGES===') {
          currentSection = 'images';
          continue;
        } else if (trimmedLine === '===DANGLING===') {
          currentSection = 'dangling';
          continue;
        } else if (trimmedLine === '===SIZE===') {
          currentSection = 'size';
          continue;
        }
        
        if (trimmedLine && !trimmedLine.includes('REPOSITORY')) {
          switch (currentSection) {
            case 'containers':
              containers.push(trimmedLine);
              break;
            case 'images':
              images.push(trimmedLine);
              break;
            case 'dangling':
              dangling.push(trimmedLine);
              break;
            case 'size':
              if (trimmedLine.includes('Images')) {
                const sizeMatch = trimmedLine.match(/(\d+\.?\d*[GMK]?B)/);
                if (sizeMatch) {
                  size = sizeMatch[1];
                }
              }
              break;
          }
        }
      }
      
      let running = 0;
      let stopped = 0;
      for (const container of containers) {
        if (container.includes('Up')) {
          running++;
        } else {
          stopped++;
        }
      }

      return {
        containers: {
          running,
          stopped,
          total: running + stopped
        },
        images: {
          local: images.length,
          size,
          dangling: dangling.length
        }
      };
    } catch (error) {
      console.error("Error retrieving Docker data:", error);
      return {
        containers: { running: 0, stopped: 0, total: 0 },
        images: { local: 0, size: "0B", dangling: 0 }
      };
    }
  }
}

export type { ConfigServer, Docker };