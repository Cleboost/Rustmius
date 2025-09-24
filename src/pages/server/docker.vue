<script setup lang="ts">
import { computed, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import ServerHeader from "@/components/ServerHeader.vue";
import { Icon } from "@iconify/vue";
import { useServerInstanceStore } from "@/stores/serverInstance";
import EditServerModal from "@/components/modal/EditServerModal.vue";
import DockerCard from "@/components/DockerCard.vue";

const router = useRouter();
const serverInstancesStore = useServerInstanceStore();

const server = computed(() =>
    serverInstancesStore.getServerInstance(useRoute().params.id as string),
);

const editModalOpen = ref(false);



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
    </div>

    <EditServerModal
        v-model:open="editModalOpen"
        :server="server"
    /> 
</template>
