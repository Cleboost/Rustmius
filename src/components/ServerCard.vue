<script lang="ts" setup>
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Icon } from "@iconify/vue";
import {
    ContextMenu,
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuTrigger,
} from "@/components/ui/context-menu";
import { Pencil, Trash2 } from "lucide-vue-next";
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
import { useServerInstanceStore } from "@/stores/serverInstance";
import { useRouter } from "vue-router";
import { ref } from "vue";
import Server from "@/class/Server";
import { useServerConfigStore } from "@/stores/servers";

const props = defineProps<{
    server: Server;
}>();

const confirmOpen = ref(false);
const router = useRouter();
const serverInstanceStore = useServerInstanceStore();
const serverStore = useServerConfigStore();

async function connect() {
    await serverInstanceStore.addServerInstance(props.server);
    return router.push(`/server/${props.server.config().getID()}`);
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
                    <div class="text-lg font-semibold">
                        {{ server.config().getName() || "N/A" }}
                    </div>
                    <div class="text-sm text-gray-500">
                        IP:
                        {{ server.config().getIP() || "N/A" }}
                    </div>
                </div>
                <Button class="self-center" variant="outline" @click="connect()"
                    >View</Button
                >
            </Card>
        </ContextMenuTrigger>
        <ContextMenuContent>
            <ContextMenuItem @select="">
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
                <AlertDialogAction @click="serverStore.removeServer(props.server.config().getID());">Supprimer</AlertDialogAction>
            </AlertDialogFooter>
        </AlertDialogContent>
    </AlertDialog>
</template>
