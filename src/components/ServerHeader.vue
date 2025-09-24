<script setup lang="ts">
import { computed } from "vue";
import { Icon } from "@iconify/vue";
import { Button } from "@/components/ui/button";
import Server from "@/class/Server";

defineProps<{
    server: Server;
    buttons?:Boolean;
}>();

const emit = defineEmits<{
    (e: "close"): void;
    (e: "edit"): void;
}>();
</script>

<template>
    <div class="border rounded-lg p-4 bg-card">
        <div class="flex items-center justify-between">
            <div class="flex items-center gap-3">
                <Icon icon="logos:debian" class="w-8 h-8" />
                <div>
                    <h1 class="text-xl font-semibold">
                        {{ server.config().getName() }}
                    </h1>
                    <p class="text-sm text-muted-foreground">
                        IP: {{ server.config().getIP() }}
                    </p>
                </div>
            </div>
            <div class="flex items-center gap-2" v-if="buttons !== false">
                <Button
                    variant="ghost"
                    size="icon"
                    @click="emit('edit')"
                    class="text-blue-500 hover:text-blue-600 hover:bg-blue-50"
                    title="Edit server"
                >
                    <Icon icon="lucide:pencil" class="w-4 h-4" />
                </Button>

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
