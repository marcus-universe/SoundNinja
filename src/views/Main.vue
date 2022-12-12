<template>
  <div class="main flex_c_v gap1 flex_start" >
    <Tabs />
    <SoundContainer />
  </div>
</template>

<script>
import { defineComponent } from 'vue';
import Tabs from '@/components/Section/Tabs.vue';
import SoundContainer from '@/components/Section/SoundContainer.vue';
export default defineComponent({
  name: 'Main',
  components: {
    Tabs,
    SoundContainer
  },
});
</script>
