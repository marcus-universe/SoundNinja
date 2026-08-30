---
title: Editor de grabación
description: Captura micrófono o audio del PC, edita el clip y suéltalo en tu tablero.
order: 6
---

# Editor de grabación

Graba un clip dentro de SoundNinja, recórtalo e impórtalo a una pestaña. Abre el editor con el botón **Grabar** en la barra del reproductor.

## Elegir una entrada

En **Ajustes → Audio**:

- **Dispositivos de entrada** — micrófono
- **Dispositivos de salida (audio del PC)** — loopback de lo que reproduce el PC
- **Volumen de entrada** — ganancia de captura del 0 % al 200 %

El botón Grabar usa el dispositivo que eliges aquí.

## Editar la toma

- Grabar y detener
- Observar el medidor de nivel de entrada
- **Normalizar** o **Cancelación de ruido**
- Recortar todo el clip, recortar a una selección o borrar una selección
- Deshacer / rehacer
- Acercar y alejar

## Preparar e importar

1. Añade la selección o la pista completa a la lista de staging.
2. La lista empieza vacía: graba, edita y luego añade un clip.
3. **Importar selección** o **Importar todo** en la pestaña actual o en una pestaña que elijas.

## Separación de stems (opcional)

El Record Editor puede conservar **voces** o **música** con BS-RoFormer. El modelo pesa unos 158 MB y **no** viene incluido con la app.

Descárgalo desde **Ajustes → Separación de stems**, o acepta el aviso del primer arranque. El instalador de Windows también puede ofrecer descargar el modelo al primer inicio.

Las builds desde código con stems necesitan un flag extra (`tauri:serve:stems` / `tauri:build:stems`).
