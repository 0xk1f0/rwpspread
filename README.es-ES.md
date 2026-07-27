<div align="center" style="text-decoration: none;">
  <h1>rwpspread</h1>
  <a href="https://github.com/0xk1f0/rwpspread/releases/latest">
    <img src="https://img.shields.io/github/v/release/0xk1f0/rwpspread?style=for-the-badge&color=blue" />
  </a>
  <a href="https://crates.io/crates/rwpspread">
    <img src="https://img.shields.io/crates/v/rwpspread?style=for-the-badge&color=orange" />
  </a>
  <a href="https://aur.archlinux.org/packages/rwpspread">
    <img src="https://img.shields.io/aur/version/rwpspread?style=for-the-badge&color=1793D1" />
  </a>
  <br><br>
  <a href="https://wallhaven.cc/w/l8q5k2">
    <img width=70% height=75% src="https://github.com/user-attachments/assets/836131fe-0f8e-449e-993d-9df2ebd33865"></img>
  </a>
</div>

## Características

- Fondo de pantalla extendido a través de todos los monitores
- Detección de conexión/desconexión de monitores (hotplugging)
- Generación de paleta de colores
- Compensación de ppi y marcos (bezel) del monitor
- Soporte para varios asignadores de fondo de pantalla
  - [`wpaperd`](https://github.com/danyspin97/wpaperd)
  - [`swaybg`](https://github.com/swaywm/swaybg)
  - [`hyprpaper`](https://github.com/hyprwm/hyprpaper)
- Generación de configuración para bloqueadores de pantalla
  - [`swaylock`](https://github.com/swaywm/swaylock)
  - [`hyprlock`](https://github.com/hyprwm/hyprlock)

## Instalación

[![Arch Linux](https://img.shields.io/badge/Arch_Linux-via_AUR-grey?style=for-the-badge&logo=arch-linux&color=1793D1)](https://aur.archlinux.org/packages/rwpspread)

```bash
# estable
paru -S rwpspread
# git
paru -S rwpspread-git
```

[![Nix](https://img.shields.io/badge/nix-via%20nixpkgs-grey?style=for-the-badge&logo=Nixos&color=5277C3)](https://search.nixos.org/packages?query=rwpspread)

```bash
# probarlo
nix run nixpkgs#rwpspread
# o añadir a cualquier lista de paquetes de usuario/sistema como
pkgs.rwpspread
# master
tome `rwpspread` directamente del flake de este repositorio.
```

[![Crates.io](https://img.shields.io/badge/crates.io-via_cargo-grey?style=for-the-badge&logo=rust&color=FFC933)](https://crates.io/crates/rwpspread)

```bash
# globalmente
cargo install rwpspread
```

## Compilación

```bash
git clone https://github.com/0xk1f0/rwpspread.git
cd rwpspread/
cargo build --release
```

## Uso

```text
rwpspread 0.5.1 - Multi-Monitor Wallpaper Spanning Utility

Usage:
  rwpspread [OPTIONS] <--image <IMAGE>|--info>

Options:
  -i, --image <IMAGE>           Ruta al archivo de imagen o directorio
      --info                    Mostrar información detectable
  -o, --output <OUTPUT>         Ruta del directorio de salida
  -a, --align <ALIGN>           No reducir la imagen base, alinear el diseño en su lugar [valores posibles: tl, tr, tc, bl, br, bc, rc, lc, ct]
  -b, --backend <BACKEND>       Backend del asignador de fondo de pantalla [valores posibles: wpaperd, swaybg, hyprpaper]
  -l, --locker <LOCKER>         Implementación de pantalla de bloqueo para generar [valores posibles: swaylock, hyprlock]
      --bezel <BEZEL>           Cantidad de marco (bezel) en píxeles para compensar
  -m, --monitors <MONITORS>...  Lista de monitores con su diagonal en pulgadas [formato: "<NAME>:<INCHES>"]
      --ppi                     Compensar diferentes valores de ppi de los monitores
  -d, --daemon                  Habilitar modo demonio y redistribuir al cambiar las salidas
  -p, --palette                 Generar una paleta de colores a partir de la imagen de entrada
      --pre <PRE>               Script a ejecutar antes de la división
      --post <POST>             Script a ejecutar después de la división
  -w, --watch                   Vigilar cambios en la fuente del fondo y redistribuir al detectarlos
  -f, --force-resplit           Forzar redistribución, omite todas las comprobaciones de caché de imagen
  -h, --help                    Mostrar ayuda
  -V, --version                 Mostrar versión
```

## Ejemplos

```bash
# Proporcionar una imagen de entrada
# Las pantallas se leen automáticamente
rwpspread -i /alguna/ruta/fondo.png

# También puedes especificar un directorio
# rwpspread elegirá la imagen al azar
# formatos soportados: jpg, jpeg, png
rwpspread -i /alguna/ruta/fondos/

# Si deseas redistribuciones automáticas
# al conectar/desconectar monitores
# inicia en modo demonio
rwpspread -di /alguna/ruta/fondo.png

# Usar, por ejemplo, la integración con wpaperd
# esto autogenera el archivo de configuración
# y reinicia wpaperd automáticamente
# necesitarás tener wpaperd instalado
rwpspread -b wpaperd -i /alguna/ruta/fondo.png
```

> [!NOTE]  
> `rwpspread` intentará forzar el cierre de cualquier instancia de backend que ya esté ejecutándose; esto puede fallar en algunos casos y evitar que se asignen los fondos de pantalla. Ver Issue https://github.com/0xk1f0/rwpspread/issues/100
> 
> Asegúrate de que `rwpspread` sea el primero en iniciar cualquier proceso de `swaybg`, `hyprpaper` o `wpaperd`, aunque los dos últimos podrían no verse afectados.

## Integración con `swaylock`

Se colocará una cadena de opciones para swaylock en `/home/$USER/.cache/rwpspread/rwps_swaylock.conf` que puede verse así:

```text
-i <ruta_imagen_1> -i <ruta_imagen_2>
```

Este archivo puede ser cargado y usado con tu comando de swaylock, por ejemplo:

```bash
#!/usr/bin/env bash

# cargar las opciones del comando
IMAGES=$(cat /home/$USER/.cache/rwpspread/rwps_swaylock.conf)
# ejecutar con ellas
swaylock $IMAGES --scaling fill
```

## Integración con `hyprlock`

Simplemente incluye `/home/$USER/.cache/rwpspread/rwps_hyprlock.conf` en tu `hyprlock.conf` habitual de esta manera:

```text
# include generado por rwpspread
source=/home/$USER/.cache/rwpspread/rwps_hyprlock.conf
```

Esto te permite configurar elementos adicionales de `hyprlock` después de la sentencia de importación.

## Compensación de ppi del monitor

Muchos usuarios utilizan configuraciones con diferentes resoluciones y tamaños de monitor. Por ejemplo, podrían tener un monitor principal de alta resolución capaz de 3840x2160 4K, mientras que su monitor secundario es solo de resolución Full-HD 1920x1080.
Un monitor 4K de 27' tiene una densidad de píxeles más alta que un monitor Full-HD de 27', lo que representa un problema para los divisores de fondos de pantalla, ya que la imagen en el monitor Full-HD se verá extrañamente estirada junto a la pantalla de mayor resolución. Aquí es donde entra en juego la compensación de ppi.

```bash
rwpspread --ppi --monitors "DP-1:32 DP-2:27" -i /alguna/ruta/fondo.png
```

El comando anterior le indica a `rwpspread` que tienes un monitor de 32' en DisplayPort 1 y un monitor de 27' en DisplayPort 2. Al combinar esta información adicional con la resolución de las pantallas, se puede calcular un valor de píxeles por pulgada. Basándose en este valor, `rwpspread` escalará las imágenes divididas para los monitores de menor resolución y las realineará en consecuencia para compensar su menor densidad de píxeles.
El resultado final debería ser una transición más alineada entre pantallas de diferente resolución.

## Compensación de marcos (bezel) del monitor

Mientras que la compensación de ppi se encarga del trabajo pesado con pantallas de diferente resolución, la compensación de marcos puede ayudarte en escenarios donde haya más distancia entre tus monitores de la que desearías. En ese caso, las divisiones podrían no tener transiciones fluidas de monitor a monitor, ya que las pantallas físicas no están directamente pegadas.

```bash
rwpspread --bezel 40 -i /alguna/ruta/fondo.png
```

La compensación de marcos aplica un desplazamiento fijo en píxeles entre los bordes que se tocan de los monitores, para que la transición y las divisiones se vean más fluidas. También puedes usarlo en combinación con la compensación de ppi para obtener una configuración perfecta.

## Scripts Personalizados

Puedes especificar scripts o programas personalizados para ejecutar antes y después de que se realice la división.

```bash
# antes de dividir
rwpspread --pre /alguna/ruta/script_pre.sh -di /alguna/ruta/fondo.png
# después de dividir
rwpspread --post /alguna/ruta/script_post.sh -di /alguna/ruta/fondo.png
# o ambos
rwpspread --pre /alguna/ruta/script_pre.sh --post /alguna/ruta/script_post.sh -di /alguna/ruta/fondo.png
```

Cuando está en modo `daemon`, estos scripts también se ejecutarán en las redistribuciones, por ejemplo, al conectar monitores.

> [!NOTE]  
> `rwpspread` esperará a que estos scripts terminen de ejecutarse antes de continuar con su propia ejecución.
> 
> Por lo tanto, asegúrate de no proporcionar scripts que bloqueen la ejecución indefinidamente.

## Ubicaciones de Guardado

Si se utiliza solo para dividir imágenes, las imágenes de salida se guardan en el directorio de trabajo actual.

```bash
# archivos de salida en $PWD
rwpspread -i /alguna/ruta/fondo.png
```

Cuando se utiliza con la opción de backend o demonio, las imágenes de salida se almacenan en `$XDG_CACHE_HOME/rwpspread/` o alternativamente en `$HOME/.cache/rwpspread/` con el prefijo `rwps_`.

```bash
# archivos de salida en $XDG_CACHE_HOME/rwpspread/ o $HOME/.cache/rwpspread/
rwpspread -b swaybg -i /alguna/ruta/fondo.png
```

Para obtener todos los archivos, simplemente haz:

```bash
ls /home/$USER/.cache/rwpspread/
```
> [!NOTE]
> Si utilizas el backend `wpaperd`, `rwpspread` usará su ruta de configuración predeterminada `/home/$USER/.config/wpaperd/` para la configuración autogenerada.

Si deseas personalizar la carpeta de salida, utiliza la opción `-o`:

```bash
# archivos de salida en /alguna/otra/ruta/
rwpspread -o /alguna/otra/ruta/ -i /alguna/ruta/fondo.png
```

> [!NOTE]
> ¡Ten en cuenta que `rwpspread` tomará el control total de esta carpeta y potencialmente eliminará archivos que no quieras que se eliminen!

### Nombres de archivo legibles

En general, los archivos divididos que `rwpspread` almacena no son constantes; cambian según la configuración que recibe. Esto incluye el tipo de opciones con las que se ejecutó y cuántos monitores están conectados actualmente. Los archivos tienen un formato específico.

```bash
# archivo de salida real
rwps_<nombre-monitor>_<hash-config>.png
```

Esto puede hacer que estos archivos sean un poco engorrosos de usar en herramientas externas o asignadores de fondos de pantalla. Por eso, `rwpspread` también crea enlaces simbólicos adicionales con un nombre predecible que apuntan al archivo de salida. Es importante notar que esto solo lo hará si se especifica `-b` o `-d`.

```bash
# enlace simbólico al archivo real
rwps_<nombre-monitor>.png
```

Puedes usar esto en cualquier otra herramienta que utilice los archivos de salida de `rwpspread` sin preocuparte por los cambios de nombre.

## Solución de Problemas

Si encuentras problemas después de una actualización o con una nueva versión, por favor haz lo siguiente:

```bash
# limpiar imágenes en caché
rm -r /home/$USER/.cache/rwpspread/
# limpiar config de wpaperd (si lo usas)
rm /home/$USER/.config/wpaperd/wallpaper.toml
```

E inténtalo de nuevo.

Si esto no soluciona tu problema, no dudes en abrir un PR y le echaré un vistazo cuando tenga tiempo.

## Créditos y Agradecimientos

- [fsnkty](https://github.com/fsnkty) - Mantenedor del paquete de Nix
- [smithay-client-toolkit](https://github.com/Smithay/client-toolkit) - Interacción de Rust con Wayland
- [wpaperd](https://github.com/danyspin97/wpaperd) - Excelente demonio de fondos de pantalla
- [swaylock](https://github.com/swaywm/swaylock) - Utilidad de bloqueo de pantalla
- [swaybg](https://github.com/swaywm/swaybg) - Utilidad de fondo de pantalla
- [hyprpaper](https://github.com/hyprwm/hyprpaper) - Demonio de fondos de pantalla de Hypr
- [hyprlock](https://github.com/hyprwm/hyprlock) - Bloqueador de pantalla de Hypr
- [material-colors](https://github.com/Aiving/material-colors) - Generación de colores Material
