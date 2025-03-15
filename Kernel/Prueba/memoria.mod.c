#include <linux/module.h>
#define INCLUDE_VERMAGIC
#include <linux/build-salt.h>
#include <linux/elfnote-lto.h>
#include <linux/export-internal.h>
#include <linux/vermagic.h>
#include <linux/compiler.h>

#ifdef CONFIG_UNWINDER_ORC
#include <asm/orc_header.h>
ORC_HEADER;
#endif

BUILD_SALT;
BUILD_LTO_INFO;

MODULE_INFO(vermagic, VERMAGIC_STRING);
MODULE_INFO(name, KBUILD_MODNAME);

__visible struct module __this_module
__section(".gnu.linkonce.this_module") = {
	.name = KBUILD_MODNAME,
	.init = init_module,
#ifdef CONFIG_MODULE_UNLOAD
	.exit = cleanup_module,
#endif
	.arch = MODULE_ARCH_INIT,
};

#ifdef CONFIG_RETPOLINE
MODULE_INFO(retpoline, "Y");
#endif



static const struct modversion_info ____versions[]
__used __section("__versions") = {
	{ 0x9ed12e20, "kmalloc_large" },
	{ 0xcd0bbddf, "filp_open" },
	{ 0xee85401f, "kernel_read" },
	{ 0x5b332080, "filp_close" },
	{ 0x1e6d26a8, "strstr" },
	{ 0x349cba85, "strchr" },
	{ 0x2d39b0a7, "kstrdup" },
	{ 0x37a0cba, "kfree" },
	{ 0xf0fdf6cb, "__stack_chk_fail" },
	{ 0x3addb2e7, "remove_proc_entry" },
	{ 0xbcab6ee6, "sscanf" },
	{ 0x87a21cb3, "__ubsan_handle_out_of_bounds" },
	{ 0x5c3c7387, "kstrtoull" },
	{ 0x15ba50a6, "jiffies" },
	{ 0x40c7247c, "si_meminfo" },
	{ 0x2f9b7e16, "seq_printf" },
	{ 0xddf241a2, "init_task" },
	{ 0xe2d5255a, "strcmp" },
	{ 0x4c03a563, "random_kmalloc_seed" },
	{ 0x24980310, "kmalloc_caches" },
	{ 0x1d199deb, "kmalloc_trace" },
	{ 0x53ecc78, "get_task_mm" },
	{ 0x668b19a1, "down_read" },
	{ 0x53b954a2, "up_read" },
	{ 0x54f14e33, "access_process_vm" },
	{ 0x847442f5, "mmput" },
	{ 0xeb233a45, "__kmalloc" },
	{ 0x9166fada, "strncpy" },
	{ 0x754d539c, "strlen" },
	{ 0xcbd4898c, "fortify_panic" },
	{ 0x79ccc597, "seq_read" },
	{ 0xbdfb6dbb, "__fentry__" },
	{ 0x4f4ab2c1, "proc_create" },
	{ 0x122c3a7e, "_printk" },
	{ 0x5b8239ca, "__x86_return_thunk" },
	{ 0x3ca1d81f, "single_open" },
	{ 0x656e4a6e, "snprintf" },
	{ 0x6ad2b3e, "module_layout" },
};

MODULE_INFO(depends, "");


MODULE_INFO(srcversion, "4FAF146C16AD2D82B030560");
