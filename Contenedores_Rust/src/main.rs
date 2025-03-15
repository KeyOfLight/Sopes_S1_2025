use std::fs::File;
use std::io::{self, Read};
use std::os::unix::process;
use std::path::Path;
use serde::{Deserialize, Serialize};
use std::process::Command;
use reqwest::blocking::Client;
use serde_json::Value;
use serde_json::json;
use chrono::Utc;

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
    #[serde(rename = "TotalMemory")]
    total_memory: f64,
    #[serde(rename = "FreeMemory")]
    free_memory: f64,
}

#[derive(Debug, Serialize, Clone)]
struct LogProcess {
    pid: u32,
    container_id: String,
    name: String,
    memory_usage: f64,
    cpu_usage: f64,
    free_memory: f64,
    total_memory: f64,
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


    // fn get_container_type(&self) -> String {
    //     let container_id = self.get_container_id();
    //     let file_path = format!("/var/lib/docker/containers/{}/config.v2.json", container_id);
    
    //     // let mut file = match File::open(&file_path) {
    //     //     Ok(file) => file,
    //     //     Err(err) => {
    //     //         eprintln!("Error abriendo el archivo: {}", err);
    //     //         return "cpu".to_string();
    //     //     }
    //     // };

    //     let mut contents = String::new();
    //     if let Err(err) = file.read_to_string(&mut contents) {
    //         eprintln!("Error leyendo el archivo: {}", err);
    //         return "cpu".to_string();
    //     }
    
    //     let json: serde_json::Value = match serde_json::from_str(&contents) {
    //         Ok(json) => json,
    //         Err(err) => {
    //             eprintln!("Error parseando JSON: {}", err);
    //             return "cpu".to_string();
    //         }
    //     };
    
    //     if let Some(cmd) = json["Config"]["Cmd"].as_array() {
    //         let cmd_values: Vec<String> = cmd.iter().filter_map(|c| c.as_str().map(String::from)).collect();
    //         println!("Cmd: {:?}", cmd_values);
    
    //         // Devuelve el primer valor o "cpu" si no hay ninguno
    //         return cmd_values.first().cloned().unwrap_or_else(|| "cpu".to_string());
    //     } else {
    //         println!("No se encontró la clave 'Cmd'");
    //         return "cpu".to_string();
    //     }
    // }

    fn get_container_type(&self) -> String {
        let container_id = self.get_container_id();
        let file_path = format!("/var/lib/docker/containers/{}/config.v2.json", container_id);
    
    
        let output = Command::new("sudo")
            .arg("cat")
            .arg(&file_path)
            .output();
    
    
        let output = match output {
            Ok(output) if output.status.success() => output.stdout,
            Ok(output) => {
                eprintln!(
                    "Error ejecutando sudo cat: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                return "cpu".to_string();
            }
            Err(err) => {
                eprintln!("Error al ejecutar sudo: {}", err);
                return "python".to_string();
            }
        };
    
    
        let contents = String::from_utf8_lossy(&output);
    
    
        let json: Value = match serde_json::from_str(&contents) {
            Ok(json) => json,
            Err(err) => {
                eprintln!("Error parseando JSON: {}", err);
                return "python".to_string();
            }
        };
    
        let cmd_array = json
        .get("Config")
        .and_then(|config| config.get("Cmd"))
        .and_then(|cmd| cmd.as_array());

        if let Some(cmd_array) = cmd_array {
            let cmd_values: Vec<String> = cmd_array.iter().filter_map(|c| c.as_str().map(String::from)).collect();
            println!("Cmd: {:?}", cmd_values);

            if let Some(index) = cmd_values.iter().position(|x| x == "stress") {
                if let Some(next_value) = cmd_values.get(index + 1) {
                    return next_value.trim_start_matches('-').to_string();
                }
            }
        }

        "python".to_string()
    }


    
    fn get_container_name(&self) -> String {
        let container_id = self.get_container_id();
        let file_path = format!("/var/lib/docker/containers/{}/config.v2.json", container_id);
    
    
        let output = Command::new("sudo")
            .arg("cat")
            .arg(&file_path)
            .output();
    
    
        let output = match output {
            Ok(output) if output.status.success() => output.stdout,
            Ok(output) => {
                eprintln!(
                    "Error ejecutando sudo cat: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                return "cpu".to_string();
            }
            Err(err) => {
                eprintln!("Error al ejecutar sudo: {}", err);
                return "python".to_string();
            }
        };
    
    
        let contents = String::from_utf8_lossy(&output);
    
    
        let json: Value = match serde_json::from_str(&contents) {
            Ok(json) => json,
            Err(err) => {
                eprintln!("Error parseando JSON: {}", err);
                return "python".to_string();
            }
        };
    
        let cmd_array = json.get("Name");
        // .and_then(|config| config.get("Cmd"))
        // .and_then(|cmd| cmd.as_array());

        // if let Some(cmd_array) = cmd_array {
        //     let cmd_values: Vec<String> = cmd_array.iter().filter_map(|c| c.as_str().map(String::from)).collect();
        //     println!("Name: {:?}", cmd_values);

        //     if let Some(name) = json.get("Name").and_then(|v| v.as_str()) {
        //         return name.trim_start_matches('/').to_string();
        //     }
        // }

        if let Some(name) = json.get("Name").and_then(|v| v.as_str()) {
            return name.trim_start_matches('/').to_string();
        }

        "N/A".to_string()
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


    let (cpu_list, other_containers): (Vec<&Process>, Vec<&Process>) = 
    processes_list.iter()
    .partition(|c| c.get_container_type() == "cpu");

    let (vm_list, other_containers): (Vec<&Process>, Vec<&Process>) = 
        other_containers.into_iter()
        .partition(|c| c.get_container_type() == "vm");

    let (io_list, other_containers): (Vec<&Process>, Vec<&Process>) = 
        other_containers.into_iter()
        .partition(|c| c.get_container_type() == "io");

    let (hdd_list, other_containers): (Vec<&Process>, Vec<&Process>) = 
        other_containers.into_iter()
        .partition(|c| c.get_container_type() == "hdd");


    println!("cpu");
    println!("{}",cpu_list.len());
    for process in &cpu_list {
        println!("PID: {}, Name: {}, container ID: {}, Memory Usage: {}, CPU Usage: {}, tipo: {}", process.pid, process.get_container_name(), process.get_container_id(), process.memory_usage, process.cpu_usage, process.get_container_type());
    }

    println!("vm_list");
    println!("{}",vm_list.len());
    for process in &vm_list {
        println!("PID: {}, Name: {}, container ID: {}, Memory Usage: {}, CPU Usage: {}, tipo: {}", process.pid, process.get_container_name(), process.get_container_id(), process.memory_usage, process.cpu_usage, process.get_container_type());
    }

    println!("io_list");
    println!("{}",io_list.len());
    for process in &io_list {
        println!("PID: {}, Name: {}, container ID: {}, Memory Usage: {}, CPU Usage: {}, tipo: {}", process.pid, process.get_container_name(), process.get_container_id(), process.memory_usage, process.cpu_usage, process.get_container_type());
    }

    println!("hdd_list");
    println!("{}",hdd_list.len());
    for process in &hdd_list {
        println!("PID: {}, Name: {}, container ID: {}, Memory Usage: {}, CPU Usage: {}, tipo: {}", process.pid, process.get_container_name(), process.get_container_id(), process.memory_usage, process.cpu_usage, process.get_container_type());
    }


    println!("------------------------------");
  
    if cpu_list.len() > 2 {
        for process in cpu_list.iter().skip(1) {
            let log_process = LogProcess {
                pid: process.pid,
                container_id: process.get_container_id().to_string(),
                name: process.get_container_name().to_string(),//process.name.clone(),
                memory_usage: process.memory_usage,
                cpu_usage: process.cpu_usage,
                free_memory: process.free_memory,
                total_memory: process.total_memory,
            };
    
            log_proc_list.push(log_process.clone());

            let _output = kill_container(&process.get_container_id());

        }
    } 

    if hdd_list.len() > 1 {
        for process in hdd_list.iter().skip(1) {
            let log_process = LogProcess {
                pid: process.pid,
                container_id: process.get_container_id().to_string(),
                name: process.get_container_name().to_string(),//process.name.clone(),
                free_memory: process.free_memory,
                total_memory: process.total_memory,
                memory_usage: process.memory_usage,
                cpu_usage: process.cpu_usage,
            };
    
            log_proc_list.push(log_process.clone());

            // Matamos el contenedor.
            let _output = kill_container(&process.get_container_id());

        }
    } 

    
    if io_list.len() > 1 {
        
        for process in io_list.iter().take(io_list.len() - 1) {
            let log_process = LogProcess {
                pid: process.pid,
                container_id: process.get_container_id().to_string(),
                free_memory: process.free_memory,
                total_memory: process.total_memory,
                name: process.get_container_name().to_string(),//process.name.clone(),
                memory_usage: process.memory_usage,
                cpu_usage: process.cpu_usage
            };
    
            log_proc_list.push(log_process.clone());

            // Matamos el contenedor.
            let _output = kill_container(&process.get_container_id());

        }
    }

    if vm_list.len() > 1 {
        
        for process in io_list.iter().take(io_list.len() - 1) {
            let log_process = LogProcess {
                pid: process.pid,
                container_id: process.get_container_id().to_string(),
                free_memory: process.free_memory,
                total_memory: process.total_memory,
                name: process.get_container_name().to_string(),//process.name.clone(),
                memory_usage: process.memory_usage,
                cpu_usage: process.cpu_usage
            };
    
            log_proc_list.push(log_process.clone());

            // Matamos el contenedor.
            let _output = kill_container(&process.get_container_id());

        }
    }

    let log_proc_list: Vec<serde_json::Value> = processes_list.iter().map(|process| {
        json!({
            "pid": process.pid,
            "container_id": process.get_container_id(),
            "name": process.get_container_name().clone(),
            "memory_usage": process.memory_usage,
            "cpu_usage": process.cpu_usage,
            "vsz": 0 ,  // Ajustar si el dato no está disponible
            "rss": 0,  // Ajustar si el dato no está disponible
            "action": process.get_container_type(),  // Definir acción adecuada
            "timestamp": Utc::now().to_rfc3339() // Timestamp en formato ISO8601
        })
    }).collect();
    
    // 🔹 Imprimir JSON antes de enviarlo para verificar formato
    println!("JSON formateado para enviar:");
    println!("{}", serde_json::to_string_pretty(&log_proc_list).unwrap());
    
    // 🔹 Enviar datos al servidor
    let res = client.post("http://0.0.0.0:8000/logs")
        .json(&log_proc_list)  // Ahora `log_proc_list` es JSON válido
        .send();
        println!("Enviado.");

        match res {
        Ok(response) => {
            println!("Enviado correctamente. Código de estado: {}", response.status());
            match response.text() {
                Ok(text) => println!("Respuesta del servidor: {}", text),
                Err(e) => println!("Error al leer la respuesta: {}", e),
            }
        },
        Err(e) => println!("Error al enviar la solicitud: {}", e),
    }
    
    
    // println!("Contenedores muertos");
    // for process in log_proc_list {
    //     println!("PID: {}, Name: {}, Container ID: {}, Memory Usage: {}, CPU Usage: {} ", process.pid, process.name, process.container_id,  process.memory_usage, process.cpu_usage);
    // }

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
    .args(&["docker-compose", "-f", "../Python_Server/docker-compose.yaml", "up", "--build", "-d"])
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