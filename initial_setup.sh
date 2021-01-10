#!/bin/bash

#
#
# Flashes and does preliminary setup procedures so that upon first
# power up of new OS, it will produce a production ready EagleDaddyHub
#
#

export SDA=/dev/sda
export BOOT=/dev/sda1
export ROOT=/dev/sda2
export ROOT_MNT=/mnt/root
export BOOT_MNT=/mnt/boot 
export OS_IMG_FILE_NAME=2020-12-02-raspios-buster-armhf-lite.img

export OS_IMG=/etc/raspberry-os/$OS_IMG_FILE_NAME

export NETWORK=wlan0
export STATIC_IP=192.168.1.100
export ROUTER_IP=192.168.1.1
export DOMAIN_SERVER=192.168.1.32

export RED='\033[0;31m'
export GREEN='\033[0;32m'
export NC='\033[0m' # No Color

cleanup(){
    write "removing files.."
    rm \
        wpa_supplicant.conf

    umount $BOOT
    umount $ROOT
    write "unmounting..."

}

write_wpa_file(){
    cat << EOF > wpa_supplicant.conf
    country=US
    ctrl_interface=DIR=/var/run/wpa_supplicant GROUP=netdev
    update_config=1

    network={
    ssid="${WIFI_SSID}"
    psk="${WIFI_PASSWD}"
    }
EOF
}


install_etcher_cli(){
    curl -sL https://deb.nodesource.com/setup_12.x | sudo -E bash -
    apt-get install -y nodejs && npm install -g etcher-cli --unsafe-perm
}

write(){

    if [ -z $2 ]
    then
        echo -e "[${GREEN}+${NC}]" $1
    else
        echo -e "[${RED}-${NC}]" $1
    fi
}

write "Checking root priviledges.."
if [ "$EUID" -ne 0 ]
  then write "Please run as root" 0
  exit
fi

write "Checking if WIFI variables are set"
if [ -z $WIFI_SSID ] || [ -z $WIFI_PASSWD ]
then
    write "Must set WIFI_SSID and WIFI_PASSWD to continue" 0
    exit -1
fi

# write wifi file so that new hub can connect to local wifi for setup
write "writing wpa_supplicant.conf"; write_wpa_file

# check if proper IMG file exists on host machine
if ! [ -f $OS_IMG ]
then
    write "could not find IMG file" 0
    exit -1
fi

# check if etcher-cli is properly installed
if ! command -v etcher &> /dev/null
then
    write "etcher-cli is not installed\nattempting automatic install..." 0
    install_etcher_cli
    exit -1
fi

# check to see if host can detect SD Card
if  [ -z $SDA ]
then
    write "Could not detect SD card in USB, remove and try again..." 0
    exit -1
fi

# now actually flash the sd card using etcher
write "flashing.. [$OS_IMG_FILE_NAME -> $SDA]"
etcher -d $SDA -y $OS_IMG
write "flashing....done"


# mount partitions
if ! [ -d $ROOT_MNT ]
then
    write "$ROOT_MNT does not exist...creating" 0
    mkdir -p $ROOT_MNT
    write "$ROOT_MNT created.."
fi

if ! [ -d $BOOT_MNT ]
then
    write "$BOOT_MNT does not exist...creating" 0
    mkdir -p $BOOT_MNT
    write "$BOOT_MNT created.."
fi


write "`mount -v $BOOT $BOOT_MNT`"
write "`mount -v $ROOT $ROOT_MNT`"

#### BOOT ######################

# transfer config.txt file over to boot
write "setting configuration: `cp -v config.txt $BOOT_MNT/`"

# transfer wpa_supplicant file over to boot
write "enabling WIFI: `cp -v wpa_supplicant.conf $BOOT_MNT/`"

# transfer ssh file over to boot (to enable ssh)
write "enabling ssh: `cp -v ssh $BOOT_MNT/`"



#### ROOT ######################
write "enabling static IP, $STATIC_IP,  `cp -v dhcpcd.conf $ROOT_MNT/etc/`"
write "transfering setup script `cp -v hub_setup.sh $ROOT_MNT/home/`"


# cleanup things
cleanup
write "done"
exit 0