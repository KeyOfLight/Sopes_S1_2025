#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/init.h>
#include <linux/cgroup.h>
#include <linux/memcontrol.h>

MODULE_LICENSE("GPL");
MODULE_AUTHOR("Key");
MODULE_DESCRIPTION("Módulo de Kernel para leer la memoria usada por Docker");

static int __init docker_mem_usage_init(void)
{
    printk(KERN_INFO "memoria cargada por docker.\n");
    // Tu código para leer memoria usada
    return 0; // Si es 0, significa que el módulo se cargó correctamente
}

static void __exit docker_mem_usage_exit(void)
{
    printk(KERN_INFO "memoria sin cargar de docker.\n");
}

module_init(docker_mem_usage_init);
module_exit(docker_mem_usage_exit);