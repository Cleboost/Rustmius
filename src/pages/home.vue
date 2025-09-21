<script setup lang="ts">
import { ref, onMounted } from "vue";
import ServerCard from "@/components/ServerCard.vue";
import { Button } from "@/components/ui/button";
import { useServersStore } from "@/stores/servers";
import { Icon } from "@iconify/vue";
import NewServerModal from "@/components/modal/NewServerModal.vue";

const serversStore = useServersStore();
const newServerOpen = ref(false);

onMounted(async () => {
    await serversStore.load();
});
</script>

<template>
    <div>
        <h1 class="text-4xl font-bold">Servers</h1>
        <div class="flex flex-row space-x-2">
            <Button class="mt-4" size="sm" @click="newServerOpen = true">
                <Icon icon="ph:plus" class="size-3" />
                New Host
            </Button>
            <Button class="mt-4" size="sm" variant="secondary" disabled>
                <Icon icon="mdi:console" class="size-3" />
                Terminal
            </Button>
            <Button class="mt-4" size="sm" variant="secondary" disabled>
                <Icon icon="mdi:import" class="size-3" />
                Serial
            </Button>
        </div>
        <NewServerModal v-model:open="newServerOpen" />
        <h2 class="mt-8 text-2xl font-semibold">Your hosts</h2>
        <div class="mt-4 flex flex-wrap gap-2">
            <ServerCard
                v-for="server in serversStore.getServers"
                :key="server.id"
                :server="server"
                @connect="(id) => console.log('Connect to server with id:', id)"
            />
        </div>
    </div>
</template>
