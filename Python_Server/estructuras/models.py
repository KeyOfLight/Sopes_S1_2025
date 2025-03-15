from pydantic import BaseModel # type: ignore
from typing import List




class LogProcess(BaseModel):
    pid: int
    container_id: str
    name: str
    vsz: int
    rss: int
    memory_usage: float
    cpu_usage: float
    uso_disk: int
    io: str
    action: str
    timestamp: str



