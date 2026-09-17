---
title: Organizar sonidos
description: Etiquetas, filtros, pestañas, grupos, orden, selección múltiple, volumen y atajos.
order: 4
---

# Organizar sonidos

SoundNinja está hecho para bibliotecas grandes. **Etiquetas y filtros**, pestañas, búsqueda, selección múltiple y atajos — el clip que necesitas está a un clic (o una tecla).

## Pestañas

- Añade una pestaña con **Añadir pestaña**. Dale un color para escanear la barra rápido.
- La vista **Todos** muestra cada sonido de todas las pestañas.
- Un sonido puede pertenecer a **una o más pestañas**. Clic derecho en un sonido y usa **Mover a pestaña** para asignarlo.

Las transiciones de pestaña están en **Ajustes → General**: Slide, Fade, Botones uno a uno, o Ninguna.

## Grupos

Desde la v0.5.3, los separadores pueden convertirse en grupos con nombre:

- Añadir, renombrar o quitar un grupo desde el menú contextual
- Arrastra la cabecera del grupo — los hijos se mueven con ella
- Suelta sonidos dentro o fuera de un grupo
- Define color de borde, color del nombre y alineación de botones (o usa el valor por defecto de la pestaña)

Sigue pudiendo añadir un **Separador** simple si solo necesitas un corte visual.

Clic derecho en el espacio vacío del tablero para **Añadir grupo**.

## Etiquetas / Filtros

Las etiquetas cruzan las pestañas. El panel **Filtro** (barra lateral **Filtro**) es donde las creas y filtras el tablero.

### Crear y editar etiquetas

1. Abre **Filtro** en la barra lateral.
2. En **Etiquetas**, usa **Añadir etiqueta** — nombre y color.
3. Renombra, recolorea o **Eliminar etiqueta** en la misma lista.
4. Si ves **Aún no hay etiquetas**, crea una aquí antes de asignar.

### Asignar etiquetas a sonidos

Clic derecho en un sonido → **Etiquetas** (casillas). Un sonido puede tener varias. Si aún no hay ninguna, el menú te lleva al panel de filtro.

### Filtrar el tablero

- Activa etiquetas en el panel Filtro o como **chips bajo la barra de búsqueda**.
- La coincidencia es **OR**: un sonido sigue visible si tiene **cualquiera** de las etiquetas seleccionadas.
- El campo **Buscar** combina con **AND** el filtro de etiquetas activo (texto + etiquetas juntos).
- Apaga los chips (o limpia el filtro) para ver de nuevo toda la pestaña.

Los nombres de etiqueta en color pueden aparecer en los botones. Actívalos en **Ajustes → Comportamiento → Mostrar badges de etiquetas**.

## Orden

Abre el panel de filtro y elige un modo:

- **Definido por el usuario** — tu orden de pestaña, grupos visibles
- **Nombre**, **Fecha de añadido**, **Tiempo de reproducción** o **Tamaño de archivo** — lista plana ordenada; **los grupos se ocultan**

Ordenar por duración puede precargar los metadatos de duración la primera vez.

## Reordenar

Activa arrastrar y soltar en **Ajustes → Comportamiento** (**Permitir reordenar**). Luego arrastra sonidos y grupos a su sitio. Reordenar aplica en el orden **Definido por el usuario**.

## Selección múltiple

Activa **Selección múltiple** en la barra de navegación, luego:

- Clic para alternar un sonido
- **Mayús+clic** para un rango desde el último ancla
- Arrastra un **marco** en el espacio vacío del tablero
- Clic en el tablero vacío (o fuera) para limpiar la selección

Una barra masiva muestra el recuento, más **Color**, **Mover a pestaña…**, **Eliminar** y **Hecho**.

En selección múltiple, cada botón muestra un **chip de ID de 8 caracteres**. Clic derecho en el chip para copiarlo.

## Volumen por sonido

Clic derecho → **Volumen**. Arrastra el control de 0 % a 100 %. **Doble clic** lo restablece a 100 %.

Se acumula con el volumen de salida maestro en Ajustes.

## Atajos e IDs de sonido

Cada sonido tiene un ID de 8 caracteres. Cópialo desde el menú contextual del botón (**Copiar ID**) o el chip de ID de la selección múltiple.

Abre **Ajustes → Atajos** para vincular una tecla a un ID de sonido. Los atajos globales son opcionales. **Asignar atajo** en el menú contextual salta allí con el ID rellenado.

El mismo ID es el que [Bitfocus Companion](/docs/remote-control) usa para disparar un clip.

## Imágenes de fondo y colores

Clic derecho en un sonido para definir colores por botón o una imagen de fondo. Eso es independiente del [Theme Creator](/docs/theme-editor), que da estilo a todo el tablero.

El selector es un diálogo **Imagen de fondo** (PNG, JPEG, GIF, WebP):

- **Archivos locales** — añade carpetas, elige un archivo, busca por nombre, previsualiza la cuadrícula o quita una carpeta de la biblioteca
- **GifSnap** — busca GIFs en línea sin clave API. La primera visita pide aceptar las peticiones a GifSnap; los resultados se cachean para reducir tráfico. Ver [GifSnap](https://gifsnap.com).
- **KLIPY** — busca GIFs en línea. Pon tu propia clave API en **Ajustes → Comportamiento**. La clave no se guarda en los archivos del proyecto.

Tras elegir, mueve y recorta la imagen en el botón.
