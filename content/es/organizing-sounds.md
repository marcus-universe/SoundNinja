---
title: Organizar sonidos
description: Pestañas, etiquetas, filtros, orden, selección múltiple, volumen y atajos.
order: 4
---

# Organizar sonidos

SoundNinja está hecho para bibliotecas grandes. Pestañas, etiquetas, búsqueda, selección múltiple y atajos — el clip que necesitas está a un clic (o una tecla).

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

## Etiquetas

Las etiquetas cruzan las pestañas. Créalas en el panel **Filtro** (barra lateral **Filtro**):

- Añade una etiqueta con nombre y color
- Renombra, recolorea o elimina desde el mismo panel

Asigna etiquetas con clic derecho → menú **Etiquetas** (casillas). Si aún no hay etiquetas, el menú te lleva al panel de filtro.

## Filtro y búsqueda

Activa etiquetas en el panel de filtro o como **chips bajo la barra de búsqueda**. La coincidencia es **OR**: un sonido aparece si tiene cualquiera de las etiquetas seleccionadas.

El campo **Buscar** en la barra de navegación se actualiza al escribir y combina con **AND** el filtro de etiquetas activo.

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
- **KLIPY** — busca GIFs en línea. Pon tu propia clave API en **Ajustes → Comportamiento**. La clave no se guarda en los archivos del proyecto.

Tras elegir, mueve y recorta la imagen en el botón.
