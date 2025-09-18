import { defineStore } from "pinia";
import { reactive, toRefs } from "vue";
import { Command } from "@tauri-apps/plugin-shell";

export type ConsoleSession = {
  serverId: string;
  output: string;
  connecting: boolean;
  connected: boolean;
  error?: string;
  child?: any; // plugin-shell Child
};

export const useConsolesStore = defineStore("consoles", () => {
  const sessions = reactive<Record<string, ConsoleSession>>({});

  async function openConsole(serverId: string): Promise<void> {
    if (sessions[serverId]?.child) return;
    const session: ConsoleSession = {
      serverId,
      output: "",
      connecting: true,
      connected: false,
    };
    sessions[serverId] = session;
    try {
      const cmd = await Command.create("ssh", [
        "-vv",
        "-tt",
        "-o",
        "StrictHostKeyChecking=accept-new",
        serverId,
      ]);
      const child = await cmd.spawn();
      session.child = child;
      child.stdout.on("data", (l: unknown) => {
        session.output += String(l);
      });
      child.stderr.on("data", (l: unknown) => {
        session.output += String(l);
      });
      child.on("close", ({ code }: { code: number }) => {
        session.output += `\n[exit ${code}]`;
        session.connected = false;
        session.connecting = false;
      });
      // nudge for prompt
      await child.write("\n");
      session.connecting = false;
      session.connected = true;
    } catch (e) {
      session.error = String(e);
      session.connecting = false;
      session.connected = false;
    }
  }

  async function send(serverId: string, input: string): Promise<void> {
    const s = sessions[serverId];
    if (!s?.child) return;
    await s.child.write(input);
  }

  async function closeConsole(serverId: string): Promise<void> {
    const s = sessions[serverId];
    if (!s) return;
    try {
      await s.child?.kill?.();
    } catch {}
    delete sessions[serverId];
  }

  function list(): ConsoleSession[] {
    return Object.values(sessions);
  }

  return { sessions, openConsole, send, closeConsole, list };
});
