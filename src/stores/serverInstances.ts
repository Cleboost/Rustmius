import { defineStore } from "pinia";
import { computed, ref } from "vue";
import Server from "@/class/Server";

export const useServerInstancesStore = defineStore("serverInstances", () => {
  const instancesArray = ref<any[]>([]);
  const loadedInstances = ref<Set<string>>(new Set());

  function getServerInstance(serverId: string): Server {
    const existing = instancesArray.value.find(server => server.id === serverId);
    if (existing) return existing;

    const serverInstance = new Server(serverId);
    console.log('Created server instance (not added to sidebar yet):', serverId);
    return serverInstance;
  }

  function addToSidebar(serverId: string): void {
    if (loadedInstances.value.has(serverId)) return;
    
    const existing = instancesArray.value.find(server => server.id === serverId);
    if (!existing) {
      const serverInstance = new Server(serverId);
      instancesArray.value.push(serverInstance);
      loadedInstances.value.add(serverId);
      console.log('Added server instance to sidebar:', serverId, 'total:', instancesArray.value.length);
    }
  }

  const sidebarInstances = computed(() => {
    const result = instancesArray.value.map((server) => ({
      id: server.id,
      name: server.getName(),
      icon: "server",
      route: `/server/${server.id}`,
    }));
    console.log('sidebarInstances computed called, result:', result);
    return result;
  });

  function removeServerInstance(serverId: string) {
    console.log('Removing server instance:', serverId);
    console.log('Instances before removal:', instancesArray.value.length);
    
    const index = instancesArray.value.findIndex(server => server.id === serverId);
    if (index !== -1) {
      instancesArray.value.splice(index, 1);
      loadedInstances.value.delete(serverId);
      console.log('Instances after removal:', instancesArray.value.length);
    } else {
      console.log('Server instance not found:', serverId);
    }
  }

  return {
    instancesArray,
    getServerInstance,
    addToSidebar,
    sidebarInstances,
    removeServerInstance,
  };
});
