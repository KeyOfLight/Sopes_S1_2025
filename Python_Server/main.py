from fastapi import FastAPI, HTTPException
from typing import List
from pydantic import BaseModel
import json
from pathlib import Path
import threading

app = FastAPI()

LOGS_DIR = Path("./logs")  # Ajusta la ruta según sea necesario
LOGS_DIR.mkdir(parents=True, exist_ok=True)
LOGS_FILE = LOGS_DIR / "logs.json"

# Bloqueo para manejar concurrencia
lock = threading.Lock()

# Modelo de datos
class LogProcess(BaseModel):
    pid: int
    container_id: str
    name: str
    vsz: int
    rss: int
    memory_usage: float
    cpu_usage: float
    uso_disk: int
    action: str
    io: str
    timestamp: str

@app.get("/")
def read_root():
    return {"message": "Sistema de monitoreo activo"}

@app.post("/logs")
def receive_logs(logs_proc: List[LogProcess]):
    """Recibe una lista de logs y los almacena en logs.json"""
    
    # Imprimir logs recibidos en la consola para depuración
    print("Datos recibidos:", [log.dict() for log in logs_proc])

    with lock:  # Evita problemas de concurrencia
        logs = []
        
        # Intentamos cargar los logs previos
        if LOGS_FILE.exists():
            try:
                with LOGS_FILE.open("r") as file:
                    logs = json.load(file)
            except (json.JSONDecodeError, FileNotFoundError):
                logs = []  # Si hay un error, iniciamos un archivo nuevo

        # Verificar si alguno de los container_id ya está registrado
        container_ids_existentes = {log['container_id'] for log in logs}
        nuevos_logs = []
        
        for log in logs_proc:
            if log.container_id in container_ids_existentes:
                print(f"El container_id {log.container_id} ya está registrado. Ignorando log.")
                continue  # Ignorar este log y pasar al siguiente
            nuevos_logs.append(log.dict())
        
        # Agregar los nuevos logs al archivo
        logs.extend(nuevos_logs)

        # Guardamos el archivo con indentación para mejor lectura
        try:
            with LOGS_FILE.open("w") as file:
                json.dump(logs, file, indent=4, ensure_ascii=False)  # Asegura que los caracteres especiales se guarden correctamente
        except Exception as e:
            raise HTTPException(status_code=500, detail=f"Error al guardar logs: {str(e)}")

    return {"status": "Logs guardados correctamente", "total_logs": len(nuevos_logs)}

@app.get("/logs_get")
def get_logs():
    """Devuelve todos los logs almacenados en logs.json"""
    if LOGS_FILE.exists():
        try:
            with LOGS_FILE.open("r") as file:
                logs = json.load(file)
            return {"logs": logs}
        except (json.JSONDecodeError, FileNotFoundError):
            return {"logs": [], "message": "El archivo de logs está corrupto o vacío"}
    else:
        return {"logs": [], "message": "No hay logs almacenados"}
