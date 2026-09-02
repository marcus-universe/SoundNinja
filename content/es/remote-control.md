---
title: Control remoto
description: Dispara SoundNinja desde Bitfocus Companion en la red local.
order: 7
---

# Control remoto

SoundNinja puede escuchar en tu red local para que Bitfocus Companion (o cualquier cliente HTTP/WebSocket) reproduzca y detenga sonidos.

## Activar el servidor remoto

1. Abre **Ajustes → Remoto**.
2. Activa el servidor remoto. El puerto por defecto es **7331**.
3. Opcional: define un token. Los clientes deben enviarlo como Bearer token o `?token=`.
4. Copia la URL `http://IP:PORT`, o copia la IP del sistema desde **Ajustes → Acerca de**.

Windows puede mostrar un aviso del firewall la primera vez que arranca el servidor. Permite el acceso a la red privada.

## Bitfocus Companion

1. Instala [Bitfocus Companion](https://bitfocus.io/companion).
2. Descarga [companion-module-soundninja.tgz](https://github.com/marcus-universe/SoundNinja/releases/latest/download/companion-module-soundninja.tgz). Aún no hay paquete en Companion Store.
3. En Companion: **Modules → Load module package** y elige el `.tgz`.
4. Añade una conexión **Sound Ninja**. Pega la IP del PC, el puerto y el token si definiste uno.

## Acciones

- **Trigger Sound** — elige en el menú desplegable en vivo, o escribe un ID de sonido de 8 caracteres (las variables de Companion funcionan)
- **Stop Sound** — detiene un sonido en reproducción por ID
- **Stop All** — detiene todos los sonidos en reproducción

Copia un ID de sonido en SoundNinja desde el menú contextual del botón o el chip de ID de la selección múltiple.

## Feedbacks y variables

- **Sound Playing** — verdadero mientras ese ID de sonido está activo (úsalo para el color del botón)
- `$(soundninja:connected)` — `true` / `false`
- `$(soundninja:playing_count)` — cuántos sonidos suenan
- `$(soundninja:last_triggered)` — último ID de sonido disparado

## API HTTP (avanzado)

Ruta base: `/api/v1`

- `GET /info` — nombre de la app, versión, protocolo, si se requiere un token
- `GET /sounds` — lista de sonidos (id, name, tabs, active)
- `GET /state` — sonidos más IDs que suenan ahora
- `POST /trigger` con `{ "id" }` — reproducir un sonido
- `GET /trigger/:id` — reproducir (fácil de probar en el navegador)
- `POST /stop` con `{ "id" }` opcional — detener un sonido o todos
- `GET /ws` — estado en vivo; enviar comandos trigger/stop de entrada
