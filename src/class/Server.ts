import { useServerConfigStore } from "@/stores/servers";
import { Server as ServerType } from "@/types/server";
import { Command } from "@tauri-apps/plugin-shell";
import Key from "./Class";

export default class Server {
  public readonly id: ServerType["id"];
  configClass: ConfigServer;
  console: ServerConsole;
  docker: Docker;
  systemMonitor: SystemMonitor;
  readonly serversStore = useServerConfigStore();

  constructor(id: string) {
    this.id = id;
    this.configClass = new ConfigServer(this.serversStore.getServer(id));
    this.console = new ServerConsole(this);
    this.docker = new Docker(this);
    this.systemMonitor = new SystemMonitor(this);
  }

  config(): ConfigServer {
    return this.configClass;
  }
}

export class ConfigServer {
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
}

class Docker {
  readonly server: Server;

  constructor(server: Server) {
    this.server = server;
  }

  async getVersion(): Promise<string> {
    try {
      const output = await this.server.console.execute("docker version");
      const lines = output.split("\n");
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

      const lines = output.split("\n");
      let currentSection = "";
      const containers: string[] = [];
      const images: string[] = [];
      const dangling: string[] = [];
      let size = "0B";

      for (const line of lines) {
        const trimmedLine = line.trim();

        if (trimmedLine === "===CONTAINERS===") {
          currentSection = "containers";
          continue;
        } else if (trimmedLine === "===IMAGES===") {
          currentSection = "images";
          continue;
        } else if (trimmedLine === "===DANGLING===") {
          currentSection = "dangling";
          continue;
        } else if (trimmedLine === "===SIZE===") {
          currentSection = "size";
          continue;
        }

        if (trimmedLine && !trimmedLine.includes("REPOSITORY")) {
          switch (currentSection) {
            case "containers":
              containers.push(trimmedLine);
              break;
            case "images":
              images.push(trimmedLine);
              break;
            case "dangling":
              dangling.push(trimmedLine);
              break;
            case "size":
              if (trimmedLine.includes("Images")) {
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
        if (container.includes("Up")) {
          running++;
        } else {
          stopped++;
        }
      }

      return {
        containers: {
          running,
          stopped,
          total: running + stopped,
        },
        images: {
          local: images.length,
          size,
          dangling: dangling.length,
        },
      };
    } catch (error) {
      console.error("Error retrieving Docker data:", error);
      return {
        containers: { running: 0, stopped: 0, total: 0 },
        images: { local: 0, size: "0B", dangling: 0 },
      };
    }
  }
}

class SystemMonitor {
  readonly server: Server;

  constructor(server: Server) {
    this.server = server;
  }

  async getSystemInfo(): Promise<{
    hostname: string;
    uptime: string;
    os: string;
    kernel: string;
    architecture: string;
  }> {
    try {
      const commands = [
        "hostname",
        "uptime -p",
        "uname -s",
        "uname -r",
        "uname -m",
      ];

      const results = await Promise.all(
        commands.map((cmd) => this.server.console.execute(cmd)),
      );

      return {
        hostname: results[0].trim(),
        uptime: results[1].trim(),
        os: results[2].trim(),
        kernel: results[3].trim(),
        architecture: results[4].trim(),
      };
    } catch (error) {
      console.error("Error getting system info:", error);
      return {
        hostname: "Unknown",
        uptime: "Unknown",
        os: "Unknown",
        kernel: "Unknown",
        architecture: "Unknown",
      };
    }
  }

  async getSystemStats(): Promise<{
    cpu: {
      usage: number;
      cores: number;
      load: number[];
    };
    memory: {
      total: number;
      used: number;
      free: number;
      cached: number;
      percentage: number;
    };
    disk: {
      total: number;
      used: number;
      free: number;
      percentage: number;
    };
    network: {
      rx: number;
      tx: number;
      rxRate: number;
      txRate: number;
    };
  }> {
    try {
      const command = `
        echo "===CPU==="
        # CPU usage from /proc/stat
        head -1 /proc/stat | awk '{idle=\$5+\$6; total=\$2+\$3+\$4+\$5+\$6+\$7+\$8; print (1-idle/total)*100}'
        nproc
        uptime | awk -F'load average:' '{print \$2}' | awk '{print \$1","\$2","\$3}' | sed 's/,/ /g'
        echo "===MEMORY==="
        free -m | grep Mem | awk '{print \$2","\$3","\$4","\$6}'
        echo "===DISK==="
        df / | tail -1 | awk '{total=\$2/1024/1024; used=\$3/1024/1024; free=\$4/1024/1024; print total","used","free}'
        echo "===NETWORK==="
        # Get network stats from first active interface
        cat /proc/net/dev | grep -E "(eth|ens|enp|wlan|wlp)" | head -1 | awk '{print \$2","\$10}' || echo "0,0"
      `;

      const output = await this.server.console.execute(command);
      console.log("System stats command output:", output); // Debug log

      const lines = output.split("\n");
      let currentSection = "";
      const cpuData: string[] = [];
      const memoryData: string[] = [];
      const diskData: string[] = [];
      const networkData: string[] = [];

      for (const line of lines) {
        const trimmedLine = line.trim();

        if (trimmedLine === "===CPU===") {
          currentSection = "cpu";
          continue;
        } else if (trimmedLine === "===MEMORY===") {
          currentSection = "memory";
          continue;
        } else if (trimmedLine === "===DISK===") {
          currentSection = "disk";
          continue;
        } else if (trimmedLine === "===NETWORK===") {
          currentSection = "network";
          continue;
        }

        if (trimmedLine && !trimmedLine.includes("===")) {
          switch (currentSection) {
            case "cpu":
              cpuData.push(trimmedLine);
              break;
            case "memory":
              memoryData.push(trimmedLine);
              break;
            case "disk":
              diskData.push(trimmedLine);
              break;
            case "network":
              networkData.push(trimmedLine);
              break;
          }
        }
      }

      const cpuUsage = parseFloat(cpuData[0] || "0");
      const cores = parseInt(cpuData[1] || "1");
      const loadAvgStr = cpuData[2] || "0 0 0";
      const loadAvg = loadAvgStr
        .split(" ")
        .map(parseFloat)
        .filter((n) => !isNaN(n));

      const memoryStr = memoryData[0] || "0,0,0,0";
      const memoryValues = memoryStr.split(",").map(Number);
      const [totalMB, usedMB, freeMB, cachedMB] = memoryValues;
      const memoryPercentage = totalMB > 0 ? (usedMB / totalMB) * 100 : 0;

      const diskStr = diskData[0] || "0,0,0";
      const diskValues = diskStr.split(",").map(Number);
      const [totalGB, usedGB, freeGB] = diskValues;
      const diskPercentage = totalGB > 0 ? (usedGB / totalGB) * 100 : 0;

      const networkStr = networkData[0] || "0,0";
      const networkValues = networkStr.split(",").map((val) => {
        const num = Number(val);
        return isNaN(num) ? 0 : num;
      });
      const [rxBytes, txBytes] = networkValues;

      return {
        cpu: {
          usage: cpuUsage,
          cores: cores,
          load: loadAvg.length >= 3 ? loadAvg : [0, 0, 0],
        },
        memory: {
          total: totalMB,
          used: usedMB,
          free: freeMB,
          cached: cachedMB,
          percentage: memoryPercentage,
        },
        disk: {
          total: totalGB,
          used: usedGB,
          free: freeGB,
          percentage: diskPercentage,
        },
        network: {
          rx: rxBytes,
          tx: txBytes,
          rxRate: 0,
          txRate: 0,
        },
      };
    } catch (error) {
      console.error("Error getting system stats:", error);
      return {
        cpu: { usage: 0, cores: 1, load: [0, 0, 0] },
        memory: { total: 0, used: 0, free: 0, cached: 0, percentage: 0 },
        disk: { total: 0, used: 0, free: 0, percentage: 0 },
        network: { rx: 0, tx: 0, rxRate: 0, txRate: 0 },
      };
    }
  }
}

export type { ConfigServer, Docker, SystemMonitor };
