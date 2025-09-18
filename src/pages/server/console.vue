<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute } from 'vue-router'
import { useConsolesStore } from '@/stores/consoles'

const route = useRoute()
const serverId = route.params.id as string
const consoles = useConsolesStore()
const session = computed(() => consoles.sessions[serverId])
const command = ref('')

async function sendCommand() {
  await consoles.send(serverId, command.value + '\n')
  command.value = ''
}
</script>

<template>
  <div class="flex flex-col h-full p-4 gap-2">
    <div class="flex-1 overflow-auto rounded-md border bg-muted p-2 font-mono text-sm whitespace-pre-wrap">{{ session?.output }}</div>
    <div class="flex gap-2">
      <input class="flex-1 rounded-md border px-3 py-2" v-model="command" @keyup.enter="sendCommand" placeholder="Type a command and press Enter" />
      <button class="rounded-md border px-3 py-2" @click="sendCommand">Send</button>
    </div>
  </div>
</template>
