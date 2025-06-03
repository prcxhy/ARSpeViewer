<script setup lang="ts">
const prop = defineProps<{
    name: string,
    mode1: string,
    mode2: string
}>();

const state = defineModel<boolean>()

</script>

<template>
    <div class="mode-switch">
        <p>{{ prop.name }}</p>
        <div class="mode-switch-slot">
            <p :class="['switch-mode', state? 'mode-active': '']" @click="state = true">{{ prop.mode1 }}</p>
            <p :class="['switch-mode', !state? 'mode-active': ''] " @click="state = false">{{ prop.mode2}}</p>
        </div>
    </div>
</template>

<style>
.switch-mode {
    padding: 1mm;
    border-radius: 1mm;
    color: rgb(128, 128, 128);
    font-weight: bold;
    transition: 0.3s;
}

.switch-mode:hover {
    background-color: rgb(192, 192, 192);
}

.mode-active {
    color: rgb(64, 64, 64);
    background-color: rgb(250, 250, 250);
}

.mode-active:hover {
    background-color: white;
    filter: drop-shadow(0px 0px 8px rgba(0, 0, 0, 0.1))
}

.mode-switch {
    display: flex;
    gap: 2mm;
    align-items: center;
    height: 1cm;
    user-select: none;
}

.mode-switch-slot {
    background-color: rgb(204, 204, 204);
    display: flex;
    padding: 1mm;
    border-radius: 2mm;
    font-size: 4mm;
}
</style>