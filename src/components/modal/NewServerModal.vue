<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from "@/components/ui/select";
import { useKeysStore } from "@/stores/keys";
import {
    Server as ServerIcon,
    Key as KeyIcon,
    Network,
    User as UserIcon,
} from "lucide-vue-next";
import { writeTextFile, readTextFile } from "@tauri-apps/plugin-fs";
import { BaseDirectory } from "@tauri-apps/api/path";
import { useServersStore } from "@/stores/servers";
import { Command } from "@tauri-apps/plugin-shell";
import type { KeyPair } from "@/types/key";

const open = defineModel<boolean>("open", { default: false });

const name = ref("");
const username = ref("");
const ip = ref("");

const canSave = computed(
    () => name.value.trim().length > 0 && username.value.trim().length > 0,
);

const keysStore = useKeysStore();
const keys = ref<KeyPair[]>([]);
const selectedKeyId = ref<string | null>(null);
const saving = ref(false);
const logDone = ref(false);
const logLines = ref<string[]>([]);
const askPasswordOpen = ref(false);
const installing = ref(false);
const password = ref("");
const lastHostAlias = ref<string | null>(null);

onMounted(async () => {
    await keysStore.load();
    keys.value = await keysStore.getKeys();
});

const emit = defineEmits<{
    (
        e: "save",
        payload: {
            name: string;
            username: string;
            ip: string;
            keyId: number | null;
        },
    ): void;
    (e: "cancel"): void;
}>();

function onCancel() {
    emit("cancel");
    open.value = false;
}

function onSave() {
    if (!canSave.value) return;
    void saveServer();
}

async function saveServer() {
    saving.value = true;
    logDone.value = false;
    logLines.value = [];
    try {
        const serverId = crypto.randomUUID();
        const hostName = name.value.trim();
        const ipAddr = ip.value.trim();
        const userName = username.value.trim();
        const keyIdNum =
            selectedKeyId.value != null ? Number(selectedKeyId.value) : null;
        lastHostAlias.value = serverId;

        const sshConfigRel = ".ssh/config";
        let content = "";
        try {
            content = await readTextFile(sshConfigRel, {
                baseDir: BaseDirectory.Home,
            });
        } catch {}
        const newBlock = [
            `Host ${serverId}`,
            ipAddr ? `  Hostname ${ipAddr}` : undefined,
            `  User ${userName}`,
            keyIdNum
                ? `  IdentityFile ${await keyPathFromId(keyIdNum)}`
                : undefined,
        ]
            .filter(Boolean)
            .join("\n");
        const newContent = content
            ? `${content.trim()}\n\n${newBlock}\n`
            : `${newBlock}\n`;
        await writeTextFile(sshConfigRel, newContent, {
            baseDir: BaseDirectory.Home,
        });

        const serversStore = useServersStore();
        await serversStore.addServer(
            {
                id: serverId,
                name: hostName,
                ip: ipAddr,
                keyID: keyIdNum ?? 0,
            } as any,
            "/",
        );

        const status = await testConnection(userName, serverId);
        if (status !== 0) {
            askPasswordOpen.value = true;
        } else {
            logDone.value = true;
        }

        emit("save", {
            name: hostName,
            username: userName,
            ip: ipAddr,
            keyId: keyIdNum,
        });
        open.value = false;
    } catch (e) {
        console.error("[server] save error", e);
        logLines.value.push(
            `[exception] ${(e as any)?.toString?.() ?? "error"}`,
        );
    } finally {
        if (!askPasswordOpen.value) logDone.value = true;
    }
}

async function keyPathFromId(id: number): Promise<string | undefined> {
    const ks = useKeysStore();
    await ks.load();
    const all = await ks.getKeys();
    return all.find((k) => k.id === id)?.private;
}

async function keyPubPathFromId(id: number): Promise<string | undefined> {
    const priv = await keyPathFromId(id);
    if (!priv) return undefined;
    const pub = `${priv}.pub`;
    return pub;
}

async function testConnection(
    userName: string,
    hostAlias: string,
): Promise<number> {
    const sshCmd = Command.create("ssh", [
        "-vv",
        "-o",
        "StrictHostKeyChecking=accept-new",
        "-o",
        "ConnectTimeout=8",
        "-o",
        "BatchMode=yes",
        "-l",
        userName,
        hostAlias,
        "exit",
    ]);
    sshCmd.on("close", ({ code }) => {
        logLines.value.push(`[exit] code=${code}`);
    });
    sshCmd.on("error", (err) => {
        logLines.value.push(`[error] ${String(err)}`);
    });
    sshCmd.stdout.on("data", (line) => {
        logLines.value.push(`[out] ${String(line).trimEnd()}`);
    });
    sshCmd.stderr.on("data", (line) => {
        const text = String(line);
        logLines.value.push(`[err] ${text.trimEnd()}`);
        if (/password:/i.test(text)) {
        }
    });
    const status = await sshCmd.execute();
    if (status.stdout) logLines.value.push(`[out] ${status.stdout.trimEnd()}`);
    if (status.stderr) logLines.value.push(`[err] ${status.stderr.trimEnd()}`);
    return status.code;
}

async function onInstallKey(userName: string, hostAlias: string) {
    if (!selectedKeyId.value || !hostAlias) {
        askPasswordOpen.value = false;
        return;
    }
    installing.value = true;
    try {
        const keyIdNum = Number(selectedKeyId.value);
        const pub = await keyPubPathFromId(keyIdNum);
        const priv = await keyPathFromId(keyIdNum);
        const keyArg = pub ?? priv;
        if (!keyArg) return;
        const proc = await Command.create("sshpass", [
            "-p",
            password.value,
            "ssh-copy-id",
            "-o",
            "StrictHostKeyChecking=accept-new",
            "-i",
            keyArg,
            `${userName}@${hostAlias}`,
        ]);
        proc.stdout.on("data", (l) =>
            logLines.value.push(`[copy] ${String(l).trimEnd()}`),
        );
        proc.stderr.on("data", (l) =>
            logLines.value.push(`[copy-err] ${String(l).trimEnd()}`),
        );
        const r = await proc.execute();
        if (r.stdout) logLines.value.push(`[copy-out] ${r.stdout.trimEnd()}`);
        if (r.stderr) logLines.value.push(`[copy-err] ${r.stderr.trimEnd()}`);
        logLines.value.push(`[copy-exit] code=${r.code}`);
        askPasswordOpen.value = false;
        const code = await testConnection(userName, hostAlias);
        logDone.value = true;
    } catch (e) {
        logLines.value.push(
            `[install-exception] ${(e as any)?.toString?.() ?? "error"}`,
        );
    } finally {
        installing.value = false;
        password.value = "";
    }
}
</script>

<template>
    <Dialog v-model:open="open">
        <DialogContent>
            <DialogHeader>
                <DialogTitle>New server</DialogTitle>
                <DialogDescription
                    >Provide basic information for the new
                    server.</DialogDescription
                >
            </DialogHeader>

            <div class="grid gap-4 py-2">
                <div class="grid gap-2">
                    <label
                        class="text-sm font-medium flex items-center gap-2"
                        for="server-name"
                    >
                        <ServerIcon class="size-4 opacity-60" /> Name
                    </label>
                    <Input
                        id="server-name"
                        v-model="name"
                        placeholder="e.g. staging-app"
                    />
                </div>
                <div class="grid gap-2">
                    <label
                        class="text-sm font-medium flex items-center gap-2"
                        for="server-user"
                    >
                        <UserIcon class="size-4 opacity-60" /> Username
                    </label>
                    <Input
                        id="server-user"
                        v-model="username"
                        placeholder="e.g. root"
                    />
                </div>
                <div class="grid gap-2">
                    <label
                        class="text-sm font-medium flex items-center gap-2"
                        for="server-ip"
                    >
                        <Network class="size-4 opacity-60" /> IP
                    </label>
                    <Input
                        id="server-ip"
                        v-model="ip"
                        placeholder="e.g. 192.168.1.10"
                    />
                </div>
                <div class="grid gap-2">
                    <label
                        class="text-sm font-medium flex items-center gap-2"
                        for="server-key"
                    >
                        <KeyIcon class="size-4 opacity-60" /> SSH key
                    </label>
                    <Select v-model="selectedKeyId">
                        <SelectTrigger id="server-key">
                            <SelectValue
                                :placeholder="
                                    keys.length
                                        ? 'Select a key'
                                        : 'No keys found'
                                "
                            />
                        </SelectTrigger>
                        <SelectContent>
                            <template v-if="keys.length">
                                <SelectItem
                                    v-for="k in keys"
                                    :key="k.id"
                                    :value="k.id"
                                >
                                    {{ k.name }}
                                </SelectItem>
                            </template>
                        </SelectContent>
                    </Select>
                </div>
            </div>

            <DialogFooter>
                <Button variant="outline" @click="onCancel">Cancel</Button>
                <Button :disabled="!canSave" @click="onSave">Save</Button>
            </DialogFooter>
        </DialogContent>
    </Dialog>

    <Dialog v-model:open="saving">
        <DialogContent class="sm:max-w-xl">
            <DialogHeader>
                <DialogTitle>Testing SSH connection…</DialogTitle>
                <DialogDescription>
                    We’re accepting the fingerprint and probing the host.
                </DialogDescription>
            </DialogHeader>
            <div class="flex items-center gap-3">
                <svg
                    v-if="!logDone"
                    class="size-5 animate-spin text-muted-foreground"
                    viewBox="0 0 24 24"
                >
                    <circle
                        class="opacity-25"
                        cx="12"
                        cy="12"
                        r="10"
                        stroke="currentColor"
                        stroke-width="4"
                        fill="none"
                    />
                    <path
                        class="opacity-75"
                        fill="currentColor"
                        d="M4 12a8 8 0 018-8v4a4 4 0 00-4 4H4z"
                    />
                </svg>
                <span class="text-sm text-muted-foreground">
                    {{
                        logDone
                            ? "Completed. Review logs below."
                            : "Running ssh -o StrictHostKeyChecking=accept-new …"
                    }}
                </span>
            </div>
            <div
                class="mt-4 h-48 overflow-auto rounded-md border bg-muted p-2 text-xs font-mono whitespace-pre-wrap"
            >
                <template v-for="(l, idx) in logLines" :key="idx"
                    >{{ l }}\n</template
                >
            </div>
            <DialogFooter v-if="logDone">
                <Button variant="outline" @click="saving = false">Close</Button>
            </DialogFooter>
        </DialogContent>
    </Dialog>

    <Dialog v-model:open="askPasswordOpen">
        <DialogContent class="sm:max-w-md">
            <DialogHeader>
                <DialogTitle>Authentication required</DialogTitle>
                <DialogDescription>
                    Enter the SSH password to install the selected key on the
                    server.
                </DialogDescription>
            </DialogHeader>
            <div class="grid gap-2">
                <label class="text-sm font-medium" for="ssh-password"
                    >Password</label
                >
                <Input
                    id="ssh-password"
                    v-model="password"
                    type="password"
                    placeholder="••••••"
                />
            </div>
            <DialogFooter>
                <Button variant="outline" @click="askPasswordOpen = false"
                    >Cancel</Button
                >
                <Button
                    :disabled="installing || !password"
                    @click="onInstallKey(username, lastHostAlias || '')"
                >
                    {{ installing ? "Installing…" : "Install key" }}
                </Button>
            </DialogFooter>
        </DialogContent>
    </Dialog>
</template>
