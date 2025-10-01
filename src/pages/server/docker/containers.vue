<script setup lang="ts">
import { computed, ref, onMounted } from "vue";
import { useRoute, useRouter } from "vue-router";
import { Icon } from "@iconify/vue";
import { Button } from "@/components/ui/button";
import { useServerInstanceStore } from "@/stores/serverInstance";

const router = useRouter();
const serverInstancesStore = useServerInstanceStore();

const server = computed(() =>
    serverInstancesStore.getServerInstance(useRoute().params.id as string),
);

const containers = ref<any[]>([]);
const loading = ref(false);
const actionLoading = ref<string | null>(null);

async function loadContainers() {
    if (!server.value) return;

    loading.value = true;
    try {
        const output = await server.value.console.execute(
            "docker ps -a --format '{{.Names}}|{{.Image}}|{{.Status}}|{{.Ports}}|{{.ID}}'",
        );
        const lines = output.split("\n").filter((line) => line.trim());

        containers.value = lines.map((line) => {
            const [name, image, status, ports, id] = line.split("|");
            return {
                name: name || "Unnamed",
                image,
                status,
                ports: ports || "No ports",
                id: id.substring(0, 12),
                fullId: id,
                isRunning: status.includes("Up"),
                isStopped:
                    status.includes("Exited") || status.includes("Created"),
            };
        });
    } catch (error) {
        console.error("Error loading containers:", error);
    } finally {
        loading.value = false;
    }
}

async function startContainer(containerId: string) {
    if (!server.value) return;

    actionLoading.value = containerId;
    try {
        await server.value.console.execute(`docker start ${containerId}`);
        await loadContainers();
    } catch (error) {
        console.error("Error starting container:", error);
        alert("Failed to start container");
    } finally {
        actionLoading.value = null;
    }
}

async function stopContainer(containerId: string) {
    if (!server.value) return;

    actionLoading.value = containerId;
    try {
        await server.value.console.execute(`docker stop ${containerId}`);
        await loadContainers();
    } catch (error) {
        console.error("Error stopping container:", error);
        alert("Failed to stop container");
    } finally {
        actionLoading.value = null;
    }
}

async function killContainer(containerId: string) {
    if (!server.value) return;

    actionLoading.value = containerId;
    try {
        await server.value.console.execute(`docker kill ${containerId}`);
        await loadContainers();
    } catch (error) {
        console.error("Error killing container:", error);
        alert("Failed to kill container");
    } finally {
        actionLoading.value = null;
    }
}

function viewContainerDetails(containerId: string) {
    router.push(`/server/${server.value?.id}/docker/container/${containerId}`);
}

function getStatusColor(status: string) {
    if (status.includes("Up")) return "text-green-500";
    if (status.includes("Exited")) return "text-red-500";
    if (status.includes("Created")) return "text-yellow-500";
    return "text-gray-500";
}

function getStatusIcon(status: string) {
    if (status.includes("Up")) return "mdi:play-circle";
    if (status.includes("Exited")) return "mdi:stop-circle";
    if (status.includes("Created")) return "mdi:help-circle";
    return "mdi:alert-circle";
}

onMounted(() => {
    loadContainers();
});
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
            <div class="flex items-center gap-3">
                <Icon icon="mdi:docker" class="w-8 h-8 text-blue-500" />
                <div>
                    <h1 class="text-2xl font-semibold">Docker Containers</h1>
                    <p class="text-sm text-muted-foreground">
                        {{ server?.config().get().name }}
                    </p>
                </div>
            </div>
        </div>

        <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
                <span class="text-sm text-muted-foreground">
                    {{ containers.length }} container{{
                        containers.length !== 1 ? "s" : ""
                    }}
                </span>
            </div>
            <Button
                @click="loadContainers"
                :disabled="loading"
                variant="outline"
                size="sm"
            >
                <Icon icon="mdi:refresh" class="w-4 h-4 mr-2" />
                {{ loading ? "Loading..." : "Refresh" }}
            </Button>
        </div>

        <div class="flex-1 overflow-auto">
            <div v-if="loading" class="flex items-center justify-center py-8">
                <Icon icon="mdi:loading" class="w-6 h-6 animate-spin" />
                <span class="ml-2">Loading containers...</span>
            </div>

            <div
                v-else-if="containers.length === 0"
                class="text-center py-8 text-muted-foreground"
            >
                <Icon
                    icon="mdi:docker"
                    class="w-12 h-12 mx-auto mb-4 opacity-50"
                />
                <p>No Docker containers found</p>
            </div>

            <div
                v-else
                class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4"
            >
                <div
                    v-for="container in containers"
                    :key="container.fullId"
                    class="border rounded-lg p-4 hover:shadow-md transition-shadow bg-card"
                >
                    <div class="flex items-center gap-3 mb-3">
                        <Icon
                            :icon="getStatusIcon(container.status)"
                            :class="[
                                'w-6 h-6',
                                getStatusColor(container.status),
                            ]"
                        />
                        <div class="flex-1 min-w-0">
                            <h3 class="font-medium truncate">
                                {{ container.name }}
                            </h3>
                            <p class="text-sm text-muted-foreground truncate">
                                {{ container.image }}
                            </p>
                        </div>
                    </div>

                    <div class="space-y-2 mb-4">
                        <div class="flex items-center justify-between text-sm">
                            <span class="text-muted-foreground">Status:</span>
                            <span
                                :class="[
                                    'font-medium',
                                    getStatusColor(container.status),
                                ]"
                            >
                                {{ container.status }}
                            </span>
                        </div>
                        <div class="flex items-center justify-between text-sm">
                            <span class="text-muted-foreground">ID:</span>
                            <span class="font-mono text-xs">{{
                                container.id
                            }}</span>
                        </div>
                        <div class="flex items-center justify-between text-sm">
                            <span class="text-muted-foreground">Ports:</span>
                            <span class="text-xs truncate ml-2">{{
                                container.ports
                            }}</span>
                        </div>
                    </div>

                    <div class="flex items-center gap-2">
                        <Button
                            v-if="container.isStopped"
                            @click="startContainer(container.fullId)"
                            :disabled="actionLoading === container.fullId"
                            size="sm"
                            class="flex-1 bg-green-500 hover:bg-green-600 text-white"
                        >
                            <Icon
                                v-if="actionLoading === container.fullId"
                                icon="mdi:loading"
                                class="w-4 h-4 animate-spin"
                            />
                            <Icon v-else icon="mdi:play" class="w-4 h-4" />
                        </Button>

                        <Button
                            v-if="container.isRunning"
                            @click="stopContainer(container.fullId)"
                            :disabled="actionLoading === container.fullId"
                            size="sm"
                            variant="outline"
                            class="flex-1 text-orange-500 hover:text-orange-700 hover:bg-orange-50"
                        >
                            <Icon
                                v-if="actionLoading === container.fullId"
                                icon="mdi:loading"
                                class="w-4 h-4 animate-spin"
                            />
                            <Icon v-else icon="mdi:stop" class="w-4 h-4" />
                        </Button>

                        <Button
                            v-if="container.isRunning"
                            @click="killContainer(container.fullId)"
                            :disabled="actionLoading === container.fullId"
                            size="sm"
                            variant="outline"
                            class="text-red-500 hover:text-red-700 hover:bg-red-50"
                        >
                            <Icon
                                v-if="actionLoading === container.fullId"
                                icon="mdi:loading"
                                class="w-4 h-4 animate-spin"
                            />
                            <Icon
                                v-else
                                icon="mdi:close-circle"
                                class="w-4 h-4"
                            />
                        </Button>

                        <Button
                            @click="viewContainerDetails(container.fullId)"
                            size="sm"
                            variant="outline"
                            class="text-blue-500 hover:text-blue-700 hover:bg-blue-50"
                        >
                            <Icon icon="mdi:eye" class="w-4 h-4" />
                        </Button>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>
