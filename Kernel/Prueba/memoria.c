#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/string.h>
#include <linux/init.h>
#include <linux/proc_fs.h>
#include <linux/seq_file.h>
#include <linux/mm.h>
#include <linux/sched.h>
#include <linux/timer.h>
#include <linux/jiffies.h>
#include <linux/uaccess.h>
#include <linux/tty.h>
#include <linux/sched/signal.h>
#include <linux/fs.h>
#include <linux/slab.h>
#include <linux/sched/mm.h>
#include <linux/binfmts.h>
#include <linux/timekeeping.h>
#include <linux/statfs.h>
#include <linux/blkdev.h>
#include <linux/path.h>
#include <linux/cgroup.h>
#include <linux/seq_file.h>
#include <linux/uaccess.h>

MODULE_LICENSE("GPL");
MODULE_AUTHOR("Key");
MODULE_DESCRIPTION("Modulo para leer informacion de memoria, CPU y I/O en JSON");
MODULE_VERSION("1.2");

#define PROC_NAME "sysinfo_201908327"
#define MAX_CMDLINE_LENGTH 256
#define CONTAINER_ID_LENGTH 64
#define CGROUP_BLKIO_PATH "/sys/fs/cgroup/blkio/docker/%s/blkio.throttle.io_service_bytes"
#define CONTAINER_NAME_LENGTH 256
#define BUFFER_SIZE 16384


static char *get_container_id(const char *cmdline);
static char *get_container_name_from_id(const char *container_id);
static int get_container_cpu_mem_usage(const char *container_id, unsigned long *cpu_usage, unsigned long *mem_usage);


static char *get_process_cmdline(struct task_struct *task) {
    struct mm_struct *mm;
    char *cmdline, *p;
    unsigned long arg_start, arg_end;
    int i, len;

    cmdline = kmalloc(MAX_CMDLINE_LENGTH, GFP_KERNEL);
    if (!cmdline)
        return NULL;

    mm = get_task_mm(task);
    if (!mm) {
        kfree(cmdline);
        return NULL;
    }

    down_read(&mm->mmap_lock);
    arg_start = mm->arg_start;
    arg_end = mm->arg_end;
    up_read(&mm->mmap_lock);

    len = arg_end - arg_start;
    if (len > MAX_CMDLINE_LENGTH - 1)
        len = MAX_CMDLINE_LENGTH - 1;

    if (access_process_vm(task, arg_start, cmdline, len, 0) != len) {
        mmput(mm);
        kfree(cmdline);
        return NULL;
    }

    cmdline[len] = '\0';

    p = cmdline;
    for (i = 0; i < len; i++)
        if (p[i] == '\0')
            p[i] = ' ';

    mmput(mm);
    return cmdline;
}


char* get_container_id(const char* cmdline) {
    char *container_id = NULL;
    char *start = NULL;
    char *end = NULL;
    int length;

    
    start = strstr(cmdline, "-id ");
    if (!start)
        return NULL;  

    start += 4;  

    end = strchr(start, ' ');
    length = end ? (end - start) : strlen(start); 

    container_id = kmalloc(length + 1, GFP_KERNEL);
    if (!container_id)
        return NULL;

    strncpy(container_id, start, length);
    container_id[length] = '\0'; 

    return container_id;
}


static int get_container_cpu_mem_usage(const char *container_id, unsigned long *cpu_usage, unsigned long *mem_usage) {
    char path[256];
    struct file *file;
    char buf[64];
    loff_t pos = 0;
    int bytes_read;
    int ret;
    
    // Obtener uso de CPU desde cgroup v2
    snprintf(path, sizeof(path), "/sys/fs/cgroup/system.slice/docker-%s.scope/cpu.stat", container_id);
    file = filp_open(path, O_RDONLY, 0);
    if (IS_ERR(file)) {
        printk(KERN_WARNING "No se pudo abrir %s\n", path);
        return -ENOENT;
    }
    
    bytes_read = kernel_read(file, buf, sizeof(buf) - 1, &pos);
    filp_close(file, NULL);
    
    if (bytes_read > 0) {
        buf[bytes_read] = '\0';
        char *cpu_line = strstr(buf, "usage_usec");
        if (cpu_line) {
            cpu_line += 11;  // Saltar "usage_usec "
            ret = kstrtoul(cpu_line, 10, cpu_usage);
            if (ret) {
                printk(KERN_WARNING "Error al convertir uso de CPU a entero\n");
                return ret;
            }
        }
    } else {
        printk(KERN_WARNING "Error al leer CPU usage\n");
        return -EIO;
    }

    // Obtener uso de memoria desde cgroup v2
    snprintf(path, sizeof(path), "/sys/fs/cgroup/system.slice/docker-%s.scope/cpu.stat", container_id);
    file = filp_open(path, O_RDONLY, 0);
    if (IS_ERR(file)) {
        printk(KERN_WARNING "No se pudo abrir %s\n", path);
        return -ENOENT;
    }
    
    pos = 0;
    bytes_read = kernel_read(file, buf, sizeof(buf) - 1, &pos);
    filp_close(file, NULL);
    
    if (bytes_read > 0) {
        buf[bytes_read] = '\0';
        ret = kstrtoul(buf, 10, mem_usage);
        if (ret) {
            printk(KERN_WARNING "Error al convertir uso de memoria a entero\n");
            return ret;
        }
    } else {
        printk(KERN_WARNING "Error al leer memoria usage\n");
        return -EIO;
    }

    return 0;
}



char* get_container_name_from_id(const char* container_id) {
    char path[512];
    char *buf;
    char *container_name = NULL;
    struct file *f;
    ssize_t bytes_read;
    loff_t pos = 0;

    snprintf(path, sizeof(path), "/var/lib/docker/containers/%s/config.v2.json", container_id);
    printk(KERN_INFO "[sysinfo_json] Intentando abrir: %s\n", path);

    buf = kmalloc(BUFFER_SIZE, GFP_KERNEL);
    if (!buf) {
        printk(KERN_ERR "[sysinfo_json] Error al asignar memoria\n");
        return " Error al asignar memoria";
    }
    
    f = filp_open(path, O_RDONLY, 0);
    if (IS_ERR(f)) {
        printk(KERN_ERR "[sysinfo_json] Error al abrir el archivo: %s\n", path);
        kfree(buf);
        return "Error al abrir el archivo";
    }

    bytes_read = kernel_read(f, buf, BUFFER_SIZE - 1, &pos);
    filp_close(f, NULL);

    if (bytes_read <= 0) {
        printk(KERN_ERR "[sysinfo_json] Error al abrir el archivo\n");
        kfree(buf);
        return "Error al abrir el archivo";
    }
    buf[bytes_read] = '\0';  // Asegurar terminación de la cadena

    // Buscar el campo "Name":
    char *name_field = strstr(buf, "\"Name\"");
    if (name_field) {
        name_field = strchr(name_field, ':');
        if (name_field) {
            name_field++;
            while (*name_field == ' ' || *name_field == '"') {
                name_field++;  // Saltar espacios y comillas iniciales
            }
            char *end = name_field;
            while (*end && *end != '"' && *end != ',' && *end != '}') {
                end++;  // Avanzar hasta el final del nombre
            }
            *end = '\0';
            
            if (*name_field == '/') {
                name_field++;  // Omitir el '/' inicial si está presente
            }
            
            container_name = kstrdup(name_field, GFP_KERNEL);
            printk(KERN_INFO "[sysinfo_json] Nombre del contenedor extraído: %s\n", container_name);
        }
    } else {
        printk(KERN_ERR "[sysinfo_json] No se encontró el campo \"Name\" en el JSON\n");
    }

    kfree(buf);
    return container_name ? container_name : NULL;
}



static int read_container_io(const char *container_id, unsigned long *read_bytes, unsigned long *write_bytes) {
    char path[256];
    struct file *file;
    char buf[256];
    int bytes_read;
    loff_t pos = 0;
    
    snprintf(path, sizeof(path), CGROUP_BLKIO_PATH, container_id);
    file = filp_open(path, O_RDONLY, 0);
    if (IS_ERR(file)) {
        return -1;
    }
    
    bytes_read = kernel_read(file, buf, sizeof(buf) - 1, &pos);
    
    if (bytes_read > 0) {
        buf[bytes_read] = '\0';
        sscanf(buf, "Read %lu Write %lu", read_bytes, write_bytes);
    }
    
    filp_close(file, NULL);
    return 0;
}

static int sysinfo_show(struct seq_file *m, void *v) {
    struct sysinfo si;
    struct task_struct *task;
    unsigned long total_jiffies = jiffies;
    int first_process = 1;
    
    si_meminfo(&si);
    unsigned long totalram = si.totalram * 4;  // Ajuste a KB

    seq_printf(m, "{\n  \"Processes\": [\n");

    for_each_process(task) {
        if (strcmp(task->comm, "containerd-shim") == 0) {
            unsigned long vsz = 0, rss = 0;
            unsigned long mem_usage = 0, cpu_usage = 0;
            unsigned long read_bytes = 0, write_bytes = 0;
            unsigned long container_cpu_usage = 0, container_mem_usage = 0;
            char *cmdline = NULL;
            char *container_id = NULL;
            char *container_name = NULL;

            if (task->mm) {
                vsz = task->mm->total_vm << (PAGE_SHIFT - 10);  // KB
                rss = get_mm_rss(task->mm) << (PAGE_SHIFT - 10); // KB
                mem_usage = (rss * 10000) / totalram;  
            }

            unsigned long total_time = task->utime + task->stime;
            cpu_usage = (total_time * 10000) / total_jiffies;  

            // Obtener línea de comando del proceso
            cmdline = get_process_cmdline(task);

            // Obtener ID del contenedor desde el comando
            container_id = get_container_id(cmdline);
            if (container_id) {
                container_name = get_container_name_from_id(container_id);
            } else {
                container_id = "NO ID";  // En caso de que no se pueda obtener el ID
            }

            // Obtener estadísticas del contenedor
            read_container_io(container_id, &read_bytes, &write_bytes);
            get_container_cpu_mem_usage(container_id, &container_cpu_usage, &container_mem_usage);

            // Manejo del formato JSON (agregar coma solo si no es el primer proceso)
            if (!first_process) {
                seq_printf(m, ",\n");
            }
            first_process = 0;

            // Imprimir información del proceso en formato JSON
            seq_printf(m, "    {\n");
            seq_printf(m, "      \"PID\": %d,\n", task->pid);
            seq_printf(m, "      \"Name\": \"%s\",\n", task->comm);
            seq_printf(m, "      \"Container_id\": \"%s\",\n", container_id);
            seq_printf(m, "      \"Container_name\": \"%s\",\n", container_name ? container_name : "FALLO DENTRO");
            seq_printf(m, "      \"Cmdline\": \"%s\",\n", cmdline ? cmdline : "N/A");
            seq_printf(m, "      \"TotalMemory\": %lu,\n", totalram);
            seq_printf(m, "      \"FreeMemory\": %lu,\n", totalram - rss);
            seq_printf(m, "      \"MemoryUsage\": %lu.%02lu,\n", mem_usage / 100, mem_usage % 100);
            seq_printf(m, "      \"CPUUsage\": %lu.%02lu,\n", cpu_usage / 100, cpu_usage % 100);
            seq_printf(m, "      \"ContainerCPUUsage\": %lu.%02lu,\n", container_cpu_usage / 100, container_cpu_usage % 100);
            seq_printf(m, "      \"ContainerMemoryUsage\": %lu.%02lu,\n", container_mem_usage / 100, container_mem_usage % 100);
            seq_printf(m, "      \"ReadBytes\": %lu,\n", read_bytes);
            seq_printf(m, "      \"WriteBytes\": %lu\n", write_bytes);
            seq_printf(m, "    }");

            // Liberar memoria asignada
            if (cmdline) {
                kfree(cmdline);
            }
            if (container_name) {
                kfree(container_name);
            }
        }
    }

    seq_printf(m, "\n  ]\n}\n");
    return 0;
}


static int sysinfo_open(struct inode *inode, struct file *file) {
    return single_open(file, sysinfo_show, NULL);
}

static const struct proc_ops sysinfo_ops = {
    .proc_open = sysinfo_open,
    .proc_read = seq_read,
};

static int __init sysinfo_init(void) {
    proc_create(PROC_NAME, 0, NULL, &sysinfo_ops);
    printk(KERN_INFO "sysinfo_json modulo cargado\n");
    return 0;
}

static void __exit sysinfo_exit(void) {
    remove_proc_entry(PROC_NAME, NULL);
    printk(KERN_INFO "sysinfo_json modulo desinstalado\n");
}

module_init(sysinfo_init);
module_exit(sysinfo_exit);

