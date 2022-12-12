<template>
<div class="soundninja flex_c_h flex_space_between">
  <NavBar />
  <ErrorAlert />
<router-view v-slot="{ Component }">
  <transition name="fade" mode="out-in">
    <component :is="Component" :key="$route.path" :class="{ 'searchMove': store.Searchbar.SearchbarActive }"/>
  </transition>
</router-view>
  </div>

  
</template>

<script>
import NavBar from '@/components/Section/NavBar.vue';
import ErrorAlert from '@/components/Popups/ErrorAlert.vue';

export default {
  name: 'App',
  components: {
    NavBar,
    ErrorAlert
  },
  computed: {
    store() {
      return this.$store.state;
    }
  }

};

</script>


<style lang="scss">
@import '@/sass/_base.sass'; 
@import '@/sass/style.sass';
</style>
