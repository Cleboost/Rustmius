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

const images = ref<any[]>([]);
const loading = ref(false);
const deleting = ref<string | null>(null);

async function loadImages() {
    if (!server.value) return;

    loading.value = true;
    try {
        const output = await server.value.console.execute(
            "docker images --format '{{.Repository}}:{{.Tag}}|{{.ID}}|{{.Size}}|{{.CreatedAt}}'",
        );
        const lines = output.split("\n").filter((line) => line.trim());

        images.value = lines.map((line) => {
            const [repository, id, size, createdAt] = line.split("|");
            return {
                repository: repository || "<none>",
                tag: repository.includes(":")
                    ? repository.split(":")[1]
                    : "latest",
                id: id.substring(0, 12),
                fullId: id,
                size,
                createdAt: createdAt
                    ? new Date(createdAt).toLocaleDateString()
                    : "Unknown",
            };
        });
    } catch (error) {
        console.error("Error loading images:", error);
    } finally {
        loading.value = false;
    }
}

async function deleteImage(imageId: string) {
    if (!server.value) return;

    deleting.value = imageId;
    try {
        await server.value.console.execute(`docker rmi ${imageId}`);
        await loadImages();
    } catch (error) {
        console.error("Error deleting image:", error);
        alert("Failed to delete image. It might be in use by a container.");
    } finally {
        deleting.value = null;
    }
}

onMounted(() => {
    loadImages();
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
                <Icon
                    icon="mdi:image-multiple"
                    class="w-8 h-8 text-green-500"
                />
                <div>
                    <h1 class="text-2xl font-semibold">Docker Images</h1>
                    <p class="text-sm text-muted-foreground">
                        {{ server?.config().get().name }}
                    </p>
                </div>
            </div>
        </div>

        <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
                <span class="text-sm text-muted-foreground">
                    {{ images.length }} image{{
                        images.length !== 1 ? "s" : ""
                    }}
                </span>
            </div>
            <Button
                @click="loadImages"
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
                <span class="ml-2">Loading images...</span>
            </div>

            <div
                v-else-if="images.length === 0"
                class="text-center py-8 text-muted-foreground"
            >
                <Icon
                    icon="mdi:image-multiple-outline"
                    class="w-12 h-12 mx-auto mb-4 opacity-50"
                />
                <p>No Docker images found</p>
            </div>

            <div v-else class="space-y-3">
                <div
                    v-for="image in images"
                    :key="image.fullId"
                    class="border rounded-lg p-4 hover:shadow-sm transition-shadow"
                >
                    <div class="flex items-center justify-between">
                        <div class="flex items-center gap-3">
                            <Icon
                                icon="mdi:docker"
                                class="w-6 h-6 text-blue-500"
                            />
                            <div>
                                <h3 class="font-medium">
                                    {{ image.repository }}
                                </h3>
                                <div
                                    class="flex items-center gap-4 text-sm text-muted-foreground"
                                >
                                    <span>ID: {{ image.id }}</span>
                                    <span>Size: {{ image.size }}</span>
                                    <span>Created: {{ image.createdAt }}</span>
                                </div>
                            </div>
                        </div>
                        <div class="flex items-center gap-2">
                            <Button
                                @click="deleteImage(image.fullId)"
                                :disabled="deleting === image.fullId"
                                variant="outline"
                                size="sm"
                                class="text-red-500 hover:text-red-700 hover:bg-red-50"
                            >
                                <Icon
                                    v-if="deleting === image.fullId"
                                    icon="mdi:loading"
                                    class="w-4 h-4 animate-spin"
                                />
                                <Icon
                                    v-else
                                    icon="mdi:delete"
                                    class="w-4 h-4"
                                />
                            </Button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>
