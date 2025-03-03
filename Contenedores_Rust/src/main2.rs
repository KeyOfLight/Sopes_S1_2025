use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize)]
struct SystemInfo {
    #[serde(rename = "Processes")]
    processes: Vec<Process>
}


#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Process {
    #[serde(rename = "PID")]
    pid: u32,
    #[serde(rename = "Name")]
    name: String,
    // #[serde(rename = "Docker_ID")]
    // docker_id: String,
    #[serde(rename = "Cmd_line")]
    cmd_line: String,
    // #[serde(rename = "TotalMemory")]
    // total_memory: u64,  
    // #[serde(rename = "FreeMemory")]
    // free_memory: u64,   
    // #[serde(rename = "VSZ")]
    // vsz: u64,           
    // #[serde(rename = "RSS")]
    // rss: u64,          
    #[serde(rename = "MemoryUsage")]
    memory_usage: f64,  
    #[serde(rename = "CPUUsage")]
    cpu_usage: f64,     
}

#[derive(Debug, Serialize, Clone)]
struct LogProcess {
    pid: u32,
    // docker_id: String,
    // cmd_line: String,
    // total_memory: u64,
    // free_memory: u64,
    // vsz: u64,  
    // rss: u64,      
    container_id: String,
    name: String,
    memory_usage: f64,
    cpu_usage: f64,
}


impl Process {
    fn get_container_id(&self) -> &str {
        let parts: Vec<&str> = self.cmd_line.split_whitespace().collect();
        for (i, part) in parts.iter().enumerate() {
            if *part == "-id" {
                if let Some(id) = parts.get(i + 1) {
                    return id;
                }
            }
        }
        "N/A"
    }
}



impl Eq for Process {}  


/* 
    Ord Trait:
    Funcionalidad: Proporciona una comparación total para dos instancias de Process. 
    Devuelve un std::cmp::Ordering que puede ser Less, Greater o Equal.
    Ejecución: Si partial_cmp devuelve Some(Ordering), sort usará el resultado de cmp para ordenar los elementos. 
    La implementación de cmp en Process compara primero el uso de CPU y, si es igual, compara el uso de memoria.
    
    ¿Qué significa esto?
        - Permite comparar procesos basándose en su uso de CPU y memoria.
        - Si el uso de CPU de un proceso es mayor que el de otro, el proceso con mayor uso de CPU es considerado mayor.
        - Si el uso de CPU de ambos procesos es igual, se comparan en base a su uso de memoria.
        - Si tanto el uso de CPU como el de memoria son iguales, los procesos se consideran iguales.

    Detalles de implementación:
        - Se utiliza unwrap_or para devolver std::cmp::Ordering::Equal en caso de que haya un valor NaN.
        - El método then_with se usa para comparar en base a la memoria si el uso de CPU es igual.
        - Los || no son necesarios aquí ya que unwrap_or maneja los valores NaN.

    Se pueden agregar más condiciones para comparar en base a otros campos si es necesario.
*/
impl Ord for Process {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.cpu_usage.partial_cmp(&other.cpu_usage).unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| self.memory_usage.partial_cmp(&other.memory_usage).unwrap_or(std::cmp::Ordering::Equal))
    }
}


impl PartialOrd for Process {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}


fn kill_container(id: &str) -> std::process::Output {
    let  output = std::process::Command::new("sudo")
        .arg("docker")
        .arg("stop")
        .arg(id)
        .output()
        .expect("failed to execute process");

    println!("Matando contenedor con id: {}", id);

    output
}

fn analyzer( system_info:  SystemInfo) {

    let mut log_proc_list: Vec<LogProcess> = Vec::new();
    let mut processes_list: Vec<Process> = system_info.processes;

    processes_list.sort();


    let (lowest_list, highest_list) = processes_list.split_at(processes_list.len() / 2);


    println!("Bajo consumo");
    for process in lowest_list {
        println!("PID: {}, Name: {}, container ID: {}, Memory Usage: {}, CPU Usage: {}", process.pid, process.name, process.get_container_id(), process.memory_usage, process.cpu_usage);
    }

    println!("------------------------------");

    println!("Alto consumo");
    for process in highest_list {
        println!("PID: {}, Name: {}, Icontainer ID {}, Memory Usage: {}, CPU Usage: {}", process.pid, process.name,process.get_container_id(),process.memory_usage, process.cpu_usage);
    }

    println!("------------------------------");

    
    if lowest_list.len() > 3 {
        // Iteramos sobre los procesos en la lista de bajo consumo.
        for process in lowest_list.iter().skip(3) {
            let log_process = LogProcess {
                pid: process.pid,
                container_id: process.get_container_id().to_string(),
                name: process.name.clone(),
                memory_usage: process.memory_usage,
                cpu_usage: process.cpu_usage,
            };
    
            log_proc_list.push(log_process.clone());

            // Matamos el contenedor.
            let _output = kill_container(&process.get_container_id());

        }
    } 

    
    if highest_list.len() > 2 {
        // Iteramos sobre los procesos en la lista de alto consumo.
        for process in highest_list.iter().take(highest_list.len() - 2) {
            let log_process = LogProcess {
                pid: process.pid,
                container_id: process.get_container_id().to_string(),
                name: process.name.clone(),
                memory_usage: process.memory_usage,
                cpu_usage: process.cpu_usage
            };
    
            log_proc_list.push(log_process.clone());

            // Matamos el contenedor.
            let _output = kill_container(&process.get_container_id());

        }
    }

    // TODO: ENVIAR LOGS AL CONTENEDOR REGISTRO

    

    // Hacemos un print de los contenedores que matamos.
    println!("Contenedores matados");
    for process in log_proc_list {
        println!("PID: {}, Name: {}, Container ID: {}, Memory Usage: {}, CPU Usage: {} ", process.pid, process.name, process.container_id,  process.memory_usage, process.cpu_usage);
    }

    println!("------------------------------");

    
}

/*  
    Función para leer el archivo proc
    - file_name: El nombre del archivo que se quiere leer.
    - Regresa un Result<String> que puede ser un error o el contenido del archivo.
*/
fn read_proc_file(file_name: &str) -> io::Result<String> {
    
    let path  = Path::new("/proc").join(file_name);
    let mut file = File::open(path)?;
    let mut content = String::new();

    file.read_to_string(&mut content)?;

    Ok(content)
}


fn parse_proc_to_struct(json_str: &str) -> Result<SystemInfo, serde_json::Error> {
    let system_info: SystemInfo = serde_json::from_str(json_str)?;

    
    Ok(system_info)
}



fn main() {

    // TODO: antes de iniciar el loop, ejecutar el docker-compose.yml y obtener el id del contenedor registro.

    // docker-compose up --build -d

    // let output = Command::new("docker-compose")
    // .args(&["up", "--build", "-d"])
    // .output()
    // .expect("Failed to execute docker-compose");

    // if output.status.success() {
    //     println!("Docker Compose executed successfully!");
    // } else {
    //     eprintln!("Error executing Docker Compose: {:?}", output);
    //     return;
    // }



    // TODO: Utilizar algo para capturar la señal de terminación y matar el contenedor registro y cronjob.

    loop {
        
        // Creamos una estructura de datos SystemInfo con un vector de procesos vacío.
        let system_info: Result<SystemInfo, _>;

        
        let json_str = read_proc_file("sysinfo_201908327").unwrap();

        // Deserializamos el contenido del archivo proc a un SystemInfo.
        system_info = parse_proc_to_struct(&json_str);
        println!("{:?}", system_info);
        // Dependiendo de si se pudo deserializar el contenido del archivo proc o no, se ejecuta una u otra rama.
        match system_info {
            Ok( info) => {
                analyzer(info);
            }
            Err(e) => println!("Failed to parse JSON: {}", e),
        }

        std::thread::sleep(std::time::Duration::from_secs(10));
    }

}
