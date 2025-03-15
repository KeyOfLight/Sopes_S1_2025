


##### INSTALAR MODULO DE KERNEL


make


sudo insmod memoria.ko
cat /proc/sysinfo_201908327
lsmod | grep memoria

### Verificar que este to bien
cat /proc/sysinfo_201908327

sudo dmesg | grep memoria

##### Remover Modulo de kernl

sudo rmmod memoria
make clean

cat /var/lib/docker/containers/456ad86011de2f6393718964f0c871c0320f842fcb11a6c6b78fcb219cd235a5/config.v2.json


### Comando para borrar todo

```bash
sudo docker stop $(sudo docker ps -a -q)
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