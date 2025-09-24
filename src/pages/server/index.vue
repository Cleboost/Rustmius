<script setup lang="ts">
import { computed, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import ServerHeader from "@/components/ServerHeader.vue";
import ServerToolCard from "@/components/ServerToolCard.vue";
import { useServerInstanceStore } from "@/stores/serverInstance";

const router = useRouter();
const serverInstancesStore = useServerInstanceStore();

const server = computed(() =>
    serverInstancesStore.getServerInstance(useRoute().params.id as string),
);

const editModalOpen = ref(false);

const tools = [
    {
        name: "Terminal",
        desc: "Open SSH terminal",
        icon: "mdi:terminal",
        click: () => server.value.console.create(),
    },
    {
        name: "Docker",
        desc: "Docker management UI",
        icon: "logos:docker",
        disabled: true,
        click: () => {},
    },
    {
        name: "File Manager",
        desc: "Browse server files",
        icon: "mdi:folder-open",
        disabled: true,
        click: () => {},
    },
    {
        name: "System Monitor",
        desc: "Monitor system resources",
        icon: "mdi:chart-line",
        disabled: true,
        click: () => {},
    },
    {
        name: "Logs Viewer",
        desc: "View system logs",
        icon: "mdi:file-document-outline",
        disabled: true,
        click: () => {},
    },
    {
        name: "Services",
        desc: "Manage system services",
        icon: "mdi:cog",
        disabled: true,
        click: () => {},
    },
];

function closeServerTab() {
    useServerInstanceStore().removeServerInstance(server.value?.id);
    router.push("/home");
}

function editServer() {
    editModalOpen.value = true;
}
</script>

<template>
    <div class="flex flex-col gap-4 max-w-full h-full min-h-0 p-4">
        <ServerHeader
            :server="server"
            @close="closeServerTab"
            @edit="editServer"
        />

        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            <ServerToolCard
                v-for="tool in tools"
                :key="tool.name"
                :name="tool.name"
                :desc="tool.desc"
                :icon="tool.icon"
                :disabled="tool.disabled || false"
                @click="tool.click()"
            />
        </div>
    </div>

    <!-- <EditServerModal
        v-model:open="editModalOpen"
        :server-id="serverId"
        @server-updated="handleServerUpdated"
    /> -->
</template>
