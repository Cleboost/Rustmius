<script setup lang="ts">
import { computed } from "vue";
import { useServerInstancesStore } from "@/stores/serverInstances";
import { Icon } from "@iconify/vue";
import { Button } from "@/components/ui/button";

const props = defineProps<{
    serverId: string;
}>();

const emit = defineEmits<{
    (e: "close"): void;
    (e: "edit"): void;
}>();

const serverInstancesStore = useServerInstancesStore();

const serverInstance = computed(() => {
    return serverInstancesStore.instancesArray.find(
        (s) => s.id === props.serverId,
    );
});
const serverName = computed(() => serverInstance.value?.getName() || "Unknown");
const serverIp = computed(() => serverInstance.value?.getIP() || "Unknown");
</script>

<template>
    <div class="border rounded-lg p-4 bg-card">
        <div class="flex items-center justify-between">
            <div class="flex items-center gap-3">
                <Icon icon="logos:debian" class="w-8 h-8" />
                <div>
                    <h1 class="text-xl font-semibold">{{ serverName }}</h1>
                    <p class="text-sm text-muted-foreground">
                        IP: {{ serverIp }}
                    </p>
                </div>
            </div>
            <div class="flex items-center gap-2">
                <!-- Edit Button -->
                <Button
                    variant="ghost"
                    size="icon"
                    @click="emit('edit')"
                    class="text-blue-500 hover:text-blue-600 hover:bg-blue-50"
                    title="Edit server"
                >
                    <Icon icon="lucide:pencil" class="w-4 h-4" />
                </Button>

                <!-- Close Button -->
                <Button
                    variant="ghost"
                    size="icon"
                    @click="emit('close')"
                    class="text-muted-foreground hover:text-foreground"
                    title="Close server"
                >
                    <Icon icon="lucide:x" class="w-4 h-4" />
                </Button>
            </div>
        </div>
    </div>
</template>
