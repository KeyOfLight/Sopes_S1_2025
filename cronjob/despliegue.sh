#!/bin/bash


generate_random_name() {
    cat /dev/urandom | tr -dc 'a-zA-Z0-9' | head -c 8
}

images=(
    "low_image"
    "high_image"
)


#Opciones de estrés
stress_types=(
    "cpu"   
    "vm"    
    "io"    
    "hdd"   
)


for ((i=0; i<10; i++)); do
    container_name=$(generate_random_name)
    stress_type=${stress_types[RANDOM % ${#stress_types[@]}]}

    echo "Creating container: $container_name with stress type: $stress_type"

    case $stress_type in
    "cpu")
        sudo docker run -d --name "$container_name" containerstack/alpine-stress stress --cpu 2 #--timeout 30s
        #docker logs "$container_name"
        ;;
    "vm")
        sudo docker run -d --name "$container_name" containerstack/alpine-stress stress --vm 1 --vm-bytes 256M #--timeout 30s
        #docker logs "$container_name"
        ;;
    "io")
        sudo docker run -d --name "$container_name" containerstack/alpine-stress stress --io 4 #--timeout 30s
        #docker logs "$container_name"
        ;;
    "hdd")
        sudo docker run -d --name "$container_name" containerstack/alpine-stress stress --hdd 1 --hdd-bytes 512M #--timeout 30s
        #docker logs "$container_name"
        ;;
esac
done
