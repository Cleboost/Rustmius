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

const containerId = computed(() => useRoute().params.cid as string);

const containerInfo = ref<any>(null);
const containerStats = ref<any>(null);
const containerLogs = ref<string>("");
const loading = ref(false);
const logsLoading = ref(false);
const actionLoading = ref<string | null>(null);
const statsInterval = ref<NodeJS.Timeout | null>(null);

const chartData = ref({
    cpu: [] as number[],
    memory: [] as number[],
    network: [] as number[],
    io: [] as number[],
});

const maxDataPoints = 60;

const logsContainer = ref<HTMLElement | null>(null);

const visibleEnvVars = ref<Set<string>>(new Set());

async function loadContainerInfo() {
    if (!server.value || !containerId.value) return;

    loading.value = true;
    try {
        const inspectOutput = await server.value.console.execute(
            `docker inspect ${containerId.value}`,
        );
        const inspectData = JSON.parse(inspectOutput);
        containerInfo.value = inspectData[0];
    } catch (error) {
        console.error("Error loading container info:", error);
    } finally {
        loading.value = false;
    }
}

async function loadContainerStats() {
    if (!server.value || !containerId.value) return;

    try {
        const statsOutput = await server.value.console.execute(
            `docker stats ${containerId.value} --no-stream --format "{{.CPUPerc}}|{{.MemUsage}}|{{.NetIO}}|{{.BlockIO}}"`,
        );
        const [cpuPerc, memUsage, netIO, blockIO] = statsOutput.split("|");
        containerStats.value = {
            cpuPerc: cpuPerc || "0%",
            memUsage: memUsage || "0B / 0B",
            netIO: netIO || "0B / 0B",
            blockIO: blockIO || "0B / 0B",
        };

        if (statsInterval.value) {
            updateChartData(cpuPerc, memUsage, netIO, blockIO);
        }
    } catch (error) {
        console.error("Error loading container stats:", error);
    }
}

function updateChartData(
    cpuPerc: string,
    memUsage: string,
    netIO: string,
    blockIO: string,
) {
    const cpuValue = parseFloat(cpuPerc.replace("%", "")) || 0;

    const memValue =
        parseFloat(memUsage.split(" / ")[0].replace(/[^\d.]/g, "")) || 0;

    const netValue =
        parseFloat(netIO.split(" / ")[0].replace(/[^\d.]/g, "")) || 0;

    const ioValue =
        parseFloat(blockIO.split(" / ")[0].replace(/[^\d.]/g, "")) || 0;

    chartData.value.cpu.push(cpuValue);
    chartData.value.memory.push(memValue);
    chartData.value.network.push(netValue);
    chartData.value.io.push(ioValue);

    if (chartData.value.cpu.length > maxDataPoints) {
        chartData.value.cpu.shift();
        chartData.value.memory.shift();
        chartData.value.network.shift();
        chartData.value.io.shift();
    }
}

async function loadContainerLogs() {
    if (!server.value || !containerId.value) return;

    logsLoading.value = true;
    try {
        const logsOutput = await server.value.console.execute(
            `docker logs ${containerId.value} --tail 100`,
        );
        containerLogs.value = logsOutput;

        setTimeout(() => {
            scrollLogsToBottom();
        }, 100);
    } catch (error) {
        console.error("Error loading container logs:", error);
        containerLogs.value = "Failed to load logs";
    } finally {
        logsLoading.value = false;
    }
}

function scrollLogsToBottom() {
    if (logsContainer.value) {
        logsContainer.value.scrollTop = logsContainer.value.scrollHeight;
    }
}

function isSensitiveEnvVar(envVar: string): boolean {
    const sensitiveKeywords = ['password', 'pass', 'secret', 'key', 'token', 'auth', 'credential', 'pwd'];
    const lowerEnvVar = envVar.toLowerCase();
    return sensitiveKeywords.some(keyword => lowerEnvVar.includes(keyword));
}

function toggleEnvVarVisibility(envVar: string) {
    if (visibleEnvVars.value.has(envVar)) {
        visibleEnvVars.value.delete(envVar);
    } else {
        visibleEnvVars.value.add(envVar);
    }
}

function isEnvVarVisible(envVar: string): boolean {
    return visibleEnvVars.value.has(envVar);
}

function maskEnvVarValue(envVar: string): string {
    const [key, value] = envVar.split('=');
    if (!value) return envVar;
    
    const maskedValue = '*'.repeat(Math.min(value.length, 8));
    return `${key}=${maskedValue}`;
}

function startStatsAutoRefresh() {
    if (statsInterval.value) {
        clearInterval(statsInterval.value);
    }

    clearChartData();

    loadContainerStats();

    statsInterval.value = setInterval(() => {
        loadContainerStats();
    }, 1000);

    console.log("Stats auto-refresh started");
}

function stopStatsAutoRefresh() {
    if (statsInterval.value) {
        clearInterval(statsInterval.value);
        statsInterval.value = null;
        console.log("Stats auto-refresh stopped");
    }
}

function clearChartData() {
    chartData.value.cpu = [];
    chartData.value.memory = [];
    chartData.value.network = [];
    chartData.value.io = [];
}

async function startContainer() {
    if (!server.value || !containerId.value) return;

    actionLoading.value = "start";
    try {
        await server.value.console.execute(`docker start ${containerId.value}`);
        await loadContainerInfo();
        startStatsAutoRefresh();
    } catch (error) {
        console.error("Error starting container:", error);
        alert("Failed to start container");
    } finally {
        actionLoading.value = null;
    }
}

async function stopContainer() {
    if (!server.value || !containerId.value) return;

    actionLoading.value = "stop";
    try {
        await server.value.console.execute(`docker stop ${containerId.value}`);
        await loadContainerInfo();
        stopStatsAutoRefresh();
    } catch (error) {
        console.error("Error stopping container:", error);
        alert("Failed to stop container");
    } finally {
        actionLoading.value = null;
    }
}

async function restartContainer() {
    if (!server.value || !containerId.value) return;

    actionLoading.value = "restart";
    try {
        await server.value.console.execute(
            `docker restart ${containerId.value}`,
        );
        await loadContainerInfo();
        startStatsAutoRefresh();
    } catch (error) {
        console.error("Error restarting container:", error);
        alert("Failed to restart container");
    } finally {
        actionLoading.value = null;
    }
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

function generateChartPoints(data: number[], maxValue: number): string {
    if (data.length === 0) return "";

    const width = 200;
    const height = 40;
    const padding = 2;

    const points = data.map((value, index) => {
        const x = (index / (data.length - 1)) * (width - 2 * padding) + padding;
        const y =
            height - padding - (value / maxValue) * (height - 2 * padding);
        return `${x},${y}`;
    });

    return points.join(" ");
}

onMounted(async () => {
    await loadContainerInfo();
    await loadContainerLogs();

    console.log("Container status:", containerInfo.value?.State?.Status);

    if (containerInfo.value?.State?.Status?.includes("Up")) {
        console.log("Container is running, starting auto-refresh");
        startStatsAutoRefresh();
    } else {
        console.log("Container is not running, loading stats once");
        loadContainerStats();
    }
});

onUnmounted(() => {
    stopStatsAutoRefresh();
});
</script>

<template>
    <div
        class="flex flex-col gap-4 max-w-full h-full min-h-0 p-4 overflow-y-auto"
    >
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
                    <h1 class="text-2xl font-semibold">Container Details</h1>
                    <p class="text-sm text-muted-foreground">
                        {{ server?.config().get().name }}
                    </p>
                </div>
            </div>
        </div>

        <div v-if="loading" class="flex items-center justify-center py-8">
            <Icon icon="mdi:loading" class="w-6 h-6 animate-spin" />
            <span class="ml-2">Loading container information...</span>
        </div>

        <div v-else-if="containerInfo" class="space-y-6">
            <div class="border rounded-lg p-6 bg-card">
                <div class="flex items-center justify-between mb-4">
                    <div class="flex items-center gap-3">
                        <Icon
                            :icon="getStatusIcon(containerInfo.State.Status)"
                            :class="[
                                'w-8 h-8',
                                getStatusColor(containerInfo.State.Status),
                            ]"
                        />
                        <div>
                            <h2 class="text-xl font-semibold">
                                {{ containerInfo.Name.replace("/", "") }}
                            </h2>
                            <p class="text-sm text-muted-foreground">
                                {{ containerInfo.Config.Image }}
                            </p>
                        </div>
                    </div>
                    <div class="flex items-center gap-2">
                        <Button
                            v-if="containerInfo.State.Status.includes('Exited')"
                            @click="startContainer"
                            :disabled="actionLoading === 'start'"
                            class="bg-green-500 hover:bg-green-600 text-white"
                        >
                            <Icon
                                v-if="actionLoading === 'start'"
                                icon="mdi:loading"
                                class="w-4 h-4 animate-spin mr-2"
                            />
                            <Icon v-else icon="mdi:play" class="w-4 h-4 mr-2" />
                            Start
                        </Button>
                        <Button
                            v-if="containerInfo.State.Status.includes('Up')"
                            @click="stopContainer"
                            :disabled="actionLoading === 'stop'"
                            variant="outline"
                            class="text-orange-500 hover:text-orange-700 hover:bg-orange-50"
                        >
                            <Icon
                                v-if="actionLoading === 'stop'"
                                icon="mdi:loading"
                                class="w-4 h-4 animate-spin mr-2"
                            />
                            <Icon v-else icon="mdi:stop" class="w-4 h-4 mr-2" />
                            Stop
                        </Button>
                        <Button
                            @click="restartContainer"
                            :disabled="actionLoading === 'restart'"
                            variant="outline"
                            class="text-blue-500 hover:text-blue-700 hover:bg-blue-50"
                        >
                            <Icon
                                v-if="actionLoading === 'restart'"
                                icon="mdi:loading"
                                class="w-4 h-4 animate-spin mr-2"
                            />
                            <Icon
                                v-else
                                icon="mdi:restart"
                                class="w-4 h-4 mr-2"
                            />
                            Restart
                        </Button>
                        <Button
                            @click="
                                statsInterval
                                    ? stopStatsAutoRefresh()
                                    : startStatsAutoRefresh()
                            "
                            variant="outline"
                            size="sm"
                            class="text-purple-500 hover:text-purple-700 hover:bg-purple-50"
                        >
                            <Icon
                                :icon="statsInterval ? 'mdi:pause' : 'mdi:play'"
                                class="w-4 h-4 mr-2"
                            />
                            {{ statsInterval ? "Pause Stats" : "Live Stats" }}
                        </Button>
                    </div>
                </div>

                <div v-if="containerStats" class="space-y-4">
                    <div
                        v-if="statsInterval"
                        class="flex items-center justify-center gap-2 text-sm text-muted-foreground"
                    >
                        <Icon
                            icon="mdi:loading"
                            class="w-4 h-4 animate-spin text-green-500"
                        />
                        <span>Live stats updating every second</span>
                    </div>

                    <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                        <div class="text-center">
                            <div class="text-2xl font-bold text-blue-500">
                                {{ containerStats.cpuPerc }}
                            </div>
                            <div class="text-sm text-muted-foreground">
                                CPU Usage
                            </div>
                            <div
                                v-if="statsInterval && chartData.cpu.length > 1"
                                class="mt-2"
                            >
                                <svg
                                    width="100%"
                                    height="40"
                                    class="border rounded"
                                >
                                    <polyline
                                        :points="
                                            generateChartPoints(
                                                chartData.cpu,
                                                100,
                                            )
                                        "
                                        fill="none"
                                        stroke="#3b82f6"
                                        stroke-width="2"
                                    />
                                </svg>
                            </div>
                        </div>
                        <div class="text-center">
                            <div class="text-2xl font-bold text-green-500">
                                {{ containerStats.memUsage.split(" / ")[0] }}
                            </div>
                            <div class="text-sm text-muted-foreground">
                                Memory Used
                            </div>
                            <div
                                v-if="
                                    statsInterval && chartData.memory.length > 1
                                "
                                class="mt-2"
                            >
                                <svg
                                    width="100%"
                                    height="40"
                                    class="border rounded"
                                >
                                    <polyline
                                        :points="
                                            generateChartPoints(
                                                chartData.memory,
                                                Math.max(...chartData.memory) ||
                                                    1,
                                            )
                                        "
                                        fill="none"
                                        stroke="#10b981"
                                        stroke-width="2"
                                    />
                                </svg>
                            </div>
                        </div>
                        <div class="text-center">
                            <div class="text-2xl font-bold text-purple-500">
                                {{ containerStats.netIO.split(" / ")[0] }}
                            </div>
                            <div class="text-sm text-muted-foreground">
                                Network RX
                            </div>
                            <div
                                v-if="
                                    statsInterval &&
                                    chartData.network.length > 1
                                "
                                class="mt-2"
                            >
                                <svg
                                    width="100%"
                                    height="40"
                                    class="border rounded"
                                >
                                    <polyline
                                        :points="
                                            generateChartPoints(
                                                chartData.network,
                                                Math.max(
                                                    ...chartData.network,
                                                ) || 1,
                                            )
                                        "
                                        fill="none"
                                        stroke="#8b5cf6"
                                        stroke-width="2"
                                    />
                                </svg>
                            </div>
                        </div>
                        <div class="text-center">
                            <div class="text-2xl font-bold text-orange-500">
                                {{ containerStats.blockIO.split(" / ")[0] }}
                            </div>
                            <div class="text-sm text-muted-foreground">
                                Block I/O
                            </div>
                            <div
                                v-if="statsInterval && chartData.io.length > 1"
                                class="mt-2"
                            >
                                <svg
                                    width="100%"
                                    height="40"
                                    class="border rounded"
                                >
                                    <polyline
                                        :points="
                                            generateChartPoints(
                                                chartData.io,
                                                Math.max(...chartData.io) || 1,
                                            )
                                        "
                                        fill="none"
                                        stroke="#f97316"
                                        stroke-width="2"
                                    />
                                </svg>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div class="border rounded-lg p-4 bg-card">
                    <h3
                        class="text-lg font-semibold mb-4 flex items-center gap-2"
                    >
                        <Icon icon="mdi:information" class="w-5 h-5" />
                        Basic Information
                    </h3>
                    <div class="space-y-3">
                        <div class="flex justify-between">
                            <span class="text-muted-foreground"
                                >Container ID:</span
                            >
                            <span class="font-mono text-sm">{{
                                containerId
                            }}</span>
                        </div>
                        <div class="flex justify-between">
                            <span class="text-muted-foreground">Status:</span>
                            <span
                                :class="[
                                    'font-medium',
                                    getStatusColor(containerInfo.State.Status),
                                ]"
                            >
                                {{ containerInfo.State.Status }}
                            </span>
                        </div>
                        <div class="flex justify-between">
                            <span class="text-muted-foreground">Created:</span>
                            <span class="text-sm">{{
                                new Date(containerInfo.Created).toLocaleString()
                            }}</span>
                        </div>
                        <div class="flex justify-between">
                            <span class="text-muted-foreground">Started:</span>
                            <span class="text-sm">{{
                                containerInfo.State.StartedAt
                                    ? new Date(
                                          containerInfo.State.StartedAt,
                                      ).toLocaleString()
                                    : "Never"
                            }}</span>
                        </div>
                    </div>
                </div>

                <div class="border rounded-lg p-4 bg-card">
                    <h3
                        class="text-lg font-semibold mb-4 flex items-center gap-2"
                    >
                        <Icon icon="mdi:network" class="w-5 h-5" />
                        Network & Ports
                    </h3>
                    <div class="space-y-3">
                        <div v-if="containerInfo.NetworkSettings?.IPAddress">
                            <div class="flex justify-between">
                                <span class="text-muted-foreground"
                                    >IP Address:</span
                                >
                                <span class="font-mono text-sm">{{
                                    containerInfo.NetworkSettings.IPAddress
                                }}</span>
                            </div>
                        </div>
                        <div v-if="containerInfo.NetworkSettings?.Ports">
                            <div class="text-muted-foreground mb-2">
                                Port Mappings:
                            </div>
                            <div
                                v-for="(
                                    portBindings, containerPort
                                ) in containerInfo.NetworkSettings.Ports"
                                :key="containerPort"
                                class="text-sm"
                            >
                                <span class="font-mono">{{
                                    containerPort
                                }}</span>
                                <span
                                    v-if="portBindings"
                                    class="text-muted-foreground"
                                >
                                    → {{ portBindings[0]?.HostPort }}:{{
                                        portBindings[0]?.HostIp
                                    }}
                                </span>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <div
                v-if="containerInfo.Config?.Env?.length"
                class="border rounded-lg p-4 bg-card"
            >
                <h3 class="text-lg font-semibold mb-4 flex items-center gap-2">
                    <Icon icon="mdi:environment" class="w-5 h-5" />
                    Environment Variables
                </h3>
                <div class="space-y-2 max-h-40 overflow-y-auto">
                    <div
                        v-for="envVar in containerInfo.Config.Env.slice(0, 10)"
                        :key="envVar"
                        class="flex items-center gap-2 text-sm font-mono bg-gray-100 dark:bg-gray-800 p-2 rounded"
                    >
                        <span class="flex-1">
                            {{ isSensitiveEnvVar(envVar) && !isEnvVarVisible(envVar) ? maskEnvVarValue(envVar) : envVar }}
                        </span>
                        <button 
                            v-if="isSensitiveEnvVar(envVar)"
                            @click="toggleEnvVarVisibility(envVar)"
                            class="flex items-center justify-center w-6 h-6 rounded hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors"
                        >
                            <Icon 
                                :icon="isEnvVarVisible(envVar) ? 'mdi:eye-off' : 'mdi:eye'" 
                                class="w-4 h-4 text-gray-600 dark:text-gray-400"
                            />
                        </button>
                    </div>
                    <div
                        v-if="containerInfo.Config.Env.length > 10"
                        class="text-sm text-muted-foreground"
                    >
                        ... and {{ containerInfo.Config.Env.length - 10 }} more
                    </div>
                </div>
            </div>

            <div class="border rounded-lg p-4 bg-card">
                <div class="flex items-center justify-between mb-4">
                    <h3 class="text-lg font-semibold flex items-center gap-2">
                        <Icon icon="mdi:file-document" class="w-5 h-5" />
                        Container Logs
                    </h3>
                    <Button
                        @click="loadContainerLogs"
                        :disabled="logsLoading"
                        variant="outline"
                        size="sm"
                    >
                        <Icon icon="mdi:refresh" class="w-4 h-4 mr-2" />
                        {{ logsLoading ? "Loading..." : "Refresh" }}
                    </Button>
                </div>
                <div
                    v-if="logsLoading"
                    class="flex items-center justify-center py-4"
                >
                    <Icon icon="mdi:loading" class="w-5 h-5 animate-spin" />
                    <span class="ml-2">Loading logs...</span>
                </div>
                <div
                    ref="logsContainer"
                    class="bg-black text-green-400 p-4 rounded font-mono text-sm max-h-60 overflow-y-auto"
                >
                    <pre>{{ containerLogs || "No logs available" }}</pre>
                </div>
            </div>
        </div>
    </div>
</template>
