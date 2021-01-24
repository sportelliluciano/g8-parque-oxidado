# Grupo 8 | Primer Proyecto Rust: Parque Oxidado
---

## Uso:

```bash
$ ./parque-oxidado [--personas=<PERSONAS>] [--juegos=<JUEGOS>] [--capacidad=<N>] [-h|--help] [-d|--debug]
```

### Configurar la cantidad de personas
Para definir la cantidad de personas que ingresarán al parque y sus presupuestos iniciales se debe utilizar el parámetro `--personas=<PERSONAS>`. Si el mismo no
se especifica se iniciará la simulación con la cantidad de personas y presupuesto
inicial por defecto: 5 personas con $ 40 cada una.

Existen tres formas de especificar las personas que ingresarán al parque:
 - Indicando explícitamente los presupuestos iniciales de cada persona, `n1,n2,n3,...`.
 - Indicando la cantidad de personas que ingresarán y un presupuesto inicial
   común a todas las personas, `N:P`.
 - Indicando la cantidad de personas que ingresarán y un rango para generar
   presupuestos aleatorios, `N:Pmin:Pmax`.

Ejemplos:

`--personas=5,18,10,25`: Ingresarán 4 personas con presupuestos iniciales de $ 5, $ 18, $ 10 y $ 25, respectivamente.

`--personas=5:40`: Ingresarán 5 personas, todas con presupuesto inicial de $ 40 (notar `:` en lugar de `,`).

`--personas=5:10:20`: Ingresarán 5 personas con presupuestos iniciales aleatorios uniformemente distribuidos entre $ 10 y $ 20.

### Configurar la cantidad de juegos
Para definir la cantidad de juegos y el costo de cada juego se utiliza el parámetro
`--juegos=<JUEGOS>`. Si el mismo no se especifica se utilizará el valor por defecto
de 5 juegos de $ 10 cada uno.

Al igual que con las personas, hay 3 formas de definir los juegos:
 - Indicando explícitamente el costo de cada juego, `j1,j2,j3`.
 - Indicando la cantidad de juegos y un costo común a todos los juegos, `N:P`.
 - Indicando la cantidad de juegos y un rango para generar costos aleatorios, `N:Pmin:Pmax`.

Ejemplos:

`--juegos=5,18,10,25`: Habrá 4 juegos con costos de $ 5, $ 18, $ 10 y $ 25, respectivamente.

`--juegos=5:40`: Habrá 5 juegos, todos con costo de $ 40 (notar `:` en lugar de `,`).

`--juegos=5:10:20`: Habrá 5 juegos con costos aleatorios uniformemente distribuidos entre $ 10 y $ 20.

### Configurar la capacidad del parque
Para definir la cantidad de personas que pueden estar simultáneamente dentro del parque en un momento específico se debe usar el parámetro `--capacidad`. Si no se
especifica el valor por defecto será de 10 personas. El valor de este parámetro deberá ser un número natural.

### Modo debug
El simulador mostrará por defecto el estado de la simulación por la salida estándar. Opcionalmente se puede activar la opción `--debug` para guardar este registro a un archivo.