<script setup lang="ts">
import { Icon } from "@iconify/vue";
import Server from "@/class/Server";
import { ref, onMounted } from "vue";

const props = defineProps<{
    server: Server;
}>();

const dockerState = ref<string>("Loading...");

onMounted(async () => {
    dockerState.value = "Loading...";
    dockerState.value = await props.server.docker.getVersion();
});
</script>

<template>
    <div class="border rounded-lg p-4 bg-card">
        <div class="flex items-center justify-between">
            <div class="flex items-center gap-3">
                <Icon icon="logos:docker-icon" class="w-8 h-8" />
                <div>
                    <h1 class="text-xl font-semibold">
                        Docker State
                    </h1>
                    <p class="text-sm text-muted-foreground">
                        {{ dockerState }}
                    </p>
                </div>
            </div>
        </div>
    </div>
</template>
