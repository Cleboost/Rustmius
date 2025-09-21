<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useServerInstancesStore } from "@/stores/serverInstances";
import ServerHeader from "@/components/ServerHeader.vue";
import ServerToolCard from "@/components/ServerToolCard.vue";
import EditServerModal from "@/components/modal/EditServerModal.vue";

const route = useRoute();
const router = useRouter();
const serverInstancesStore = useServerInstancesStore();

const serverId = computed(() => route.params.id as string);
const serverInstance = computed(() => {
    const existing = serverInstancesStore.instancesArray.find(server => server.id === serverId.value);
    if (existing) {
        return existing;
    }
    console.warn('Server instance not found for ID:', serverId.value);
    return null;
});

const editModalOpen = ref(false);

const tools = [
    {
        name: "Terminal",
        desc: "Open SSH terminal",
        icon: "mdi:terminal",
        click: () => serverInstance.value?.launchTerminal(),
    },
    {
        name: "Docker",
        desc: "Docker management UI",
        icon: "logos:docker",
        disabled: true,
        click: () => {
            window.open("http://localhost:9000", "_blank");
        },
    },
    {
        name: "File Manager",
        desc: "Browse server files",
        icon: "mdi:folder-open",
        disabled: true,
        click: () => {
            console.log(
                "File manager not implemented yet for server:",
                serverInstance.value?.getName(),
            );
        },
    },
    {
        name: "System Monitor",
        desc: "Monitor system resources",
        icon: "mdi:chart-line",
        disabled: true,
        click: () => {
            console.log(
                "System monitor not implemented yet for server:",
                serverInstance.value?.getName(),
            );
        },
    },
    {
        name: "Logs Viewer",
        desc: "View system logs",
        icon: "mdi:file-document-outline",
        disabled: true,
        click: () => {
            console.log(
                "Logs viewer not implemented yet for server:",
                serverInstance.value?.getName(),
            );
        },
    },
    {
        name: "Services",
        desc: "Manage system services",
        icon: "mdi:cog",
        disabled: true,
        click: () => {
            console.log(
                "Services manager not implemented yet for server:",
                serverInstance.value?.getName(),
            );
        },
    },
];

onMounted(async () => {
    if (serverInstance.value) {
        await serverInstance.value.ensureLoaded();
    }
});

function closeServerTab() {
    console.log('Server page: Closing server tab for:', serverId.value);
    serverInstancesStore.removeServerInstance(serverId.value);
    router.push("/home");
}

function editServer() {
    console.log('Server page: Editing server:', serverId.value);
    editModalOpen.value = true;
}

async function handleServerUpdated() {
    console.log('Server page: Server updated, refreshing instance');
    editModalOpen.value = false;
    
    const instance = serverInstancesStore.instancesArray.find(s => s.id === serverId.value);
    if (instance) {
        await instance.ensureLoaded();
        console.log('Instance reloaded, new name:', instance.getName());
    }
}
</script>

<template>
    <div class="flex flex-col gap-4 max-w-full h-full min-h-0 p-4">
        <!-- Server Header -->
        <ServerHeader :server-id="serverId" @close="closeServerTab" @edit="editServer" />

        <!-- Tools Grid -->
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

    <!-- Edit Server Modal -->
    <EditServerModal 
        v-model:open="editModalOpen" 
        :server-id="serverId"
        @server-updated="handleServerUpdated"
    />
</template>
