<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from "vue";
import { useRoute, useRouter } from "vue-router";
import { Icon } from "@iconify/vue";
import { Button } from "@/components/ui/button";
import { useServerInstanceStore } from "@/stores/serverInstance";

const router = useRouter();
const serverInstancesStore = useServerInstanceStore();

const server = computed(() =>
    serverInstancesStore.getServerInstance(useRoute().params.id as string),
);

const systemInfo = ref<any>(null);
const systemStats = ref<any>(null);
const loading = ref(false);
const refreshInterval = ref<NodeJS.Timeout | null>(null);
async function loadSystemInfo() {
    if (!server.value) return;

    try {
        systemInfo.value = await server.value.systemMonitor.getSystemInfo();
    } catch (error) {
        console.error("Error loading system info:", error);
    }
}

async function loadSystemStats() {
    if (!server.value) return;

    try {
        const stats = await server.value.systemMonitor.getSystemStats();
        systemStats.value = stats;
    } catch (error) {
        console.error("Error loading system stats:", error);
    }
}

function startLiveMonitoring() {
    if (refreshInterval.value) {
        clearInterval(refreshInterval.value);
    }

    loadSystemStats();

    refreshInterval.value = setInterval(() => {
        loadSystemStats();
    }, 2000);

    console.log("Live monitoring started");
}

function stopLiveMonitoring() {
    if (refreshInterval.value) {
        clearInterval(refreshInterval.value);
        refreshInterval.value = null;
        console.log("Live monitoring stopped");
    }
}

function formatBytes(bytes: number): string {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
}

function getUsageColor(percentage: number): string {
    if (percentage < 50) return "text-green-500";
    if (percentage < 80) return "text-yellow-500";
    return "text-red-500";
}

function getUsageBgColor(percentage: number): string {
    if (percentage < 50) return "bg-green-500";
    if (percentage < 80) return "bg-yellow-500";
    return "bg-red-500";
}

onMounted(async () => {
    await loadSystemInfo();
    await loadSystemStats();
    startLiveMonitoring();
});

onUnmounted(() => {
    stopLiveMonitoring();
});
</script>

<template>
    <div
        class="flex flex-col gap-6 max-w-full h-full min-h-0 p-4 overflow-y-auto"
    >
        <div class="flex items-center gap-4">
            <button
                @click="router.back()"
                class="flex items-center justify-center w-10 h-10 rounded-lg bg-gray-100 hover:bg-gray-200 dark:bg-gray-800 dark:hover:bg-gray-700 transition-colors"
            >
                <Icon icon="mdi:arrow-left" class="w-5 h-5" />
            </button>
            <div class="flex items-center gap-3">
                <Icon icon="mdi:chart-line" class="w-8 h-8 text-blue-500" />
                <div>
                    <h1 class="text-2xl font-semibold">System Monitor</h1>
                    <p class="text-sm text-muted-foreground">
                        {{ server?.config().get().name }}
                    </p>
                </div>
            </div>
        </div>

        <div
            v-if="refreshInterval"
            class="flex items-center justify-center gap-2 text-sm text-muted-foreground bg-green-50 dark:bg-green-900/20 p-3 rounded-lg border border-green-200 dark:border-green-800"
        >
            <Icon
                icon="mdi:loading"
                class="w-4 h-4 animate-spin text-green-500"
            />
            <span>Live monitoring active - updating every 2 seconds</span>
        </div>

        <div v-if="loading" class="flex items-center justify-center py-8">
            <Icon icon="mdi:loading" class="w-6 h-6 animate-spin" />
            <span class="ml-2">Loading system information...</span>
        </div>

        <div v-else-if="systemInfo && systemStats" class="space-y-6">
            <div class="border rounded-lg p-6 bg-card">
                <h2 class="text-xl font-semibold mb-4 flex items-center gap-2">
                    <Icon icon="mdi:information" class="w-5 h-5" />
                    System Information
                </h2>
                <div
                    class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4"
                >
                    <div class="space-y-1">
                        <div class="text-sm text-muted-foreground">
                            Hostname
                        </div>
                        <div class="font-medium">{{ systemInfo.hostname }}</div>
                    </div>
                    <div class="space-y-1">
                        <div class="text-sm text-muted-foreground">Uptime</div>
                        <div class="font-medium">{{ systemInfo.uptime }}</div>
                    </div>
                    <div class="space-y-1">
                        <div class="text-sm text-muted-foreground">OS</div>
                        <div class="font-medium">{{ systemInfo.os }}</div>
                    </div>
                    <div class="space-y-1">
                        <div class="text-sm text-muted-foreground">Kernel</div>
                        <div class="font-medium">{{ systemInfo.kernel }}</div>
                    </div>
                    <div class="space-y-1">
                        <div class="text-sm text-muted-foreground">
                            Architecture
                        </div>
                        <div class="font-medium">
                            {{ systemInfo.architecture }}
                        </div>
                    </div>
                </div>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                <div class="border rounded-lg p-4 bg-card">
                    <div class="flex items-center justify-between mb-3">
                        <div class="flex items-center gap-2">
                            <Icon
                                icon="mdi:cpu-64-bit"
                                class="w-5 h-5 text-blue-500"
                            />
                            <span class="font-medium">CPU</span>
                        </div>
                        <span
                            :class="[
                                'text-sm font-semibold',
                                getUsageColor(systemStats.cpu.usage),
                            ]"
                        >
                            {{ systemStats.cpu.usage.toFixed(1) }}%
                        </span>
                    </div>
                    <div class="space-y-2">
                        <div class="flex justify-between text-sm">
                            <span class="text-muted-foreground">Cores:</span>
                            <span>{{ systemStats.cpu.cores }}</span>
                        </div>
                        <div class="flex justify-between text-sm">
                            <span class="text-muted-foreground">Load:</span>
                            <span>{{
                                systemStats.cpu.load[0].toFixed(2)
                            }}</span>
                        </div>
                        <div class="mt-2 relative">
                            <svg
                                width="100%"
                                height="40"
                                class="border rounded bg-gray-50 dark:bg-gray-900"
                            >
                                <path
                                    d="M 2,35 Q 25,30 50,25 T 100,30 T 150,20 T 200,28"
                                    fill="none"
                                    stroke="#3b82f6"
                                    stroke-width="2"
                                    opacity="0.4"
                                    stroke-linecap="round"
                                />
                                <path
                                    d="M 2,32 Q 25,27 50,22 T 100,27 T 150,17 T 200,25"
                                    fill="none"
                                    stroke="#3b82f6"
                                    stroke-width="1.5"
                                    opacity="0.25"
                                    stroke-linecap="round"
                                />
                                <path
                                    d="M 2,38 Q 25,33 50,28 T 100,33 T 150,23 T 200,31"
                                    fill="none"
                                    stroke="#3b82f6"
                                    stroke-width="1"
                                    opacity="0.15"
                                    stroke-linecap="round"
                                />
                            </svg>
                            <div
                                class="absolute inset-0 flex items-center justify-center bg-white/95 dark:bg-gray-900/95 backdrop-blur-md rounded"
                            >
                                <span
                                    class="text-xs font-medium text-gray-700 dark:text-gray-300"
                                    >Soon...</span
                                >
                            </div>
                        </div>
                    </div>
                </div>
                <div class="border rounded-lg p-4 bg-card">
                    <div class="flex items-center justify-between mb-3">
                        <div class="flex items-center gap-2">
                            <Icon
                                icon="mdi:memory"
                                class="w-5 h-5 text-green-500"
                            />
                            <span class="font-medium">Memory</span>
                        </div>
                        <span
                            :class="[
                                'text-sm font-semibold',
                                getUsageColor(systemStats.memory.percentage),
                            ]"
                        >
                            {{ systemStats.memory.percentage.toFixed(1) }}%
                        </span>
                    </div>
                    <div class="space-y-2">
                        <div class="flex justify-between text-sm">
                            <span class="text-muted-foreground">Used:</span>
                            <span>{{ systemStats.memory.used }}MB</span>
                        </div>
                        <div class="flex justify-between text-sm">
                            <span class="text-muted-foreground">Total:</span>
                            <span>{{ systemStats.memory.total }}MB</span>
                        </div>
                        <div class="mt-2 relative">
                            <svg
                                width="100%"
                                height="40"
                                class="border rounded bg-gray-50 dark:bg-gray-900"
                            >
                                <path
                                    d="M 2,25 Q 25,35 50,30 T 100,20 T 150,35 T 200,15"
                                    fill="none"
                                    stroke="#10b981"
                                    stroke-width="2"
                                    opacity="0.4"
                                    stroke-linecap="round"
                                />
                                <path
                                    d="M 2,28 Q 25,38 50,33 T 100,23 T 150,38 T 200,18"
                                    fill="none"
                                    stroke="#10b981"
                                    stroke-width="1.5"
                                    opacity="0.25"
                                    stroke-linecap="round"
                                />
                                <path
                                    d="M 2,22 Q 25,32 50,27 T 100,17 T 150,32 T 200,12"
                                    fill="none"
                                    stroke="#10b981"
                                    stroke-width="1"
                                    opacity="0.15"
                                    stroke-linecap="round"
                                />
                            </svg>
                            <div
                                class="absolute inset-0 flex items-center justify-center bg-white/95 dark:bg-gray-900/95 backdrop-blur-md rounded"
                            >
                                <span
                                    class="text-xs font-medium text-gray-700 dark:text-gray-300"
                                    >Soon...</span
                                >
                            </div>
                        </div>
                    </div>
                </div>

                <div class="border rounded-lg p-4 bg-card">
                    <div class="flex items-center justify-between mb-3">
                        <div class="flex items-center gap-2">
                            <Icon
                                icon="mdi:harddisk"
                                class="w-5 h-5 text-purple-500"
                            />
                            <span class="font-medium">Disk</span>
                        </div>
                        <span
                            :class="[
                                'text-sm font-semibold',
                                getUsageColor(systemStats.disk.percentage),
                            ]"
                        >
                            {{ systemStats.disk.percentage.toFixed(1) }}%
                        </span>
                    </div>
                    <div class="space-y-2">
                        <div class="flex justify-between text-sm">
                            <span class="text-muted-foreground">Used:</span>
                            <span
                                >{{ systemStats.disk.used.toFixed(1) }}GB</span
                            >
                        </div>
                        <div class="flex justify-between text-sm">
                            <span class="text-muted-foreground">Total:</span>
                            <span
                                >{{ systemStats.disk.total.toFixed(1) }}GB</span
                            >
                        </div>
                        <div class="mt-2 relative">
                            <svg
                                width="100%"
                                height="40"
                                class="border rounded bg-gray-50 dark:bg-gray-900"
                            >
                                <path
                                    d="M 2,20 Q 25,10 50,15 T 100,25 T 150,10 T 200,20"
                                    fill="none"
                                    stroke="#8b5cf6"
                                    stroke-width="2"
                                    opacity="0.4"
                                    stroke-linecap="round"
                                />
                                <path
                                    d="M 2,23 Q 25,13 50,18 T 100,28 T 150,13 T 200,23"
                                    fill="none"
                                    stroke="#8b5cf6"
                                    stroke-width="1.5"
                                    opacity="0.25"
                                    stroke-linecap="round"
                                />
                                <path
                                    d="M 2,17 Q 25,7 50,12 T 100,22 T 150,7 T 200,17"
                                    fill="none"
                                    stroke="#8b5cf6"
                                    stroke-width="1"
                                    opacity="0.15"
                                    stroke-linecap="round"
                                />
                            </svg>
                            <div
                                class="absolute inset-0 flex items-center justify-center bg-white/95 dark:bg-gray-900/95 backdrop-blur-md rounded"
                            >
                                <span
                                    class="text-xs font-medium text-gray-700 dark:text-gray-300"
                                    >Soon...</span
                                >
                            </div>
                        </div>
                    </div>
                </div>

                <div class="border rounded-lg p-4 bg-card">
                    <div class="flex items-center justify-between mb-3">
                        <div class="flex items-center gap-2">
                            <Icon
                                icon="mdi:network"
                                class="w-5 h-5 text-orange-500"
                            />
                            <span class="font-medium">Network</span>
                        </div>
                        <span class="text-sm font-semibold text-orange-500">
                            Active
                        </span>
                    </div>
                    <div class="space-y-2">
                        <div class="flex justify-between text-sm">
                            <span class="text-muted-foreground">RX:</span>
                            <span>{{
                                formatBytes(systemStats.network.rx)
                            }}</span>
                        </div>
                        <div class="flex justify-between text-sm">
                            <span class="text-muted-foreground">TX:</span>
                            <span>{{
                                formatBytes(systemStats.network.tx)
                            }}</span>
                        </div>
                        <div class="mt-2 relative">
                            <svg
                                width="100%"
                                height="40"
                                class="border rounded bg-gray-50 dark:bg-gray-900"
                            >
                                <path
                                    d="M 2,30 Q 25,15 50,20 T 100,35 T 150,15 T 200,25"
                                    fill="none"
                                    stroke="#f97316"
                                    stroke-width="2"
                                    opacity="0.4"
                                    stroke-linecap="round"
                                />
                                <path
                                    d="M 2,33 Q 25,18 50,23 T 100,38 T 150,18 T 200,28"
                                    fill="none"
                                    stroke="#f97316"
                                    stroke-width="1.5"
                                    opacity="0.25"
                                    stroke-linecap="round"
                                />
                                <path
                                    d="M 2,27 Q 25,12 50,17 T 100,32 T 150,12 T 200,22"
                                    fill="none"
                                    stroke="#f97316"
                                    stroke-width="1"
                                    opacity="0.15"
                                    stroke-linecap="round"
                                />
                            </svg>
                            <div
                                class="absolute inset-0 flex items-center justify-center bg-white/95 dark:bg-gray-900/95 backdrop-blur-md rounded"
                            >
                                <span
                                    class="text-xs font-medium text-gray-700 dark:text-gray-300"
                                    >Soon...</span
                                >
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <div class="flex items-center justify-between">
                <div class="flex items-center gap-2">
                    <span class="text-sm text-muted-foreground">
                        Live monitoring
                        {{ refreshInterval ? "active" : "inactive" }}
                    </span>
                </div>
                <div class="flex items-center gap-2">
                    <Button
                        @click="
                            refreshInterval
                                ? stopLiveMonitoring()
                                : startLiveMonitoring()
                        "
                        :variant="refreshInterval ? 'outline' : 'default'"
                        size="sm"
                    >
                        <Icon
                            :icon="refreshInterval ? 'mdi:pause' : 'mdi:play'"
                            class="w-4 h-4 mr-2"
                        />
                        {{
                            refreshInterval
                                ? "Pause Monitoring"
                                : "Start Monitoring"
                        }}
                    </Button>
                    <Button
                        @click="loadSystemStats"
                        :disabled="loading"
                        variant="outline"
                        size="sm"
                    >
                        <Icon icon="mdi:refresh" class="w-4 h-4 mr-2" />
                        {{ loading ? "Loading..." : "Refresh" }}
                    </Button>
                </div>
            </div>
        </div>
    </div>
</template>
