<template>
  <div class="flex-center LogoSection ">
    <div class="flex-center LogoIconContainer"><img src="../Assets/IMG/Logo_Icon.svg" alt=""></div>

    <h1>Sound<em>Ninja</em></h1>
  </div>

  <div class="windowoptions">

    <svg v-on:click="Minimize" class="HideApp" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 200">
      <g id="Fullscreen">
        <rect class="cls-1" x="9.07" y="139.37" width="181.85" height="30.96" rx="12" />
      </g>
    </svg>

    <svg v-on:click="Maximize" class="Maximize" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 200">
      <g id="Fullscreen">
        <path class="cls-1"
          d="M166.93,53.44v92.89H33.07V53.44H166.93m12-24H21.07a12,12,0,0,0-12,12V158.33a12,12,0,0,0,12,12H178.93a12,12,0,0,0,12-12V41.44a12,12,0,0,0-12-12Z" />
      </g>
    </svg>

    <svg v-on:click="ExitApp" class="ExitApp" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 200">
      <g id="Exit">
        <path class="cls-1"
          d="M122.39,100.06l53.3-53.31a7,7,0,0,0,0-9.85L163.16,24.37a7,7,0,0,0-9.86,0L100,77.67,46.7,24.37a7,7,0,0,0-9.86,0L24.31,36.9a7,7,0,0,0,0,9.85l53.3,53.31-53.3,53.3a7,7,0,0,0,0,9.86l12.53,12.53a7,7,0,0,0,9.86,0L100,122.44l53.3,53.31a7,7,0,0,0,9.86,0l12.53-12.53a7,7,0,0,0,0-9.86Z" />
      </g>
    </svg>

  </div>
</template>

<script>
import { ipcRenderer } from 'electron';
// const remote = require('electron').remote


export default {
  name: 'HelloWorld',
  props: {
    msg: String
  },
  //  data(){
  //           return {
  //               w: remote.getCurrentWindow(),
  //           }
  //       }, 
  methods: {
    Minimize() {
        ipcRenderer.send('hideapp');
      },
    ExitApp() {
        window.close()

      },
    Maximize() {
        ipcRenderer.send('maximizeapp');
      }
    }
  }




//?##########################
//? ANCHOR Maximize
//?##########################
// const maximizeButton = document.querySelector('.Maximize')

// maximizeButton.addEventListener('click', () => {
//     ipcRenderer.send('maximizeapp');
// })

//?##########################
//? ANCHOR Hide
//?##########################
// const hideButton = document.querySelector('.HideApp');

// hideButton.addEventListener('click', () => {
//     ipcRenderer.send('hideapp');
// })

//?##########################
//? ANCHOR Exit
//?##########################
// const ExitButtons = document.querySelectorAll('#Exit')

// ExitButtons.forEach(XButton => {
//     XButton.addEventListener('click', () => {
//         ipcRenderer.send('exitapp');
//         console.log("test");
//     });
// })

</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style>

</style>
