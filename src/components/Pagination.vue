<script setup lang="ts">
import IconArrow from "../assets/right.svg?component";
const prop = defineProps<{
    length: number
}>();

const index = defineModel<number>({ default: 0 })
</script>

<template>
    <div class="pagination">
        <button @click="index = index <= 0 ? 0 : index - 1" :disabled="index == 0" title="上一帧">
            <IconArrow style="rotate: 180deg; padding-left: 1px;"/>
        </button>
        <input type="hidden" v-model.number="index">
        <p>{{ `${index + 1}/${prop.length}` }}</p>
        <button @click="index = index >= length - 1 ? length - 1 : index + 1" :disabled="index == length - 1"
            title="下一帧">
            <IconArrow />
        </button>
    </div>
</template>

<style>
.pagination {
    display: flex;
    gap: 1mm;
}

.pagination > p {
    align-content: center;
    user-select: none;
    -webkit-user-select: none;
    cursor: default;
}
</style>