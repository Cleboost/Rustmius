<script setup lang="ts">
import { ref, computed } from "vue";
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
    CheckCircle,
    AlertCircle,
    Loader2,
} from "lucide-vue-next";
import { useServerConfigStore } from "@/stores/servers";
import { Command } from "@tauri-apps/plugin-shell";
import {
    readTextFile,
    writeTextFile,
    BaseDirectory,
} from "@tauri-apps/plugin-fs";
import type { KeyPair } from "@/types/key";
import { Server as ServerType } from "@/types/server";

const open = defineModel<boolean>("open", { default: false });

const name = ref("");
const username = ref("");
const ip = ref("");
const selectedKeyId = ref<string | null>(null);

const keysStore = useKeysStore();
const serverStore = useServerConfigStore();
const keys = ref<KeyPair[]>([]);
keys.value = keysStore.listKeys();

const testing = ref(false);
const connectionStatus = ref<
    "idle" | "testing" | "success" | "failed" | "needs-key"
>("idle");
const logLines = ref<string[]>([]);

const keyDeploymentConfirmOpen = ref(false);
const passwordModalOpen = ref(false);
const deploying = ref(false);
const password = ref("");

const successModalOpen = ref(false);
const errorModalOpen = ref(false);
const serverAdded = ref(false);
const keyDeployed = ref(false);
const keyAlreadyExists = ref(false);
const currentServerId = ref<string | null>(null);

function isValidIP(ip: string): boolean {
    const trimmed = ip.trim();
    if (!trimmed) return false;
    
    // Support IPv4 and hostname
    const ipv4Regex = /^(\d{1,3}\.){3}\d{1,3}$/;
    const hostnameRegex = /^[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(\.[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$/;
    
    if (ipv4Regex.test(trimmed)) {
        // Validate IPv4 octets
        const parts = trimmed.split('.');
        return parts.every(part => {
            const num = parseInt(part, 10);
            return num >= 0 && num <= 255;
        });
    }
    
    // Allow hostname if not IPv4
    return hostnameRegex.test(trimmed);
}

const canSave = computed(
    () =>
        name.value.trim().length > 0 &&
        username.value.trim().length > 0 &&
        isValidIP(ip.value) &&
        selectedKeyId.value !== null,
);

const canTestConnection = computed(
    () => username.value.trim().length > 0 && isValidIP(ip.value),
);

function resetForm() {
    name.value = "";
    username.value = "";
    ip.value = "";
    selectedKeyId.value = null;
    connectionStatus.value = "idle";
    logLines.value = [];
    password.value = "";
    serverAdded.value = false;
    keyDeployed.value = false;
    keyAlreadyExists.value = false;
    currentServerId.value = null;
    errorModalOpen.value = false;
}

function onCancel() {
    if (testing.value) return;
    open.value = false;
    resetForm();
}

function handleDialogOpenChange(newValue: boolean) {
    if (!newValue && testing.value) {
        return;
    }
    open.value = newValue;
    if (!newValue) {
        resetForm();
    }
}

async function testConnection() {
    if (!canTestConnection.value) return;

    testing.value = true;
    connectionStatus.value = "testing";
    logLines.value = [];

    try {
        const sshCmd = Command.create("ssh", [
            "-o",
            "StrictHostKeyChecking=accept-new",
            "-o",
            "ConnectTimeout=8",
            "-o",
            "BatchMode=yes",
            "-l",
            username.value.trim(),
            ip.value.trim(),
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
        });

        const status = await sshCmd.execute();

        if (status.stdout)
            logLines.value.push(`[out] ${status.stdout.trimEnd()}`);
        if (status.stderr)
            logLines.value.push(`[err] ${status.stderr.trimEnd()}`);

        if (status.code === 0) {
            connectionStatus.value = "success";
            if (
                !keyDeployed.value &&
                !logLines.value.some((log) => /password:/i.test(log))
            ) {
                keyAlreadyExists.value = true;
            }
        } else {
            const needsPassword = logLines.value.some(
                (log) =>
                    /password:/i.test(log) || /permission denied/i.test(log),
            );
            connectionStatus.value = needsPassword ? "needs-key" : "failed";
        }
    } catch (error) {
        logLines.value.push(
            `[exception] ${(error as any)?.toString?.() ?? "error"}`,
        );
        connectionStatus.value = "failed";
    } finally {
        testing.value = false;
    }
}

async function deployKey() {
    if (!selectedKeyId.value || !password.value) return;

    deploying.value = true;

    try {
        const keyIdNum = Number(selectedKeyId.value);
        const keyPath = await getKeyPath(keyIdNum);
        if (!keyPath) {
            logLines.value.push("[error] Key not found");
            return;
        }

        const proc = Command.create("sshpass", [
            "-p",
            password.value,
            "ssh-copy-id",
            "-o",
            "StrictHostKeyChecking=accept-new",
            "-i",
            keyPath,
            `${username.value.trim()}@${ip.value.trim()}`,
        ]);

        proc.stdout.on("data", (line) => {
            logLines.value.push(`[copy] ${String(line).trimEnd()}`);
        });

        proc.stderr.on("data", (line) => {
            logLines.value.push(`[copy-err] ${String(line).trimEnd()}`);
        });

        const result = await proc.execute();

        if (result.stdout)
            logLines.value.push(`[copy-out] ${result.stdout.trimEnd()}`);
        if (result.stderr)
            logLines.value.push(`[copy-err] ${result.stderr.trimEnd()}`);
        logLines.value.push(`[copy-exit] code=${result.code}`);

        if (result.code === 0) {
            passwordModalOpen.value = false;
            password.value = "";
            keyDeployed.value = true;
            
            await testConnectionAfterKeyDeployment();
        }
    } catch (error) {
        logLines.value.push(
            `[install-exception] ${(error as any)?.toString?.() ?? "error"}`,
        );
    } finally {
        deploying.value = false;
    }
}

async function testConnectionAfterKeyDeployment() {
    testing.value = true;
    connectionStatus.value = "testing";
    logLines.value = [];

    try {
        const sshCmd = Command.create("ssh", [
            "-o",
            "StrictHostKeyChecking=accept-new",
            "-o",
            "ConnectTimeout=8",
            "-o",
            "BatchMode=yes",
            "-l",
            username.value.trim(),
            ip.value.trim(),
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
        });

        const status = await sshCmd.execute();

        if (status.stdout)
            logLines.value.push(`[out] ${status.stdout.trimEnd()}`);
        if (status.stderr)
            logLines.value.push(`[err] ${status.stderr.trimEnd()}`);

        if (status.code === 0) {
            await createServer();
            connectionStatus.value = "success";
            successModalOpen.value = true;
        } else {
            connectionStatus.value = "failed";
            errorModalOpen.value = true;
        }
    } catch (error) {
        logLines.value.push(
            `[exception] ${(error as any)?.toString?.() ?? "error"}`,
        );
        connectionStatus.value = "failed";
        errorModalOpen.value = true;
    } finally {
        testing.value = false;
    }
}

async function getKeyPath(keyId: number): Promise<string | undefined> {
    await keysStore.load();
    const allKeys = await keysStore.getKeys();
    return allKeys.find((k) => k.id === keyId)?.private;
}

async function onSave() {
    if (!canSave.value) return;

    await testConnectionBeforeSave();
}

async function testConnectionBeforeSave() {
    if (!canTestConnection.value) return;

    testing.value = true;
    connectionStatus.value = "testing";
    logLines.value = [];

    try {
        const sshCmd = Command.create("ssh", [
            "-o",
            "StrictHostKeyChecking=accept-new",
            "-o",
            "ConnectTimeout=8",
            "-o",
            "BatchMode=yes",
            "-l",
            username.value.trim(),
            ip.value.trim(),
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
        });

        const status = await sshCmd.execute();

        if (status.stdout)
            logLines.value.push(`[out] ${status.stdout.trimEnd()}`);
        if (status.stderr)
            logLines.value.push(`[err] ${status.stderr.trimEnd()}`);

        if (status.code === 0) {
            await createServer();
            connectionStatus.value = "success";
            if (
                !keyDeployed.value &&
                !logLines.value.some((log) => /password:/i.test(log))
            ) {
                keyAlreadyExists.value = true;
            }
            successModalOpen.value = true;
        } else {
            const needsPassword = logLines.value.some(
                (log) =>
                    /password:/i.test(log) || /permission denied/i.test(log),
            );
            if (needsPassword && selectedKeyId.value) {
                connectionStatus.value = "needs-key";
                keyDeploymentConfirmOpen.value = true;
            } else {
                connectionStatus.value = "failed";
                errorModalOpen.value = true;
            }
        }
    } catch (error) {
        logLines.value.push(
            `[exception] ${(error as any)?.toString?.() ?? "error"}`,
        );
        connectionStatus.value = "failed";
        errorModalOpen.value = true;
    } finally {
        testing.value = false;
    }
}

async function createServer() {
    const serverId = crypto.randomUUID();
    const server: ServerType = {
        id: serverId,
        name: name.value.trim(),
        ip: ip.value.trim(),
        keyID: selectedKeyId.value ? Number(selectedKeyId.value) : 0,
        username: username.value.trim(),
    };

    try {
        await updateSSHConfig(server);
        serverStore.addServer(server);
        serverAdded.value = true;
        currentServerId.value = serverId;
        open.value = false;
    } catch (error) {
        console.error("Error saving server:", error);
        throw error;
    }
}

function agreeToDeployKey() {
    keyDeploymentConfirmOpen.value = false;
    passwordModalOpen.value = true;
}

async function declineKeyDeployment() {
    keyDeploymentConfirmOpen.value = false;
    resetForm();
}

async function cancelKeyDeployment() {
    passwordModalOpen.value = false;
    password.value = "";
    resetForm();
}

async function updateSSHConfig(server: ServerType) {
    const sshConfigPath = ".ssh/config";

    let content = "";
    try {
        content = await readTextFile(sshConfigPath, {
            baseDir: BaseDirectory.Home,
        });
    } catch {}

    const keyPath = server.keyID ? await getKeyPath(server.keyID) : null;

    const newBlock = [
        `Host ${server.id}`,
        server.ip ? `  Hostname ${server.ip}` : undefined,
        `  User ${server.username}`,
        keyPath ? `  IdentityFile ${keyPath}` : undefined,
    ]
        .filter(Boolean)
        .join("\n");

    const newContent = content
        ? `${content.trim()}\n\n${newBlock}\n`
        : `${newBlock}\n`;

    await writeTextFile(sshConfigPath, newContent, {
        baseDir: BaseDirectory.Home,
    });
}

async function removeFromSSHConfig(serverId: string) {
    const sshConfigPath = ".ssh/config";

    try {
        const content = await readTextFile(sshConfigPath, {
            baseDir: BaseDirectory.Home,
        });

        const lines = content.split("\n");
        const newLines: string[] = [];
        let skipBlock = false;

        for (let i = 0; i < lines.length; i++) {
            const line = lines[i];

            if (line.trim().startsWith(`Host ${serverId}`)) {
                skipBlock = true;
                continue;
            }

            if (
                skipBlock &&
                (line.trim() === "" || line.trim().startsWith("Host "))
            ) {
                skipBlock = false;
            }

            if (!skipBlock) {
                newLines.push(line);
            }
        }

        await writeTextFile(sshConfigPath, newLines.join("\n"), {
            baseDir: BaseDirectory.Home,
        });
    } catch (error) {
        console.error("Error removing server from SSH config:", error);
    }
}
</script>

<template>
    <Dialog :open="open" @update:open="handleDialogOpenChange">
        <DialogContent class="sm:max-w-md">
            <DialogHeader>
                <DialogTitle>New Server</DialogTitle>
                <DialogDescription>
                    Configure your new server connection
                </DialogDescription>
            </DialogHeader>

            <div class="grid gap-4 py-4">
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
                        <Network class="size-4 opacity-60" /> IP Address
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
                        <KeyIcon class="size-4 opacity-60" /> SSH Key
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

                <!-- Zone de statut de connexion -->
                <div v-if="testing || connectionStatus !== 'idle'" class="mt-4">
                    <div
                        class="p-4 bg-muted rounded-lg border"
                        :class="{
                            'border-blue-500/50': connectionStatus === 'testing',
                            'border-green-500/50': connectionStatus === 'success',
                            'border-red-500/50': connectionStatus === 'failed',
                            'border-yellow-500/50': connectionStatus === 'needs-key',
                        }"
                    >
                        <div class="flex items-center gap-3 mb-3">
                            <Loader2
                                v-if="connectionStatus === 'testing'"
                                class="size-5 animate-spin text-blue-400"
                            />
                            <CheckCircle
                                v-else-if="connectionStatus === 'success'"
                                class="size-5 text-green-400"
                            />
                            <AlertCircle
                                v-else-if="connectionStatus === 'failed'"
                                class="size-5 text-red-400"
                            />
                            <AlertCircle
                                v-else-if="connectionStatus === 'needs-key'"
                                class="size-5 text-yellow-400"
                            />
                            <span class="font-medium text-sm">
                                <template v-if="connectionStatus === 'testing'"
                                    >Testing SSH connection...</template
                                >
                                <template v-else-if="connectionStatus === 'success'"
                                    >Connection successful!</template
                                >
                                <template v-else-if="connectionStatus === 'failed'"
                                    >Connection failed</template
                                >
                                <template v-else-if="connectionStatus === 'needs-key'"
                                    >SSH key deployment required</template
                                >
                            </span>
                        </div>

                        <!-- Logs en temps réel -->
                        <div
                            v-if="logLines.length > 0"
                            class="max-h-32 overflow-y-auto bg-black/50 rounded p-2 text-xs font-mono text-muted-foreground space-y-1"
                        >
                            <div
                                v-for="(line, idx) in logLines"
                                :key="idx"
                                class="text-left"
                            >
                                {{ line }}
                            </div>
                        </div>
                        <div v-else-if="testing" class="text-sm text-muted-foreground animate-pulse">
                            <div class="flex items-center gap-2">
                                <Loader2 class="size-4 animate-spin" />
                                <span>Attempting to connect to {{ ip }}...</span>
                            </div>
                            <p class="text-xs mt-2 opacity-75">
                                This may take a few seconds. Please wait...
                            </p>
                        </div>
                    </div>
                </div>
            </div>

            <DialogFooter class="gap-2">
                <Button
                    variant="outline"
                    :disabled="testing"
                    @click="onCancel"
                    >Cancel</Button
                >
                <Button
                    :disabled="!canSave || testing"
                    @click="onSave"
                >
                    <template v-if="testing">
                        <Loader2 class="size-4 mr-2 animate-spin" />
                        Testing connection...
                    </template>
                    <template v-else>Save Server</template>
                </Button>
            </DialogFooter>
        </DialogContent>
    </Dialog>

    <Dialog v-model:open="keyDeploymentConfirmOpen">
        <DialogContent class="sm:max-w-md">
            <DialogHeader>
                <DialogTitle class="flex items-center gap-2">
                    <KeyIcon class="size-5 text-blue-400" />
                    SSH Key Deployment Required
                </DialogTitle>
                <DialogDescription>
                    The server requires password authentication to deploy your
                    SSH key.
                </DialogDescription>
            </DialogHeader>

            <div class="space-y-4">
                <div
                    class="p-3 bg-blue-950/50 rounded-lg border border-blue-800/50"
                >
                    <div class="flex items-start gap-3">
                        <AlertCircle class="size-5 text-blue-400 mt-0.5" />
                        <div class="space-y-2">
                            <p class="text-sm font-medium text-blue-100">
                                Password authentication detected
                            </p>
                            <p class="text-sm text-blue-200">
                                To complete the server setup, you'll need to
                                enter your SSH password to deploy the selected
                                key :
                                <strong class="text-blue-100">
                                    {{
                                        keys.find(
                                            (k) =>
                                                String(k.id) ===
                                                String(selectedKeyId),
                                        )?.name
                                    }}
                                </strong>
                            </p>
                        </div>
                    </div>
                </div>

                <div class="p-3 bg-muted rounded-lg">
                    <h4 class="font-medium mb-2">Server Details</h4>
                    <div class="space-y-1 text-sm text-muted-foreground">
                        <div><strong>Name:</strong> {{ name }}</div>
                        <div><strong>IP:</strong> {{ ip }}</div>
                        <div><strong>Username:</strong> {{ username }}</div>
                    </div>
                </div>
            </div>

            <DialogFooter class="gap-2">
                <Button variant="outline" @click="declineKeyDeployment">
                    Decline
                </Button>
                <Button @click="agreeToDeployKey"> Agree & Deploy Key </Button>
            </DialogFooter>
        </DialogContent>
    </Dialog>

    <Dialog v-model:open="passwordModalOpen">
        <DialogContent class="sm:max-w-md">
            <DialogHeader>
                <DialogTitle class="flex items-center gap-2">
                    <KeyIcon class="size-5 text-green-400" />
                    Deploy SSH Key
                </DialogTitle>
                <DialogDescription>
                    Enter your SSH password to deploy the key to {{ ip }}
                </DialogDescription>
            </DialogHeader>

            <div class="space-y-4">
                <div class="grid gap-2">
                    <label class="text-sm font-medium" for="ssh-password"
                        >SSH Password</label
                    >
                    <Input
                        id="ssh-password"
                        v-model="password"
                        type="password"
                        placeholder="Enter your SSH password"
                        @keydown.enter="deployKey"
                    />
                </div>

                <div class="p-3 bg-muted rounded-lg">
                    <div class="flex items-center gap-2 mb-2">
                        <KeyIcon class="size-4 text-muted-foreground" />
                        <span class="text-sm font-medium">Deploying Key</span>
                    </div>
                    <div class="text-sm text-muted-foreground">
                        <strong>{{
                            keys.find(
                                (k) => String(k.id) === String(selectedKeyId),
                            )?.name
                        }}</strong>
                        to {{ username }}@{{ ip }}
                    </div>
                </div>
            </div>
            <DialogFooter class="gap-2">
                <Button variant="outline" @click="cancelKeyDeployment"
                    >Cancel</Button
                >
                <Button :disabled="deploying || !password" @click="deployKey">
                    <template v-if="deploying">
                        <Loader2 class="size-3 mr-1 animate-spin" />
                    </template>
                    {{ deploying ? "Deploying..." : "Deploy Key" }}
                </Button>
            </DialogFooter>
        </DialogContent>
    </Dialog>

    <Dialog v-model:open="successModalOpen">
        <DialogContent class="sm:max-w-md">
            <DialogHeader>
                <DialogTitle class="flex items-center gap-2">
                    <CheckCircle class="size-5 text-green-400" />
                    <template v-if="keyDeployed"
                        >Key Deployed Successfully!</template
                    >
                    <template v-else-if="keyAlreadyExists"
                        >Server Added Successfully!</template
                    >
                    <template v-else>Server Added Successfully!</template>
                </DialogTitle>
                <DialogDescription>
                    <template v-if="keyDeployed">
                        Your SSH key has been deployed and the server is ready
                        to use.
                    </template>
                    <template v-else-if="keyAlreadyExists">
                        The server is ready to use. No key deployment needed -
                        it was already configured.
                    </template>
                    <template v-else>
                        Your server has been configured and added to your SSH
                        config.
                    </template>
                </DialogDescription>
            </DialogHeader>

            <div class="space-y-4">
                <div
                    class="p-3 bg-green-950/50 rounded-lg border border-green-800/50"
                >
                    <h4 class="font-medium mb-2 text-green-100">
                        Server Details
                    </h4>
                    <div class="space-y-1 text-sm text-green-200">
                        <div>
                            <strong class="text-green-100">Name:</strong>
                            {{ name }}
                        </div>
                        <div>
                            <strong class="text-green-100">IP:</strong> {{ ip }}
                        </div>
                        <div>
                            <strong class="text-green-100">Username:</strong>
                            {{ username }}
                        </div>
                        <div v-if="selectedKeyId">
                            <strong class="text-green-100">SSH Key:</strong>
                            {{
                                keys.find(
                                    (k) =>
                                        String(k.id) === String(selectedKeyId),
                                )?.name || "Selected key"
                            }}
                        </div>
                    </div>
                </div>

                <div class="p-3 bg-muted rounded-lg">
                    <div class="flex items-center gap-2">
                        <template v-if="keyDeployed">
                            <CheckCircle class="size-4 text-green-400" />
                            <span class="text-sm font-medium text-green-300"
                                >✅ Key deployed successfully</span
                            >
                        </template>
                        <template v-else-if="keyAlreadyExists">
                            <CheckCircle class="size-4 text-blue-400" />
                            <span class="text-sm font-medium text-blue-300"
                                >✅ Key was already present</span
                            >
                        </template>
                        <template v-else-if="selectedKeyId">
                            <AlertCircle class="size-4 text-yellow-400" />
                            <span class="text-sm font-medium text-yellow-300"
                                >⚠️ Key deployment not attempted</span
                            >
                        </template>
                        <template v-else>
                            <AlertCircle class="size-4 text-gray-400" />
                            <span class="text-sm font-medium text-gray-300"
                                >ℹ️ No SSH key selected</span
                            >
                        </template>
                    </div>
                </div>
            </div>

            <DialogFooter>
                <Button
                    @click="
                        successModalOpen = false;
                        resetForm();
                    "
                    class="w-full"
                >
                    Close
                </Button>
            </DialogFooter>
        </DialogContent>
    </Dialog>

    <Dialog v-model:open="errorModalOpen">
        <DialogContent class="sm:max-w-md">
            <DialogHeader>
                <DialogTitle class="flex items-center gap-2">
                    <AlertCircle class="size-5 text-red-400" />
                    Connection Failed
                </DialogTitle>
                <DialogDescription>
                    Unable to establish SSH connection to {{ ip }}
                </DialogDescription>
            </DialogHeader>

            <div class="space-y-4">
                <div
                    class="p-3 bg-red-950/50 rounded-lg border border-red-800/50"
                >
                    <div class="flex items-start gap-3">
                        <AlertCircle class="size-5 text-red-400 mt-0.5" />
                        <div class="space-y-2">
                            <p class="text-sm font-medium text-red-100">
                                SSH connection failed
                            </p>
                            <p class="text-sm text-red-200">
                                The server could not be added because the SSH
                                connection failed. Please verify:
                            </p>
                            <ul class="list-disc list-inside text-sm text-red-200 space-y-1">
                                <li>The IP address is correct</li>
                                <li>The server is reachable</li>
                                <li>SSH is enabled on the server</li>
                                <li>
                                    {{
                                        selectedKeyId
                                            ? "The SSH key is correctly configured"
                                            : "A valid SSH key is selected"
                                    }}
                                </li>
                            </ul>
                        </div>
                    </div>
                </div>

                <div class="p-3 bg-muted rounded-lg">
                    <h4 class="font-medium mb-2">Server Details</h4>
                    <div class="space-y-1 text-sm text-muted-foreground">
                        <div><strong>Name:</strong> {{ name }}</div>
                        <div><strong>IP:</strong> {{ ip }}</div>
                        <div><strong>Username:</strong> {{ username }}</div>
                    </div>
                </div>

                <div
                    v-if="logLines.length > 0"
                    class="p-3 bg-muted rounded-lg max-h-40 overflow-y-auto"
                >
                    <h4 class="font-medium mb-2 text-sm">Connection Logs</h4>
                    <div class="text-xs font-mono text-muted-foreground space-y-1">
                        <div v-for="(line, idx) in logLines" :key="idx">
                            {{ line }}
                        </div>
                    </div>
                </div>
            </div>

            <DialogFooter>
                <Button
                    @click="
                        errorModalOpen = false;
                        resetForm();
                    "
                    class="w-full"
                >
                    Close
                </Button>
            </DialogFooter>
        </DialogContent>
    </Dialog>
</template>
