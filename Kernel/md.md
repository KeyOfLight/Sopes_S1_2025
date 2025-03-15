


##### INSTALAR MODULO DE KERNEL


make


sudo insmod memoria.ko
lsmod | grep memoria

### Verificar que este to bien
cat /proc/sysinfo_201908327


##### Remover Modulo de kernl

sudo rmmod memoria
make clean




### Comando para borrar todo

```bash
sudo docker rm $(sudo docker ps -a -q)
```

### iniciar contenedores

```bash
sudo docker start $(sudo docker ps -a -q)
```

### Parar contendores

```bash
sudo docker stop $(sudo docker ps -a -q)
```