<script setup lang="ts">
import { ref, computed, watch } from "vue";
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
import { useServerConfigStore } from "@/stores/servers";
import type { KeyPair } from "@/types/key";
import Server from "@/class/Server";

const open = defineModel<boolean>("open", { default: false });
const props = defineProps<{
    server: Server
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
const serversStore = useServerConfigStore();
const keys = ref<KeyPair[]>([]);
keys.value = keysStore.listKeys();
const selectedKeyId = ref<string | null>(null);
const saving = ref(false);

watch(open, async (isOpen) => {
    if (isOpen && props.server.config().getID()) {
        const server = serversStore.getServer(props.server.config().getID());
        if (server) {
            name.value = server.name || "";
            ip.value = server.ip || "";
            selectedKeyId.value = server.keyID?.toString() || "none";
            username.value = server.username || "";
        }
    }
});

async function saveServer() {
    if (!props.server.config().getID() || !canSave.value) return;

    saving.value = true;

    try {
        const updatedServer = {
            ...props.server.config().get(),
            name: name.value.trim(),
            username: username.value.trim(),
            ip: ip.value.trim(),
            keyID:
                selectedKeyId.value && selectedKeyId.value !== "none"
                    ? parseInt(selectedKeyId.value)
                    : 0,
        };

        await props.server.config().update(updatedServer);
        open.value = false;
    } catch (error) {
        console.error("Error saving server:", error);
    } finally {
        saving.value = false;
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
