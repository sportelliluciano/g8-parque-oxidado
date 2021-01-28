use rand::Rng;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Args {
    pub debug: bool,
    pub capacidad_parque: u32,
    pub presupuesto_personas: Vec<u32>,
    pub costo_juegos: Vec<u32>,
    pub semilla: u32,
}

pub enum ParseArgsResult {
    Ok(Args),
    MostrarAyuda,
    Error(String)
}

impl ParseArgsResult {
    pub fn error(param: &str, err: String) -> ParseArgsResult {
        ParseArgsResult::Error(format!("[{}] {}", param, err))
    }
}

type Parser = fn(&mut Args, &str) -> Result<(), String>;

pub fn parse_args() -> ParseArgsResult {
    let mut args = Args::default();
    let parsers = Args::parsers();

    for arg in std::env::args().skip(1) {
        let val = arg.split('=').collect::<Vec<&str>>();
        if val[0] == "-h" || val[0] == "--help" {
            return ParseArgsResult::MostrarAyuda;
        } else if val[0] == "-d" || val[0] == "--debug" {
            args.debug = true;
            continue;
        }

        if val.len() != 2 {
            return ParseArgsResult::error(&arg, "Argumento inválido".into());
        }

        let (argname, argvalue) = (val[0], val[1]);
        match parsers.get(argname) {
            Some(parser) => {
                match parser(&mut args, argvalue) {
                    Ok(_) => continue,
                    Err(e) => return ParseArgsResult::error(argname, e)
                }
            },
            None => return ParseArgsResult::error(&argname, "Argumento inválido".into())
        };
    }

    ParseArgsResult::Ok(args)
}

pub fn mostrar_ayuda() {
    let args: Vec<String> = std::env::args().collect();
    println!("Uso: {} [--personas=<PERSONAS>] [--juegos=<JUEGOS>] [--capacidad=N] [-h|--help]",
        args[0]);
    println!("\t --personas=<PERSONAS>: Cantidad de personas que ingresarán al parque");
    println!("\t y presupuesto inicial de las mismas");
    println!("\t --personas=n1,n2,n3,...,nk: Ingresarán k personas con presupuestos iniciales n1, n2, n3, ..., nk");
    println!("\t --personas=N:P: Ingresarán N personas, todas con presupuesto inicial P.");
    println!("\t --personas=N:Pm:PM: Ingresarán N personas con presupuestos iniciales aleatorios en el rango [Pm,PM)");
    println!();
    println!("\t --juegos=<JUEGOS>: Cantidad de juegos que habrá en el parque y costo de los mismos");
    println!("\t --juegos=c1,c2,c3,...,ck: Habrá k juegos con costos c1, c2, c3, ..., ck");
    println!("\t --juegos=N:C: Habrá N juegos, todos con costo C.");
    println!("\t --juegos=N:Cm:CM: Habrá N juegos con costos aleatorios en el rango [Cm,CM)");
    println!();
    println!("\t --capacidad=N: Cantidad de personas que puede haber simultáneamente");
    println!("\t dentro del parque en un momento dado.");
    println!("\t -d|--debug: Habilitar registro a un archivo.");
    println!("\t -h|--help: Muestra esta ayuda.");
    println!("\t --semilla=N: Semilla aleatoria a utilizar.");
}

impl Args {
    pub fn default() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            debug: false,
            capacidad_parque: 10,
            presupuesto_personas: vec![40, 40, 40, 40, 40],
            costo_juegos: vec![10, 10, 10, 10, 10],
            semilla: rng.gen()
        }
    }

    pub fn as_str(&self) -> String {
        let exe = &std::env::args().collect::<Vec<String>>()[0];
        let debug = if self.debug { "-d" } else { "" };
        let personas = self.presupuesto_personas
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<String>>()
            .join(",");
        let juegos = self.costo_juegos
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<String>>()
            .join(",");


        return format!("{} --capacidad={} --personas={} --juegos={} --semilla={} {}",
            exe, self.capacidad_parque, personas, juegos, self.semilla, debug);
    }

    pub fn parsers() -> HashMap<&'static str, Parser> {
        let mut result: HashMap<&'static str, Parser> = HashMap::new();
        result.insert("--personas", Self::parse_personas);
        result.insert("--juegos", Self::parse_juegos);
        result.insert("--capacidad", Self::parse_capacidad);
        result.insert("--semilla", Self::parse_semilla);
        result
    }

    fn parse_personas(args: &mut Args, data: &str) -> Result<(), String> {
        args.presupuesto_personas = Self::parse_array(data)?;
        Ok(())
    }

    fn parse_juegos(args: &mut Args, data: &str) -> Result<(), String> {
        args.costo_juegos = Self::parse_array(data)?;
        Ok(())
    }

    fn parse_capacidad(args: &mut Args, data: &str) -> Result<(), String> {
        args.capacidad_parque = Self::parse_u32(data)?;
        Ok(())
    }

    fn parse_semilla(args: &mut Args, data: &str) -> Result<(), String> {
        args.semilla = Self::parse_u32(data)?;
        Ok(())
    }

    fn parse_array(data: &str) -> Result<Vec<u32>, String> {
        // Formatos posibles:
        // N,N,N,N
        // N:P
        // N:Pm:PM

        let mut resultado = vec![];
        let partes: Vec<&str> = data.split(':').collect();
        
        if partes.is_empty() || partes.len() > 3 {
            return Err("Formato inválido".into());
        }
        
        if partes.len() == 1 {
            // N,N,N,N
            for parte in partes[0].split(',') {
                resultado.push(Self::parse_u32(parte)?);
            }
            return Ok(resultado);
        }
        
        // N:P o N:Pm:PM
        let n_personas = Self::parse_u32(&partes[0])?;
        let presupuesto_min = Self::parse_u32(&partes[1])?;
        let presupuesto_max = if partes.len() == 3 {
            Self::parse_u32(&partes[2])?
        } else {
            presupuesto_min
        };

        if presupuesto_min > presupuesto_max {
            return Err(format!("Rango inválido ({} > {})", presupuesto_min,
                presupuesto_max));
        }
        
        if presupuesto_min == presupuesto_max {
            // N:P
            for _ in 0..n_personas {
                resultado.push(presupuesto_min);
            }
        } else {
            // N:Pm:PM
            let mut rng = rand::thread_rng();
            for _ in 0..n_personas {
                resultado.push(
                    rng.gen_range(presupuesto_min..presupuesto_max)
                );
            }
        }
        
        Ok(resultado)
    }

    fn parse_u32(data: &str) -> Result<u32, String> {
        let ret = data.parse::<u32>()
            .map_err(|_| format!("'{}' no es un número natural", data))?;
        if ret == 0 {
            Err(format!("'{}' no es un número natural", ret))
        } else {
            Ok(ret)
        }
    }
}