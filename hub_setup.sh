#!/bin/bash

#
#
# Installs necessary packages and setups production ready EagleDaddy
#
#
USER=pi
HOME=/home/$USER
EG_TMP=/tmp/eagledaddy
EG_DIR=$HOME/eagledaddy

write(){

    if [ -z $2 ]
    then
        echo -e "[${GREEN}+${NC}]" $1
    else
        echo -e "[${RED}-${NC}]" $1
    fi
}


init_ap(){

    #### configure hostapd

    # tell dhcp to ignore wlan0 interface, we will set ip address elsewhere
    echo "denyinterfaces wlan0" >> /etc/dhcpcd.conf 

    # set static IP address for WiFi interface
    cat << EOF >> /etc/network/interfaces
    auto lo
    iface lo net loopback

    auto eth0
    iface eth0 inet dhcp

    allow-hotplug wlan0
    iface wlan0 inet static
        address 192.168.5.1
        netmask 255.255.255.0
        network 192.168.5.0
        broadcast 192.168.5.255
EOF


    # configure hostapd.conf
    cat << EOF >> /etc/hostapd/hostapd.conf
    interface=wlan0
    driver=n180211
    ssid=EagleDaddyHub
    hw_mode=g
    channel=10
    ieee80211n=1
    wmm_enabled=1
    ht_capab=[HT40][SHORT-GI-20][DSSS_CCK-40]
    macaddr_acl=0
    ignore_broadcast_ssid=0
    wpa=2
    wpa_key_mgmt=WPA-PSK
    wpa_passphrase=eagledaddy
    rsn_pairwise=CCMP
EOF

    # updated DAEMON_CONF in default hostapd so it knows where to find
    # configuration files
    sed -i "/DAEMON_CONF=/c\DAEMON_CONF=\"/etc/hostapd/hostapd.conf\"" /etc/default/hostapd

    #### configure dnsmasq
    write "configuring dns server"
    mv /etc/dnsmasq.conf /etc/dnsmasq.conf.bak

    # 
    cat << EOF >> /etc/dnsmasq.conf
    interface=wlan0
    listen-address=192.168.5.1
    bind-interfaces
    server=8.8.8.8
    domain-needed
    bogus-priv
    local=/eagledaddy/
    domain=eagledaddy
    dhcp-range=192.168.5.100,192.168.5.200,24h
EOF

    # add static domain name for this device
    # so we can address AP as hub.eagledaddy etc...
    echo "192.168.5.1 hub.eagledaddy hub" >> /etc/hosts
}


write "Checking root priviledges.."
if [ "$EUID" -ne 0 ]
  then write "Please run as root" 0
  exit
fi


# update system
apt update 

# install necessary packages
apt install -y libffi-dev libssl-dev python3-dev python3 python3-pip hostapd dnsmasq ufw

# install docker
if ! command -v docker &> /dev/null
then
    write 'docker not detected, installing it now...'
    curl https://get.docker.com | sh
fi
 

# install docker-compose
pip3 install docker-compose

# # # add new user
# useradd --groups dialout,docker,sudo -d /home/$USER -s /bin/bash -p $(echo eagledaddy | openssl passwd -1 -stdin) eagledaddy
# # # lock pi user, so we can't access account anymore
# passwd -l pi
# write "Remember to change password with 'passwd' for security reasons."

# add user to docker group
usermod -aG docker $USER



#### set up device as an Access Point using hostapd/dnsmasq
write 'Configuring device as an Access Point'
# init_ap # DO NOT RUN YET, UNTIL YOU CAN ACCESS WEB SITE FROM LOCALNETWORK FIRST

### allow port 80 exposed using ufw
ufw allow 80
ufw allow 8000
ufw enable
write "initializing ports... `sleep 5`" # wait for ufw to take affect

# copy over eagledaddysrc code to WORKDIR
mkdir -p $EG_DIR
mkdir -p $EG_TMP
tar -xzvf eagledaddy.tar.gz -C / # moves to /tmp folder
mv $EG_TMP/* $EG_DIR
cp .env $EG_DIR
chown -R $USER $EG_DIR

# build and run docker as eagledaddy user
docker-compose -f $EG_DIR/docker-compose.yml --project-name eagledaddy_hub up --detach
exit 0


