# Sound Ninja

![Sound Ninja Logo Animated](./designs/Logo_Animated.gif)
![License](https://img.shields.io/github/license/marcus-universe/SoundNinja?style=for-the-badge.svg)
![Code Size](https://img.shields.io/github/languages/code-size/marcus-universe/SoundNinja?style=for-the-badge.svg)
![Version](https://img.shields.io/github/package-json/v/marcus-universe/SoundNinja?style=for-the-badge.svg)
![Stars](https://img.shields.io/github/stars/marcus-universe/SoundNinja?style=for-the-badge.svg)
![Watchers](https://img.shields.io/github/watchers/marcus-universe/SoundNinja?style=plastic)

**Sound Ninja** is an **Open Source Soundboard App** with maximal customizability option to create your best Soundboard. Perfect for **live streaming, chatting with friends or podcast recording**. 😉👍

## How is the progress going?

```diff
-   Sound Ninja is still in development and is not ready for use!
```

The progress can be tracked on the [GitHub Project Board](https://github.com/users/marcus-universe/projects/1/views/1).

## Why Sound Ninja?

Sound Ninja is a **Open Source Soundboard App** with the goal to provide the maximal customizability options and a modern UI. Its build with **Tauri.js and Vue.js** and doesn't take much space on your pc and ram not like other soundboard apps. The main reason was to create a Soundboard-App that lets you organize really efficiently your sounds to find them really fast and easy in the moment you need them. On top, it gives everyone the option to share there soundboards with others to become together the biggest Meme-Loards of our time. 😎

## Features (+upcoming)

| Feature                      | Description                                               | Status |
| ---------------------------- | --------------------------------------------------------- | ------ |
| **Soundboard**               | Create your own Soundboard with your favorite sounds      | ✅     |
| **Tab System**               | Organize your sounds in Tabs                              | ⛔     |
| **Tag System**               | Tag your favorite sounds in to cathegories                | ⛔     |
| **Searchbar**                | Find your sounds quick and easy                           | ⛔     |
| **Midi Support**             | Control your Soundboard via Midi                          | ⛔     |
| **Themes**                   | Design your Soundboard how you like it                    | ⛔     |
| **Customizable Hotkeys**     | Customize the Hotkeys to your needs                       | ⛔     |
| **Soundboard Sharing**       | Share your Soundboard with your friends                   | ⛔     |
| **Soundboard Import/Export** | Import and Export your Soundboard                         | ⛔     |
| **Custom Profiles**          | Create multiple Soundboards with different settings       | ⛔     |
| **Websocket**                | Control your Soundboard with your Smartphone              | ⛔     |
| **Recorder**                 | Record your PC-Audio directly in Sound Ninja              | ⛔     |
| **Speech Search**            | Find sounds with the power of your voice                  | ⛔     |
| **Giphy**                    | Give your Sounds an Image/Gif                             | ⛔     |
| **Twitch Chat Control**      | Let your community decide what to play                    | ⛔     |
| **AI Assistant**             | Recommends you sounds that works in the current situation | ⛔     |

## Platforms

Sound Ninja currently supports theoretically the following platforms:

| Platform | Versions        |
| :------- | :-------------- |
| Windows  | 8 and above     |
| macOS    | 10.15 and above |
| Linux    | See below       |

**Linux Support**

- Debian (Ubuntu 18.04 and above or equivalent) with the following packages installed:
  - `libwebkit2gtk-4.0-37`, `libgtk-3-0`, `libayatana-appindicator3-1`<sup>1</sup>
- Arch with the following packages installed:
  - `webkit2gtk`, `gtk3`, `libayatana-appindicator`<sup>1</sup>
- Fedora (latest 2 versions) with the following packages installed:
  - `webkit2gtk3`, `gtk3`, `libappindicator-gtk3`<sup>1</sup>

```diff
-   Be aware that Sound Ninja is in a early development stage and so its not tested on all platforms yet.
```

## Project setup

```
npm install
```

### Available dependencies

- [Vue.js (Vue3 CLI)](https://vuejs.org/)
- [Tauri.js](https://tauri.studio/)
- [SASS](https://sass-lang.com/)
- [Vue Router](https://router.vuejs.org/)
- [Vuex](https://vuex.vuejs.org/)
- [Vue Use](https://vueuse.org/)
- [Howler.js](https://howlerjs.com/)

#### Available dependencies in future

- [Socket.io](https://socket.io/)

### Compiles and hot-reloads for development

```
npm run tauri:serve
```

### Compiles and minifies for production

```
npm run tauri:build
```

### Vue Devtools

[How to install Vue Devtools Standalone](https://devtools.vuejs.org/guide/installation.html#standalone)

```
npm run vue-devtools
```

### Lints and fixes files

```
npm run lint
```

### Customize configuration

See [Configuration Reference](https://cli.vuejs.org/config/).
