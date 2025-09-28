<script setup lang="ts">
import { computed, ref, onMounted } from "vue";
import { useRoute, useRouter } from "vue-router";
import ServerHeader from "@/components/ServerHeader.vue";
import { Icon } from "@iconify/vue";
import { useServerInstanceStore } from "@/stores/serverInstance";
import EditServerModal from "@/components/modal/EditServerModal.vue";
import DockerCard from "@/components/DockerCard.vue";
import { Button } from "@/components/ui/button";

const router = useRouter();
const serverInstancesStore = useServerInstanceStore();

const server = computed(() =>
    serverInstancesStore.getServerInstance(useRoute().params.id as string),
);

const editModalOpen = ref(false);

const containersData = ref({
    running: 0,
    stopped: 0,
    total: 0
});

const imagesData = ref({
    local: 0,
    size: "0B",
    dangling: 0
});

const loading = ref(false);

async function loadDockerData() {
    if (!server.value) return;
    
    loading.value = true;
    try {
        const data = await server.value.docker.getAllDockerData();
        
        containersData.value = data.containers;
        imagesData.value = data.images;
    } catch (error) {
        console.error("Error loading Docker data:", error);
    } finally {
        loading.value = false;
    }
}

onMounted(() => {
    loadDockerData();
});

function closeServerTab() {
    useServerInstanceStore().removeServerInstance(server.value?.id);
    router.push("/home");
}

function editServer() {
    editModalOpen.value = true;
}

function viewContainers() {
    router.push(`/server/${server.value?.id}/docker/containers`);
}

function viewImages() {
    router.push(`/server/${server.value?.id}/docker/images`);
}
</script>

<template>
    <div class="flex flex-col gap-4 max-w-full h-full min-h-0 p-4">
        <div class="flex items-center gap-4">
            <button 
                @click="router.back()"
                class="flex items-center justify-center w-10 h-10 rounded-lg bg-gray-100 hover:bg-gray-200 dark:bg-gray-800 dark:hover:bg-gray-700 transition-colors"
            >
                <Icon icon="mdi:arrow-left" class="w-5 h-5" />
            </button>
            <h1 class="text-2xl font-semibold">Docker</h1>
        </div>     
         <div class="flex w-full gap-4">
             <div class="flex-1">
                 <ServerHeader
                     :server="server"
                     :buttons="false"/> 
             </div>
             <div class="flex-1">
                 <DockerCard
                     :server="server"/>
             </div>
         </div>
         
         <div class="flex items-center justify-between">
             <h2 class="text-lg font-semibold">Docker Management</h2>
             <Button 
                 @click="loadDockerData" 
                 :disabled="loading"
                 variant="outline"
                 size="sm"
             >
                 <Icon icon="mdi:refresh" class="w-4 h-4 mr-2" />
                 {{ loading ? 'Loading...' : 'Refresh' }}
             </Button>
         </div>
         
         <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
             <div class="border rounded-lg p-6 bg-card hover:shadow-md transition-shadow">
                 <div class="flex items-center gap-3 mb-4">
                     <Icon icon="mdi:docker" class="w-8 h-8 text-blue-500" />
                     <div>
                         <h2 class="text-xl font-semibold">Containers</h2>
                         <p class="text-sm text-muted-foreground">Manage Docker containers</p>
                     </div>
                 </div>
                 <div class="space-y-2">
                     <div class="flex items-center justify-between">
                         <span class="text-sm">Running:</span>
                         <span class="text-sm font-medium">
                             <Icon v-if="loading" icon="mdi:loading" class="w-4 h-4 animate-spin" />
                             <span v-else>{{ containersData.running }}</span>
                         </span>
                     </div>
                     <div class="flex items-center justify-between">
                         <span class="text-sm">Stopped:</span>
                         <span class="text-sm font-medium">
                             <Icon v-if="loading" icon="mdi:loading" class="w-4 h-4 animate-spin" />
                             <span v-else>{{ containersData.stopped }}</span>
                         </span>
                     </div>
                     <div class="flex items-center justify-between">
                         <span class="text-sm">Total:</span>
                         <span class="text-sm font-medium">
                             <Icon v-if="loading" icon="mdi:loading" class="w-4 h-4 animate-spin" />
                             <span v-else>{{ containersData.total }}</span>
                         </span>
                     </div>
                 </div>
                 <Button 
                     @click="viewContainers"
                     class="w-full mt-4"
                 >
                     View Containers
                 </Button>
             </div>

             <div class="border rounded-lg p-6 bg-card hover:shadow-md transition-shadow">
                 <div class="flex items-center gap-3 mb-4">
                     <Icon icon="mdi:image-multiple" class="w-8 h-8 text-green-500" />
                     <div>
                         <h2 class="text-xl font-semibold">Images</h2>
                         <p class="text-sm text-muted-foreground">Manage Docker images</p>
                     </div>
                 </div>
                 <div class="space-y-2">
                     <div class="flex items-center justify-between">
                         <span class="text-sm">Local:</span>
                         <span class="text-sm font-medium">
                             <Icon v-if="loading" icon="mdi:loading" class="w-4 h-4 animate-spin" />
                             <span v-else>{{ imagesData.local }}</span>
                         </span>
                     </div>
                     <div class="flex items-center justify-between">
                         <span class="text-sm">Size:</span>
                         <span class="text-sm font-medium">
                             <Icon v-if="loading" icon="mdi:loading" class="w-4 h-4 animate-spin" />
                             <span v-else>{{ imagesData.size }}</span>
                         </span>
                     </div>
                     <div class="flex items-center justify-between">
                         <span class="text-sm">Dangling:</span>
                         <span class="text-sm font-medium">
                             <Icon v-if="loading" icon="mdi:loading" class="w-4 h-4 animate-spin" />
                             <span v-else>{{ imagesData.dangling }}</span>
                         </span>
                     </div>
                 </div>
                 <Button 
                     @click="viewImages"
                     class="w-full mt-4"
                 >
                     View Images
                 </Button>
             </div>
         </div>
    </div>

    <EditServerModal
        v-model:open="editModalOpen"
        :server="server"
    /> 
</template>
