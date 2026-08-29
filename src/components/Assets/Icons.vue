<template>
<!-- Upload Button -->
<svg v-if="icon === 'upload'" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 200" :class="[customClass]" @click="$emit('triggered', icon)">
    <g id="Upload">
        <path
            id="Ramen"
            class="cls-1"
            d="M162.12,61.73c-3.75-3.8-10.27-10.89-17.17-18.4-7.27-7.9-14.78-16.07-19.42-20.77l-2.3-2.34H63.05A27.71,27.71,0,0,0,35.57,48.08v104A27.7,27.7,0,0,0,63.05,180H137a27.7,27.7,0,0,0,27.48-27.85v-88ZM133.45,54.2c3,3.28,5.94,6.47,8.63,9.37h-12c-6.39,0-10.35-1.93-10.35-11.55V39.41C123.8,43.71,128.67,49,133.45,54.2ZM137,164H63.05A11.84,11.84,0,0,1,51.3,152.11v-104A11.85,11.85,0,0,1,63.05,36.16h41V52c0,17.22,9.74,27.49,26.07,27.49h18.6v72.6A11.84,11.84,0,0,1,137,164Z" />
        <path
            id="Pfeil"
            class="cls-1"
            d="M81,107.54a6.87,6.87,0,0,0,4.9-2.06l7.13-7.23v27.89a6.94,6.94,0,1,0,13.88,0V97.84l7,7.11a6.87,6.87,0,0,0,9.81,0,7.1,7.1,0,0,0,0-10L105.31,76.29a6.91,6.91,0,0,0-5.11-2.05h-.28q-.3,0-.6,0l-.34,0c-.11,0-.22,0-.33.06s-.25.06-.37.09l-.29.07-.35.13-.29.11-.32.14-.3.15-.29.18a1.75,1.75,0,0,0-.29.18l-.31.22-.24.19a6.39,6.39,0,0,0-.48.44l0,0-19,19.24a7.1,7.1,0,0,0,0,10A6.88,6.88,0,0,0,81,107.54Z" />
        <path
            id="Line"
            class="cls-1"
            d="M131.76,136.2H69a7,7,0,0,0,0,14.06h62.72a7,7,0,0,0,0-14.06Z" />
    </g>
</svg>

<!-- Folders Button -->
<svg v-if="icon === 'folder'" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 200" :class="[customClass]" @click="$emit('triggered', icon)">
    <g id="Folder">
        <g id="darker">
            <path
                class="cls-2"
                d="M164.85,74.82c0,19-2.62,75.75-2.94,82.83-.59.28-1.2.53-1.81.75l-70.18.86H37.27c-.92-.11-1.81-.26-2.66-.42-11.27-2.2-15.26-8.89-15.26-26.07,0,0-.4-52.58-.4-68.1S17.46,36.33,40.43,36.33H73.26L88.47,51.24h51.31C155.89,51.24,164.85,54.53,164.85,74.82Z" />
        </g>
        <g id="path">
            <path
                class="cls-1"
                d="M181.67,89.07,171.83,145a17.86,17.86,0,0,1-9.92,12.64c-.59.28-1.2.53-1.81.75l-70.18.86H37.27c-.92-.11-1.81-.26-2.66-.42L46.89,89.07c1.37-7.85,8.91-14.25,16.74-14.25H169.94C177.77,74.82,183,81.22,181.67,89.07Z" />
        </g>
    </g>
</svg>

<!-- Stop Icon -->
<svg v-if="icon === 'stop'" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 200" :class="[customClass]" @click="$emit('triggered', icon)">
    <g id="Fullscreen">
        <rect
            class="cls-1"
            x="19.3"
            y="19.3"
            width="161.41"
            height="161.41"
            rx="23.39" />
    </g>
</svg>

<!-- Reset Icon -->
<svg v-if="icon === 'reset'" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 200" :class="[customClass]" @click="$emit('triggered', icon)">
    <g id="Reset">
        <path
            class="cls-1"
            d="M175.9,100.2a77.14,77.14,0,0,0-154.28.39c0,20.07,9.15,39.42,25.75,54.47,14.93,13.53,34.14,21.61,51.4,21.61v-27c-19.73,0-50.15-21.08-50.15-49.08a50.14,50.14,0,0,1,100.28-.19l-18.86.13,32.13,31.88,32.61-32.35Z" />
    </g>
</svg>

<!-- Settings Icon -->
<svg v-if="icon === 'settings'" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 200" :class="[customClass]" @click="$emit('triggered', icon)">
    <g id="Settings">
        <path
            class="cls-1"
            d="M169.43,82.73H157.55a59.63,59.63,0,0,0-6.14-13.8l8-8a10.56,10.56,0,0,0,0-14.9l-7.66-7.66a10.56,10.56,0,0,0-14.9,0L128.18,47a59.32,59.32,0,0,0-13.84-5.26V30.05a10.56,10.56,0,0,0-10.53-10.53H93A10.57,10.57,0,0,0,82.43,30.05V42.74a59.69,59.69,0,0,0-13.2,5.92h0l-8.59-8.58a10.56,10.56,0,0,0-14.9,0l-7.66,7.66a10.56,10.56,0,0,0,0,14.9l9.16,9.16a59,59,0,0,0-5.18,13.34H29.76A10.57,10.57,0,0,0,19.22,95.67v10.84a10.57,10.57,0,0,0,10.54,10.54H42.64a59.77,59.77,0,0,0,5.72,13.21h0l-8.58,8.58a10.58,10.58,0,0,0,0,14.9l7.66,7.66a10.56,10.56,0,0,0,14.9,0l8.85-8.85h0A61,61,0,0,0,84.84,158v11.69a10.57,10.57,0,0,0,10.54,10.54h10.83a10.57,10.57,0,0,0,10.54-10.54V157.66a58.7,58.7,0,0,0,13.8-6l8,8a10.56,10.56,0,0,0,14.9,0l7.67-7.66a10.58,10.58,0,0,0,0-14.9l-8.34-8.34h0a60.17,60.17,0,0,0,5.54-14.16h11.12A10.57,10.57,0,0,0,180,104.1V93.27A10.57,10.57,0,0,0,169.43,82.73Zm-69.29,50A32.66,32.66,0,1,1,132.8,100,32.66,32.66,0,0,1,100.14,132.7Z" />
    </g>
</svg>

<!-- About Page (mdi:about) -->
<svg v-if="icon === 'about'" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" :class="[customClass]" @click="$emit('triggered', icon)">
  <path class="cls-1" d="M13 9h-2V7h2m0 10h-2v-6h2m-1-9A10 10 0 0 0 2 12a10 10 0 0 0 10 10a10 10 0 0 0 10-10A10 10 0 0 0 12 2"/>
</svg>

<!-- Plus Icon -->
<svg v-if="icon === 'plus'" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 200" :class="[customClass]" @click="$emit('triggered', icon)">
  <g id="Plus">
    <path class="cls-1" d="M112.86,87.19V25.94a5.68,5.68,0,0,0-5.66-5.67H92.8a5.68,5.68,0,0,0-5.66,5.67V87.19H25.88a5.68,5.68,0,0,0-5.67,5.67v14.4a5.68,5.68,0,0,0,5.67,5.66H87.14v61.26a5.67,5.67,0,0,0,5.66,5.66h14.4a5.67,5.67,0,0,0,5.66-5.66V112.92h61.26a5.68,5.68,0,0,0,5.67-5.66V92.86a5.68,5.68,0,0,0-5.67-5.67Z"/>
  </g>
</svg>

<!-- Drag Icon (mdi:drag) -->
<svg v-if="icon === 'drag'" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" :class="[customClass]" @click="$emit('triggered', icon)">
  <path class="cls-1" d="M7 19v-2h2v2zm4 0v-2h2v2zm4 0v-2h2v2zm-8-4v-2h2v2zm4 0v-2h2v2zm4 0v-2h2v2zm-8-4V9h2v2zm4 0V9h2v2zm4 0V9h2v2zM7 7V5h2v2zm4 0V5h2v2zm4 0V5h2v2z"/>
</svg>

<!-- Exit -->
<svg v-if="icon === 'exit'" id="Ebene_1" data-name="Ebene 1" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 200" :class="[customClass]" @click="$emit('triggered', icon)">
  <g id="Exit">
    <path class="cls-1" d="M122.39,100.06l53.3-53.31a7,7,0,0,0,0-9.85L163.16,24.37a7,7,0,0,0-9.86,0L100,77.67,46.7,24.37a7,7,0,0,0-9.86,0L24.31,36.9a7,7,0,0,0,0,9.85l53.3,53.31-53.3,53.3a7,7,0,0,0,0,9.86l12.53,12.53a7,7,0,0,0,9.86,0L100,122.44l53.3,53.31a7,7,0,0,0,9.86,0l12.53-12.53a7,7,0,0,0,0-9.86Z"/>
  </g>
</svg>

<svg v-if="icon === 'check'" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 200" :class="[customClass]" @click="$emit('triggered', icon)">
  <g id="Check">
    <path id="Check-2" data-name="Check" class="cls-1" d="M81.22,165.56,23.13,107.47a6.9,6.9,0,0,1,0-9.77l8.45-8.45a6.9,6.9,0,0,1,9.77,0l38.27,38.28A408.76,408.76,0,0,1,103,83.93C127.22,45,151.38,25.58,174.89,26.08a6.93,6.93,0,0,1,6.68,8.42l-2.69,12a6.89,6.89,0,0,1-6,5.44C159.68,53,142.58,69.13,125,97.3c-17.26,27.62-29,57.14-29.11,57.44l-3.34,8.47A6.91,6.91,0,0,1,81.22,165.56Z"/>
  </g>
</svg>

<svg v-if="icon === 'search'" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 200" :class="[customClass]" @click="$emit('triggered', icon)">
  <g id="Fullscreen">
    <path class="cls-1" d="M164.54,35.39a58.15,58.15,0,0,0-92.4,68.51L23.72,152.33a13.05,13.05,0,0,0,0,18.42l5.46,5.46a13.07,13.07,0,0,0,18.42,0L96,127.78a58.14,58.14,0,0,0,68.52-92.39Zm-59,74.08a37.71,37.71,0,0,1-8.6-6.43,37.14,37.14,0,0,1-6.43-8.6,37.52,37.52,0,1,1,15,15Z"/>
  </g>
</svg>

<!-- Zoom in (mdi:magnify-plus-outline) -->
<svg v-if="icon === 'zoom-in'" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" :class="[customClass]" @click="$emit('triggered', icon)">
  <path class="cls-1" d="M15.5 14h-.79l-.28-.27A6.47 6.47 0 0 0 16 9.5A6.5 6.5 0 1 0 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5S14 7.01 14 9.5S11.99 14 9.5 14m2.5-4h-2v2H9v-2H7V9h2V7h1v2h2z"/>
</svg>

<!-- Zoom out (mdi:magnify-minus-outline) -->
<svg v-if="icon === 'zoom-out'" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" :class="[customClass]" @click="$emit('triggered', icon)">
  <path class="cls-1" d="M15.5 14h-.79l-.28-.27A6.47 6.47 0 0 0 16 9.5A6.5 6.5 0 1 0 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5S14 7.01 14 9.5S11.99 14 9.5 14M7 9h5v1H7z"/>
</svg>

<svg v-if="icon === 'palette'" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 200" :class="[customClass]" @click="$emit('triggered', icon)">
  <g id="Palette">
    <path class="cls-1" d="M100,20A80,80,0,0,0,20,100c0,44.18,35.82,76,80,76a20,20,0,0,0,20-20,19.7,19.7,0,0,0-4.6-12.72,4,4,0,0,1-.9-2.48,4,4,0,0,1,4-4H136a44,44,0,0,0,44-44C180,50.14,144.18,20,100,20ZM54,116a14,14,0,1,1,14-14A14,14,0,0,1,54,116ZM74,68A14,14,0,1,1,88,54,14,14,0,0,1,74,68Zm52,0a14,14,0,1,1,14-14A14,14,0,0,1,126,68Zm32,48a14,14,0,1,1,14-14A14,14,0,0,1,158,116Z"/>
  </g>
</svg>

<!-- Multi-select active (mingcute:multiselect-fill) -->
<svg v-if="icon === 'multiselect-active'" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" :class="[customClass]" @click="$emit('triggered', icon)">
  <path class="cls-1" d="M20 2a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2h-3v2a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V9a2 2 0 0 1 2-2h2V4a2 2 0 0 1 2-2zM6 9H4v11h11v-2H8a2 2 0 0 1-2-2zm11.89-1.768a1 1 0 0 0-1.32-.083l-.095.083l-3.536 3.536l-1.414-1.414a1 1 0 0 0-1.497 1.32l.083.094l2.05 2.05a1.1 1.1 0 0 0 1.46.086l.096-.085l4.172-4.172a1 1 0 0 0 0-1.415Z"/>
</svg>

<!-- Multi-select inactive (mingcute:multiselect-line) -->
<svg v-if="icon === 'multiselect-inactive'" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" :class="[customClass]" @click="$emit('triggered', icon)">
  <path class="cls-1" d="M20 2a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2h-3v2a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V9a2 2 0 0 1 2-2h2V4a2 2 0 0 1 2-2zM6 9H4v11h11v-2H8a2 2 0 0 1-2-2zm14-5H8v12h12zm-2.11 3.233a1 1 0 0 1 0 1.414l-4.173 4.172a1.1 1.1 0 0 1-1.556 0l-2.05-2.05a1 1 0 0 1 1.414-1.415l1.414 1.414l3.536-3.535a1 1 0 0 1 1.414 0Z"/>
</svg>

<!-- Add / Import (icon-park-solid:add) -->
<svg v-if="icon === 'add'" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 48 48" :class="[customClass]" @click="$emit('triggered', icon)">
  <defs>
    <mask id="sn-icon-park-add">
      <g fill="none" stroke-linejoin="round" stroke-width="4">
        <rect width="36" height="36" x="6" y="6" fill="#fff" stroke="#fff" rx="3"/>
        <path stroke="#000" stroke-linecap="round" d="M24 16v16m-8-8h16"/>
      </g>
    </mask>
  </defs>
  <path class="cls-1" d="M0 0h48v48H0z" mask="url(#sn-icon-park-add)"/>
</svg>

<!-- Project (material-symbols:dashboard-rounded) -->
<svg v-if="icon === 'project'" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" :class="[customClass]" @click="$emit('triggered', icon)">
  <path class="cls-1" d="M14 9q-.425 0-.712-.288T13 8V4q0-.425.288-.712T14 3h6q.425 0 .713.288T21 4v4q0 .425-.288.713T20 9zM4 13q-.425 0-.712-.288T3 12V4q0-.425.288-.712T4 3h6q.425 0 .713.288T11 4v8q0 .425-.288.713T10 13zm10 8q-.425 0-.712-.288T13 20v-8q0-.425.288-.712T14 11h6q.425 0 .713.288T21 12v8q0 .425-.288.713T20 21zM4 21q-.425 0-.712-.288T3 20v-4q0-.425.288-.712T4 15h6q.425 0 .713.288T11 16v4q0 .425-.288.713T10 21z"/>
</svg>

<!-- Question / Info (pajamas:question) -->
<svg v-if="icon === 'question'" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" :class="[customClass]" @click="$emit('triggered', icon)">
  <path class="cls-1" fill-rule="evenodd" d="M8 16A8 8 0 1 0 8 0a8 8 0 0 0 0 16M4.927 4.99Q4.5 5.634 4.5 6.26q0 .305.27.566t.661.26q.665 0 .903-.746q.252-.713.616-1.08q.364-.366 1.134-.366q.658 0 1.075.363q.416.364.416.892a.97.97 0 0 1-.136.502a2 2 0 0 1-.336.419a14 14 0 0 1-.648.558q-.51.423-.812.73q-.3.308-.483.713c-.322 1.245 1.35 1.345 1.736.456q.07-.128.213-.284q.144-.155.382-.36a41 41 0 0 0 1.194-1.034q.332-.306.573-.73a1.95 1.95 0 0 0 .242-.984q0-.712-.424-1.32q-.423-.609-1.2-.962T8.084 3.5q-1.092 0-1.911.423T4.927 4.989Zm2.14 7.08a1 1 0 1 0 2 0a1 1 0 0 0-2 0" clip-rule="evenodd"/>
</svg>

<!-- Delete / Trash (material-symbols:delete) -->
<svg v-if="icon === 'delete'" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" :class="[customClass]" @click="$emit('triggered', icon)">
  <path class="cls-1" d="M7 21q-.825 0-1.412-.587T5 19V6H4V4h5V3h6v1h5v2h-1v13q0 .825-.587 1.413T17 21zm2-4h2V8H9zm4 0h2V8h-2z"/>
</svg>

<!-- Rename (mdi:rename) -->
<svg v-if="icon === 'rename'" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" :class="[customClass]" @click="$emit('triggered', icon)">
  <path class="cls-1" d="m15 16l-4 4h10v-4zm-2.94-8.81L3 16.25V20h3.75l9.06-9.06zm6.65.85c.39-.39.39-1.04 0-1.41l-2.34-2.34a1 1 0 0 0-1.41 0l-1.83 1.83l3.75 3.75z"/>
</svg>

<!-- Tab / Move to Tab (material-symbols:tab) -->
<svg v-if="icon === 'tab'" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" :class="[customClass]" @click="$emit('triggered', icon)">
  <path class="cls-1" d="M4 20q-.825 0-1.412-.587T2 18V6q0-.825.588-1.412T4 4h16q.825 0 1.413.588T22 6v12q0 .825-.587 1.413T20 20zm9-10h7V6h-7z"/>
</svg>

<!-- Page Separator (ri:page-separator) -->
<svg v-if="icon === 'page-separator'" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" :class="[customClass]" @click="$emit('triggered', icon)">
  <path class="cls-1" d="M17 21v-4H7v4H5v-5a1 1 0 0 1 1-1h12a1 1 0 0 1 1 1v5zM7 3v4h10V3h2v5a1 1 0 0 1-1 1H6a1 1 0 0 1-1-1V3zM2 9l4 3l-4 3zm20 0v6l-4-3z"/>
</svg>

<!-- Play (line-md:play-filled) -->
<svg v-if="icon === 'play'" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" :class="[customClass]" @click="$emit('triggered', icon)">
  <path class="cls-1" fill-opacity="0" stroke="currentColor" stroke-dasharray="38" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 6l10 6l-10 6Z">
    <animate fill="freeze" attributeName="stroke-dashoffset" dur="0.5s" values="38;0"/>
    <animate fill="freeze" attributeName="fill-opacity" begin="0.5s" dur="0.4s" to="1"/>
  </path>
</svg>

<!-- Pause (material-symbols-light:pause-rounded) -->
<svg v-if="icon === 'pause'" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" :class="[customClass]" @click="$emit('triggered', icon)">
  <path class="cls-1" d="M15 18q-.402 0-.701-.299T14 17V7q0-.402.299-.701T15 6h1.5q.402 0 .701.299T17.5 7v10q0 .402-.299.701T16.5 18zm-7.5 0q-.402 0-.701-.299T6.5 17V7q0-.402.299-.701T7.5 6H9q.402 0 .701.299T10 7v10q0 .402-.299.701T9 18z"/>
</svg>

<!-- Stems / Timeline (material-symbols:view-timeline-rounded) -->
<svg v-if="icon === 'stems'" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" :class="[customClass]" @click="$emit('triggered', icon)">
  <path class="cls-1" d="M5 21q-.825 0-1.412-.587T3 19V5q0-.825.588-1.412T5 3h14q.825 0 1.413.588T21 5v14q0 .825-.587 1.413T19 21zM7 17h4q.425 0 .713-.288T12 16t-.288-.712T11 15H7q-.425 0-.712.288T6 16t.288.713T7 17m3-4h4q.425 0 .713-.288T15 12t-.288-.712T14 11h-4q-.425 0-.712.288T9 12t.288.713T10 13m3-4h4q.425 0 .713-.288T18 8t-.288-.712T17 7h-4q-.425 0-.712.288T12 8t.288.713T13 9"/>
</svg>

<!-- Record (proicons:record) -->
<svg v-if="icon === 'record'" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" :class="[customClass]" @click="$emit('triggered', icon)">
  <g fill="none">
    <circle class="cls-1" cx="12" cy="12" r="9.25" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" fill="none"/>
    <circle class="cls-1" cx="12" cy="12" r="5"/>
  </g>
</svg>

<!-- Loop / repeat -->
<svg v-if="icon === 'loop'" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" :class="[customClass]" @click="$emit('triggered', icon)">
  <path class="cls-1" d="M7 7h10v3l4-4l-4-4v3H5v6h2zm10 10H7v-3l-4 4l4 4v-3h12v-6h-2z"/>
</svg>

<!-- Noise Cancelation (boxicons:noise-filled) -->
<svg v-if="icon === 'noise-cancel'" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" :class="[customClass]" @click="$emit('triggered', icon)">
  <path class="cls-1" d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2M6 17H5v-1h1zm0-4H5v-1h1zm0-4H5V8h1zm2 10H7v-1h1zm0-4H7v-1h1zm0-4H7v-1h1zm0-4H7V6h1zm2 10H9v-1h1zm0-4H9v-1h1zm0-4H9V8h1zm2 10h-1v-1h1zm0-4h-1v-1h1zm0-4h-1v-1h1zm0-4h-1V6h1zm2 10h-1v-1h1zm0-4h-1v-1h1zm0-4h-1V8h1zm2 10h-1v-1h1zm0-4h-1v-1h1zm0-4h-1v-1h1zm0-4h-1V6h1zm2 10h-1v-1h1zm0-4h-1v-1h1zm0-4h-1V8h1z"/>
</svg>

<!-- Playlist (ph:playlist-fill) -->
<svg v-if="icon === 'playlist'" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" :class="[customClass]" @click="$emit('triggered', icon)">
  <path class="cls-1" d="M208 32H48a16 16 0 0 0-16 16v160a16 16 0 0 0 16 16h160a16 16 0 0 0 16-16V48a16 16 0 0 0-16-16M64 72h128a8 8 0 0 1 0 16H64a8 8 0 0 1 0-16m0 48h72a8 8 0 0 1 0 16H64a8 8 0 0 1 0-16m40 64H64a8 8 0 0 1 0-16h40a8 8 0 0 1 0 16m103.59-53.47a8 8 0 0 1-10.12 5.06L184 131.1V176a24 24 0 1 1-16-22.62V120a8 8 0 0 1 10.53-7.59l24 8a8 8 0 0 1 5.06 10.12"/>
</svg>

<!-- Normalize / Waveform (ph:waveform-fill) -->
<svg v-if="icon === 'normalize'" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" :class="[customClass]" @click="$emit('triggered', icon)">
  <path class="cls-1" d="M216 40H40a16 16 0 0 0-16 16v144a16 16 0 0 0 16 16h176a16 16 0 0 0 16-16V56a16 16 0 0 0-16-16M72 152a8 8 0 0 1-16 0v-48a8 8 0 0 1 16 0Zm32 32a8 8 0 0 1-16 0V72a8 8 0 0 1 16 0Zm32-16a8 8 0 0 1-16 0V88a8 8 0 0 1 16 0Zm32-16a8 8 0 0 1-16 0v-48a8 8 0 0 1 16 0Zm32 8a8 8 0 0 1-16 0V96a8 8 0 0 1 16 0Z"/>
</svg>

<!-- Folders (ph:folders-fill) -->
<svg v-if="icon === 'folders'" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" :class="[customClass]" @click="$emit('triggered', icon)">
  <path class="cls-1" d="M224 64h-69.33l-27.74-20.8a16.12 16.12 0 0 0-9.6-3.2H72a16 16 0 0 0-16 16v16H40a16 16 0 0 0-16 16v112a16 16 0 0 0 16 16h152.89A15.13 15.13 0 0 0 208 200.89V184h16.89A15.13 15.13 0 0 0 240 168.89V80a16 16 0 0 0-16-16m0 104h-16v-56a16 16 0 0 0-16-16h-69.33L94.93 75.2a16.12 16.12 0 0 0-9.6-3.2H72V56h45.33l29.87 22.4A8 8 0 0 0 152 80h72Z"/>
</svg>

<!-- Audio File (material-symbols:audio-file-rounded) -->
<svg v-if="icon === 'audio-file'" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" :class="[customClass]" @click="$emit('triggered', icon)">
  <path class="cls-1" d="M6 22q-.825 0-1.412-.587T4 20V4q0-.825.588-1.412T6 2h7.175q.4 0 .763.15t.637.425l4.85 4.85q.275.275.425.638t.15.762V20q0 .825-.587 1.413T18 22zm7-14q0 .425.288.713T14 9h4l-5-5zm-2.25 11q.95 0 1.6-.65t.65-1.6V13h2q.425 0 .713-.288T16 12t-.288-.712T15 11h-2q-.425 0-.712.288T12 12v2.875q-.275-.2-.587-.288t-.663-.087q-.95 0-1.6.65t-.65 1.6t.65 1.6t1.6.65"/>
</svg>

</template>

<script>
export default {
    emits: ['triggered'],
    props: {
        icon: {
            type: String,
            required: false,
            default: 'upload'
        },
        customClass: {
            required: false,
            default: ''
        }
    },

}
</script>
