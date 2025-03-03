use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use serde::{Deserialize, Serialize};
use std::process::Command;
use reqwest::Client;


// CREACIÓN DE STRUCT



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
    #[serde(rename = "Cmdline")]
    cmd_line: String,
    #[serde(rename = "MemoryUsage")]
    memory_usage: f64,
    #[serde(rename = "CPUUsage")]
    cpu_usage: f64,
}

#[derive(Debug, Serialize, Clone)]
struct LogProcess {
    pid: u32,
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

    fn get_container_name(&self) -> &str {
        match &self.container_name {
            Some(name) => name, // Si hay un nombre de contenedor, lo retornamos
            None => "", // Si no hay contenedor asociado, retornamos una cadena vacía
        }
    }
}

impl Eq for Process {}  


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
    let client = Client::new();

    let mut log_proc_list: Vec<LogProcess> = Vec::new();

    let mut processes_list: Vec<Process> = system_info.processes;

    processes_list.sort_by(|a, b| a.cpu_usage.partial_cmp(&b.cpu_usage).unwrap());

    let (lowest_list, highest_list) = processes_list.split_at(processes_list.len() / 2);


    println!("Bajo consumo");
    println!("{}",lowest_list.len());
    for process in lowest_list {
        println!("PID: {}, Name: {}, container ID: {}, Memory Usage: {}, CPU Usage: {}", process.pid, process.name, process.get_container_id(), process.memory_usage, process.cpu_usage);
    }

    println!("------------------------------");

    println!("Alto consumo");
    println!("{}",highest_list.len());
    for process in highest_list {
        println!("PID: {}, Name: {}, container ID {}, Memory Usage: {}, CPU Usage: {}", process.pid, process.name,process.get_container_id(),process.memory_usage, process.cpu_usage);
    }

    println!("------------------------------");
    

    // Modificar la copia de la lista de bajo consumo
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

    // Modificar la copia de la lista de alto consumo
    if highest_list.len() > 2 {
        // Iteramos sobre los procesos en la lista de alto consumo.
        for process in highest_list.iter().take(highest_list.len() - 3) {
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

    let res = client.post("http://0.0.0.0:8000/data")
        .json(&processes_list)  // Enviar los datos como JSON 
        .send();

    // Hacemos un print de los contenedores que matamos.
    println!("Contenedores muertos");
    for process in log_proc_list {
        println!("PID: {}, Name: {}, Container ID: {}, Memory Usage: {}, CPU Usage: {} ", process.pid, process.name, process.container_id,  process.memory_usage, process.cpu_usage);
    }

    println!("------------------------------");

    
}


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

fn is_container_running(container_id: &str) -> bool {
    let output = Command::new("sudo")
        .args(&["docker", "inspect", "-f", "{{.State.Running}}", container_id])
        .output()
        .expect("Failed to execute docker inspect");

    if output.status.success() {
        let status = String::from_utf8_lossy(&output.stdout);
        return status.trim() == "true";
    }

    // Retorna `false` si el comando falló
    false
}

fn main() {

    
    
    let output = Command::new("sudo")
    .args(&["docker-compose", "-f", "../python_service/docker-compose.yaml", "up", "--build", "-d"])
    .output()
    .expect("Failed to execute docker-compose");

    if output.status.success() {
        println!("Docker Compose executed successfully!");
    } else {
        eprintln!("Error executing Docker Compose: {:?}", output);
        return;
    }



    // TODO: Utilizar algo para capturar la señal de terminación y matar el contenedor registro y cronjob.

    loop {
        
        
        let system_info: Result<SystemInfo, _>;

        
        let json_str = read_proc_file("sysinfo_201908327").unwrap();

        
        system_info = parse_proc_to_struct(&json_str);

        
        match system_info {
            Ok( info) => {
                analyzer(info);
            }
            Err(e) => println!("Failed to parse JSON: {}", e),
        }

        
        std::thread::sleep(std::time::Duration::from_secs(10));
    }

}
