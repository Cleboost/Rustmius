<script lang="ts" setup>
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Icon } from "@iconify/vue";
import { Server } from "@/types/server";
import {
    ContextMenu,
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuTrigger,
} from "@/components/ui/context-menu";
import { Pencil, Trash2 } from "lucide-vue-next";
import { useServersStore } from "@/stores/servers";
import {
    AlertDialog,
    AlertDialogAction,
    AlertDialogCancel,
    AlertDialogContent,
    AlertDialogDescription,
    AlertDialogFooter,
    AlertDialogHeader,
    AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { ref } from "vue";
import { useServerInstancesStore } from "@/stores/serverInstances";
import { useRouter } from "vue-router";

defineProps<{
    server: Server;
}>();

const emit = defineEmits<{
    (e: "connect", server: Server["id"]): void;
    (e: "edit", server: Server["id"]): void;
    (e: "delete", server: Server["id"]): void;
}>();

const confirmOpen = ref(false);
const serversStore = useServersStore();
const router = useRouter();
const serverInstancesStore = useServerInstancesStore();
async function onConfirmDelete(id: Server["id"]) {
    await serversStore.removeServer(id);
    confirmOpen.value = false;
}

async function onConnect(id: Server["id"]) {
    console.log("ServerCard: Connecting to server:", id);
    try {
        await serversStore.connectToServer(id);
        console.log("ServerCard: Successfully initiated SSH connection");
    } catch (error) {
        console.error("ServerCard: Failed to connect to server:", error);
        // TODO: Show error message to user
    }
}
</script>

<template>
    <ContextMenu>
        <ContextMenuTrigger as-child>
            <Card class="p-2 flex flex-row">
                <Icon
                    class="self-center"
                    icon="logos:debian"
                    width="32"
                    height="32"
                />
                <div class="flex flex-col">
                    <div class="text-lg font-semibold">{{ server.name }}</div>
                    <div class="text-sm text-gray-500">
                        IP: {{ server.ip || "N/A" }}
                    </div>
                </div>
                <Button
                    class="self-center"
                    variant="outline"
                    @click="onConnect(server.id)"
                    >Connect</Button
                >
            </Card>
        </ContextMenuTrigger>
        <ContextMenuContent>
            <ContextMenuItem @select="emit('edit', server.id)">
                <Pencil class="size-4 opacity-60 mr-2" /> Éditer
            </ContextMenuItem>
            <ContextMenuItem @select="confirmOpen = true">
                <Trash2 class="size-4 opacity-60 mr-2" /> Supprimer
            </ContextMenuItem>
        </ContextMenuContent>
    </ContextMenu>

    <AlertDialog v-model:open="confirmOpen">
        <AlertDialogContent>
            <AlertDialogHeader>
                <AlertDialogTitle>Supprimer ce serveur ?</AlertDialogTitle>
                <AlertDialogDescription>
                    Cette action est irréversible et supprimera l'entrée de la
                    liste locale.
                </AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
                <AlertDialogCancel>Annuler</AlertDialogCancel>
                <AlertDialogAction @click="onConfirmDelete(server.id)"
                    >Supprimer</AlertDialogAction
                >
            </AlertDialogFooter>
        </AlertDialogContent>
    </AlertDialog>
</template>
