import { defineStore } from "pinia";
import { reactive } from "vue";
import { Command } from "@tauri-apps/plugin-shell";

export type ConsoleSession = {
  serverId: string;
  state: SessionState;
  error?: string;
  process?: Command<any>;
};

export type SessionState =
  | "connecting"
  | "connected"
  | "disconnected"
  | "error";

export const useConsolesStore = defineStore("consoles", () => {
  const sessions = reactive<Record<string, ConsoleSession>>({});

  async function openConsole(serverId: string): Promise<void> {
    if (sessions[serverId]) return;
    
    const session: ConsoleSession = {
      serverId,
      state: "connecting",
    };
    sessions[serverId] = session;

    try {
      const testCmd = Command.create("ssh", [
        "-o", "StrictHostKeyChecking=accept-new",
        "-o", "ConnectTimeout=5",
        "-o", "BatchMode=yes",
        serverId,
        "echo 'SSH connection test successful'"
      ]);

      testCmd.on("close", ({ code }) => {
        if (code === 0) {
          session.state = "connected";
        } else {
          session.state = "error";
          session.error = `SSH connection failed with code ${code}`;
        }
      });

      testCmd.on("error", (err) => {
        session.state = "error";
        session.error = String(err);
      });

      await testCmd.spawn();
      
    } catch (e) {
      session.error = String(e);
      session.state = "error";
    }
  }

  async function launchTerminal(serverId: string): Promise<void> {
    const session = sessions[serverId];
    if (!session || session.state !== "connected") {
      throw new Error("Session not ready for terminal launch");
    }
    
    try {
      await launchNativeTerminal(serverId);
    } catch (err) {
      console.error(`[native-terminal-error] ${String(err)}`);
      session.state = "error";
      session.error = String(err);
      throw err;
    }
  }

  async function closeConsole(serverId: string): Promise<void> {
    const session = sessions[serverId];
    if (!session) return;
    
    try {
      if (session.process) {
        await session.process.terminate();
      }
    } catch {}
    
    delete sessions[serverId];
  }

  function list(): ConsoleSession[] {
    return Object.values(sessions);
  }

  async function launchNativeTerminal(id: string): Promise<void> {
    console.log(`Attempting to launch native terminal for server: ${id}`);
    const sshArgs = ["ssh", "-tt", "-o", "StrictHostKeyChecking=accept-new", id];
    await launchNativeTerminalWithArgs(sshArgs);
  }

  async function launchNativeTerminalWithArgs(sshArgs: string[]): Promise<void> {
    console.log(`Attempting to launch native terminal with SSH args:`, sshArgs);
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
      { bin: "footclient", args: ["-e", ...sshArgs] }, // Keep as fallback
    ];
    
    let lastErr: unknown;
    for (const c of candidates) {
      try {
        console.log(`Trying terminal: ${c.bin} with args:`, c.args);
        const proc = await Command.create(c.bin, c.args);
        
        proc.on("close", ({ code }) => {
          console.log(`Terminal ${c.bin} closed with code: ${code}`);
        });
        
        proc.on("error", (err) => {
          console.log(`Terminal ${c.bin} error:`, err);
        });
        
        await proc.spawn();
        console.log(`✅ Successfully opened terminal: ${c.bin}`);
        return;
      } catch (e) {
        lastErr = e;
        console.log(`❌ Failed to open ${c.bin}:`, e);
      }
    }
    console.error("❌ No terminal emulator available. Last error:", lastErr);
    throw lastErr ?? new Error("No terminal emulator available");
  }

  return { sessions, openConsole, launchTerminal, launchNativeTerminal, launchNativeTerminalWithArgs, closeConsole, list };
});
