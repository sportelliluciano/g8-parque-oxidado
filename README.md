# Grupo 8 | Primer Proyecto Rust: Parque Oxidado
---

## Uso:

```bash
$ ./parque-oxidado [--personas=<LISTA>] [--costo-juegos=<LISTA>] [--capacidad-juegos=<LISTA>] [--duracion-juegos=<LISTA>] [--capacidad=<N>] [-h|--help] [--semilla=<N>] [-d|--debug]
```

### Configuración del simulador
El simulador permite configurar distintas variables de simulación, tales como la cantidad de personas que ingresarán al parque, la capacidad del parque, el costo, duración y capacidad de los juegos, más algunas opciones misceláneas.

Existen tres tipos de parámetros de línea de comandos posibles:
 - `--opcion=<N>`: Opción que acepta un número natural, por ejemplo, `--opcion=23`.
 - `--opcion=<LISTA>`: Opción que acepta una lista de números naturales.
 - `--opcion`: Opción que no acepta valores, su presencia activa una bandera.

Para el caso de las listas de valores hay tres formas de expresarlas:
 - Indicando explícitamente cada valor. Por ejemplo, `--opcion=1,2,3,4`.
 - Indicando una cantidad de elementos y un valor, `N:V`. Por ejemplo `--opcion=5:10` es equivalente a `--opcion=10,10,10,10,10`.
 - Indicando la cantidad de elementos y un rango para generar valores aleatorios, `N:min:max` para generar N elementos en el rango `[min, max)`.

#### Configurar las personas
Para definir la cantidad de personas que ingresarán al parque y sus presupuestos iniciales se debe utilizar el parámetro `--personas=<LISTA>`. Si el mismo no
se especifica se iniciará la simulación con la cantidad de personas y presupuesto
inicial por defecto: 5 personas con $ 40 cada una.

Ejemplos:

`--personas=5,18,10,25`: Ingresarán 4 personas con presupuestos iniciales de $ 5, $ 18, $ 10 y $ 25, respectivamente.

`--personas=5:40`: Ingresarán 5 personas, todas con presupuesto inicial de $ 40 (notar `:` en lugar de `,`).

`--personas=5:10:20`: Ingresarán 5 personas con presupuestos iniciales aleatorios uniformemente distribuidos entre $ 10 y $ 20.

#### Configurar los juegos
Los juegos del parque tienen tres variables configurables: el precio de la entrada, la cantidad de personas que pueden subirse como máximo a un juego en una vuelta del mismo; y la duración de la vuelta.

Estos parámetros se pueden configurar con las siguientes opciones
- `--costo-juegos=<LISTA>`: Precio de la entrada de cada juego. Si no se especifica se utilizará el costo por defecto para todos los juegos de $ 10.
- `--capacidad-juegos=<LISTA>`: Cantidad de personas que pueden subirse como máximo por vuelta. Si no se especifica se utilizará la capacidad por defecto de dos personas por vuelta para todos los juegos.
- `--duracion-juegos=<LISTA>`: Duración de la vuelta de cada juego, en milisegundos. Si no se especifica se utilizará el valor por defecto para todos los juegos de 25ms.

Cabe destacar que si se especifican dos o más parámetros de configuración de juegos, **todos** deberán tener la misma cantidad de elementos; de lo contrario se producirá
un error.

#### Configurar la capacidad del parque
Para definir la cantidad de personas que pueden estar simultáneamente dentro del parque en un momento específico se debe usar el parámetro `--capacidad`. Si no se
especifica el valor por defecto será de 10 personas. El valor de este parámetro deberá ser un número natural.

#### Modo debug
El simulador mostrará por defecto el estado de la simulación por la salida estándar. Opcionalmente se puede activar la opción `--debug` para guardar este registro a un archivo.

#### Semilla aleatoria
Para definir una semilla aleatoria específica se puede utilizar el parámetro `--semilla=<N>`.