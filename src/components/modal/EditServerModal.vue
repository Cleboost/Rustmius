<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
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
import { useServersStore } from "@/stores/servers";
import { useServerInstancesStore } from "@/stores/serverInstances";
import {
    writeTextFile,
    readTextFile,
    readDir,
    exists,
} from "@tauri-apps/plugin-fs";
import { BaseDirectory } from "@tauri-apps/api/path";
import { homeDir, join } from "@tauri-apps/api/path";
import type { KeyPair } from "@/types/key";

const open = defineModel<boolean>("open", { default: false });
const serverId = defineProps<{
    serverId: string | null;
}>();

const emit = defineEmits<{
    (e: "server-updated"): void;
}>();

const name = ref("");
const username = ref("");
const ip = ref("");

const canSave = computed(
    () =>
        name.value.trim().length > 0 &&
        username.value.trim().length > 0 &&
        ip.value.trim().length > 0,
);

const keysStore = useKeysStore();
const serversStore = useServersStore();
const serverInstancesStore = useServerInstancesStore();
const keys = ref<KeyPair[]>([]);
const selectedKeyId = ref<string | null>(null);
const saving = ref(false);

watch(open, async (isOpen) => {
    if (isOpen && serverId.serverId) {
        await loadServerData();
    }
});

async function loadServerData() {
    if (!serverId.serverId) return;

    try {
        await keysStore.load();
        keys.value = await keysStore.getKeys();
        console.log("Loaded SSH keys:", keys.value.length, keys.value);

        if (keys.value.length === 0) {
            console.log(
                "No keys found, attempting to sync from SSH directory...",
            );
            await syncKeysFromSSHDirectory();
            keys.value = await keysStore.getKeys();
            console.log(
                "After sync - SSH keys:",
                keys.value.length,
                keys.value,
            );
        }

        await serversStore.load();
        const server = serversStore.findServerById(serverId.serverId);

        if (server) {
            name.value = server.name || "";
            ip.value = server.ip || "";
            selectedKeyId.value = server.keyID?.toString() || "none";

            const sshUsername = await getUsernameFromSSHConfig(
                serverId.serverId,
            );
            username.value = sshUsername || server.username || "";
        }
    } catch (error) {
        console.error("Error loading server data:", error);
    }
}

async function getUsernameFromSSHConfig(
    serverId: string,
): Promise<string | null> {
    try {
        const sshConfigRel = ".ssh/config";
        const content = await readTextFile(sshConfigRel, {
            baseDir: BaseDirectory.Home,
        });

        const lines = content.split("\n");
        let i = 0;

        while (i < lines.length) {
            const line = lines[i];
            const trimmed = line.trim();

            if (/^Host\s+/i.test(trimmed)) {
                const hostAlias = trimmed.replace(/^Host\s+/i, "").trim();

                if (hostAlias === serverId) {
                    i++;
                    while (
                        i < lines.length &&
                        !/^Host\s+/i.test(lines[i].trim())
                    ) {
                        const currentLine = lines[i].trim();
                        if (/^User\s+/i.test(currentLine)) {
                            return currentLine.replace(/^User\s+/i, "").trim();
                        }
                        i++;
                    }
                    break;
                }
            }
            i++;
        }

        return null;
    } catch (error) {
        console.error("Error reading SSH config:", error);
        return null;
    }
}

async function saveServer() {
    if (!serverId.serverId || !canSave.value) return;

    saving.value = true;

    try {
        const existingServer = serversStore.findServerById(serverId.serverId);
        if (!existingServer) {
            throw new Error("Server not found");
        }

        const updatedServer = {
            ...existingServer,
            name: name.value.trim(),
            username: username.value.trim(),
            ip: ip.value.trim(),
            keyID:
                selectedKeyId.value && selectedKeyId.value !== "none"
                    ? parseInt(selectedKeyId.value)
                    : undefined,
        };

        await updateSSHConfig(serverId.serverId, {
            name: updatedServer.name,
            username: updatedServer.username,
            ip: updatedServer.ip,
            keyID: updatedServer.keyID,
        });

        await serversStore.updateServer(serverId.serverId, updatedServer);

        const instance = serverInstancesStore.instancesArray.find(
            (s) => s.id === serverId.serverId,
        );
        if (instance) {
            await instance.ensureLoaded();
            console.log("Instance updated, new name:", instance.getName());
        }

        emit("server-updated");
        open.value = false;
    } catch (error) {
        console.error("Error saving server:", error);
    } finally {
        saving.value = false;
    }
}

async function syncKeysFromSSHDirectory() {
    try {
        const home = await homeDir();
        const sshRel = ".ssh";
        const sshAbs = await join(home, ".ssh");
        const entries = await readDir(sshRel, { baseDir: BaseDirectory.Home });
        const scanned: Array<{
            name: string;
            private: string;
            public?: string;
        }> = [];

        for (const e of entries) {
            if (e.isDirectory) continue;
            const name = e.name ?? "";
            if (!name.endsWith(".pub")) {
                continue;
            }
            const stem = name.slice(0, -4);
            if (!stem || stem === "config" || stem.startsWith("known_hosts")) {
                continue;
            }
            const privateRel = `${sshRel}/${stem}`;
            const hasPrivate = await exists(privateRel, {
                baseDir: BaseDirectory.Home,
            });
            if (!hasPrivate) {
                continue;
            }
            const publicPath = await join(sshAbs, name);
            const privatePath = await join(sshAbs, stem);
            scanned.push({
                name: stem,
                private: privatePath,
                public: publicPath,
            });
        }

        const existing = await keysStore.getKeys();
        const byPrivate = new Map(existing.map((k) => [k.private, k]));

        for (const s of scanned) {
            const found = byPrivate.get(s.private);
            if (found) {
                if (found.name !== s.name || found.public !== s.public) {
                    await keysStore.addOrUpdateKey({
                        id: found.id,
                        name: s.name,
                        private: s.private,
                        public: s.public,
                    });
                }
            } else {
                await keysStore.addOrUpdateKey({
                    name: s.name,
                    private: s.private,
                    public: s.public,
                });
            }
        }

        console.log("Synced keys from SSH directory:", scanned.length);
    } catch (error) {
        console.error("Error syncing keys from SSH directory:", error);
    }
}

async function keyPathFromId(id: number): Promise<string | undefined> {
    const ks = useKeysStore();
    await ks.load();
    const all = await ks.getKeys();
    return all.find((k) => k.id === id)?.private;
}

async function updateSSHConfig(
    serverId: string,
    config: {
        name: string;
        username: string;
        ip: string;
        keyID?: number;
    },
) {
    const sshConfigRel = ".ssh/config";

    try {
        const content = await readTextFile(sshConfigRel, {
            baseDir: BaseDirectory.Home,
        });

        const lines = content.split("\n");
        const newLines: string[] = [];
        let i = 0;
        let modified = false;

        while (i < lines.length) {
            const line = lines[i];
            const trimmed = line.trim();

            if (/^Host\s+/i.test(trimmed)) {
                const hostAlias = trimmed.replace(/^Host\s+/i, "").trim();

                if (hostAlias === serverId) {
                    const newBlock = [
                        `Host ${serverId}`,
                        config.ip ? `  Hostname ${config.ip}` : undefined,
                        config.username
                            ? `  User ${config.username}`
                            : undefined,
                        config.keyID
                            ? `  IdentityFile ${await keyPathFromId(config.keyID)}`
                            : undefined,
                    ]
                        .filter(Boolean)
                        .join("\n");

                    newLines.push(newBlock);
                    modified = true;

                    i++;
                    while (
                        i < lines.length &&
                        !/^Host\s+/i.test(lines[i].trim())
                    ) {
                        i++;
                    }
                } else {
                    newLines.push(line);
                    i++;
                    while (
                        i < lines.length &&
                        !/^Host\s+/i.test(lines[i].trim())
                    ) {
                        newLines.push(lines[i]);
                        i++;
                    }
                }
            } else {
                newLines.push(line);
                i++;
            }
        }

        if (modified) {
            await writeTextFile(sshConfigRel, newLines.join("\n"), {
                baseDir: BaseDirectory.Home,
            });
            console.log("SSH config updated successfully");
        } else {
            console.warn("Server block not found in SSH config");
        }
    } catch (error) {
        console.error("Error updating SSH config:", error);
        throw error;
    }
}

function resetForm() {
    name.value = "";
    username.value = "";
    ip.value = "";
    selectedKeyId.value = "none";
}
</script>

<template>
    <Dialog v-model:open="open" @update:open="resetForm">
        <DialogContent class="sm:max-w-[425px]">
            <DialogHeader>
                <DialogTitle class="flex items-center gap-2">
                    <ServerIcon class="size-5" />
                    Edit Server
                </DialogTitle>
                <DialogDescription>
                    Modify the server configuration. Changes will be saved to
                    your SSH config.
                </DialogDescription>
            </DialogHeader>

            <div class="grid gap-4 py-4">
                <!-- Server Name -->
                <div class="grid grid-cols-4 items-center gap-4">
                    <label for="name" class="text-right text-sm font-medium">
                        Name
                    </label>
                    <div class="col-span-3 flex items-center gap-2">
                        <ServerIcon class="size-4 text-muted-foreground" />
                        <Input
                            id="name"
                            v-model="name"
                            placeholder="Server name"
                            class="flex-1"
                        />
                    </div>
                </div>

                <!-- IP Address -->
                <div class="grid grid-cols-4 items-center gap-4">
                    <label for="ip" class="text-right text-sm font-medium">
                        IP
                    </label>
                    <div class="col-span-3 flex items-center gap-2">
                        <Network class="size-4 text-muted-foreground" />
                        <Input
                            id="ip"
                            v-model="ip"
                            placeholder="192.168.1.100"
                            class="flex-1"
                        />
                    </div>
                </div>

                <!-- Username -->
                <div class="grid grid-cols-4 items-center gap-4">
                    <label
                        for="username"
                        class="text-right text-sm font-medium"
                    >
                        User
                    </label>
                    <div class="col-span-3 flex items-center gap-2">
                        <UserIcon class="size-4 text-muted-foreground" />
                        <Input
                            id="username"
                            v-model="username"
                            placeholder="root"
                            class="flex-1"
                        />
                    </div>
                </div>

                <!-- SSH Key -->
                <div class="grid grid-cols-4 items-center gap-4">
                    <label for="key" class="text-right text-sm font-medium">
                        Key
                    </label>
                    <div class="col-span-3 flex items-center gap-2">
                        <KeyIcon class="size-4 text-muted-foreground" />
                        <Select v-model="selectedKeyId">
                            <SelectTrigger class="flex-1">
                                <SelectValue placeholder="Select SSH key" />
                            </SelectTrigger>
                            <SelectContent>
                                <SelectItem value="none">No key</SelectItem>
                                <SelectItem
                                    v-for="key in keys"
                                    :key="key.id"
                                    :value="key.id.toString()"
                                >
                                    {{ key.name }}
                                </SelectItem>
                            </SelectContent>
                        </Select>
                    </div>
                </div>
            </div>

            <DialogFooter>
                <Button variant="outline" @click="open = false">
                    Cancel
                </Button>
                <Button
                    @click="saveServer"
                    :disabled="!canSave || saving"
                    :class="{
                        'opacity-50 cursor-not-allowed': !canSave || saving,
                    }"
                >
                    <span v-if="saving">Saving...</span>
                    <span v-else>Save Changes</span>
                </Button>
            </DialogFooter>
        </DialogContent>
    </Dialog>
</template>
