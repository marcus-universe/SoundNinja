<template>
<div class="soundninja flex_c_h flex_space_between">
  <NavBar />
  <ErrorAlert />
  <router-view>


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
  methods: {
  }

};

</script>


<style lang="scss">
@import '@/sass/_base.sass'; 
@import '@/sass/style.sass';
</style>
