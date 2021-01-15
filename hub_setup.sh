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
    apt install -y hostapd dnsmasq 
    # DEBIAN_FRONTEND=noninteractive apt install -y netfilter-persistent iptables-persistent

    systemctl unmask hostapd.service
    systemctl enable hostapd.service
    #### configure hostapd

    # tell dhcp to set static ip address
        cat << EOF >> /etc/dhcpcd.conf
interface wlan0
    static ip_address=192.168.4.1/24
    nohook wpa_supplicant
EOF

    #### configure dnsmasq
    write "configuring dns server"
    mv /etc/dnsmasq.conf /etc/dnsmasq.conf.bak

    # 
    cat << EOF >> /etc/dnsmasq.conf
interface=wlan0
dhcp-range=192.168.4.2,192.168.4.20,255.255.255.0,24h
domain=eagledaddy
address=/hub.eagledaddy/192.168.4.1
EOF
    # enable 5Ghz
    write "enabling 5Ghz"
    rfkill unblock wlan

    # configure hostapd.conf
    cat << EOF >> /etc/hostapd/hostapd.conf
country_code=US
interface=wlan0
ssid=EagleDaddyHub
hw_mode=g
channel=7
auth_algs=1
macaddr_acl=0
ignore_broadcast_ssid=0
wpa=2
wpa_key_mgmt=WPA-PSK
wpa_passphrase=eagledaddy
wpa_pairwise=TKIP
rsn_pairwise=CCMP
EOF

    # updated DAEMON_CONF in default hostapd so it knows where to find
    # configuration files
    sed -i "/DAEMON_CONF=/c\DAEMON_CONF=\"/etc/hostapd/hostapd.conf\"" /etc/default/hostapd

}


write "Checking root priviledges.."
if [ "$EUID" -ne 0 ]
  then write "Please run as root" 0
  exit
fi


# update system
apt update 

# install necessary packages
apt install -y libffi-dev libssl-dev python3-dev python3 python3-pip

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
init_ap


### allow port 80 exposed using ufw
iptables -A INPUT -p tcp --dport 80 --jump ACCEPT # to manually open web port
iptables-save
write "initializing ports... `sleep 5`" # wait for ufw to take affect

# # copy over eagledaddysrc code to WORKDIR
# mkdir -p $EG_DIR
# mkdir -p $EG_TMP
# tar -xzvf eagledaddy.tar.gz -C / # moves to /tmp folder
# mv $EG_TMP/* $EG_DIR
# chown -R $USER $EG_DIR

# build and run docker as eagledaddy user
# docker-compose -f $EG_DIR/docker-compose.yml --project-name eagledaddy_hub build
docker-compose pull && docker-compose up -d
# reboot now
exit 0


