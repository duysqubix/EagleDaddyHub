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
    
    apt --autoremove purge -y ifupdown dhcpcd5 isc-dhcp-client isc-dhcp-common rsyslog
    apt-mark hold ifupdown dhcpcd5 isc-dhcp-client isc-dhcp-common rsyslog raspberrypi-net-mods openresolv
    rm -r /etc/network /etc/dhcp

    apt --autoremove purge -y avahi-daemon
    apt-mark hold avahi-daemon libnss-mdns
    apt install -y libnss-resolve
    ln -sf /run/systemd/resolve/stub-resolv.conf /etc/resolv.conf
    systemctl enable systemd-networkd.service systemd-resolved.service

    # configure wlan0
    cat > /etc/wpa_supplicant/wpa_supplicant-wlan0.conf <<EOF
country=US
ctrl_interface=DIR=/var/run/wpa_supplicant GROUP=netdev
update_config=1

network={
    ssid="EagleDaddyHub"
    mode=2
    key_mgmt=WPA-PSK
    psk="eagledaddy"
    frequency=2412
}
EOF
    chmod 600 /etc/wpa_supplicant/wpa_supplicant-wlan0.conf
    systemctl disable wpa_supplicant.service
    systemctl enable wpa_supplicant@wlan0.service
    rfkill unblock 0


    # configure wlan1

    # rename wpa_supplicant.conf to wpa_supplicant-wlan1.conf
    mv /etc/wpa_supplicant/wpa_supplicant.conf /etc/wpa_supplicant/wpa_supplicant-wlan1.conf
    chmod 600 /etc/wpa_supplicant/wpa_supplicant-wlan1.conf
    systemctl disable wpa_supplicant.service
    systemctl enable wpa_supplicant@wlan1.service
    rfkill unblock 2

    #configure interfaces
    cat > /etc/systemd/network/08-wlan0.network <<EOF
[Match]
Name=wlan0
[Network]
Address=192.168.4.1/24
# IPMasquerade is doing NAT
IPMasquerade=yes
IPForward=yes
DHCPServer=yes
[DHCPServer]
DNS=84.200.69.80 1.1.1.1
EOF

    cat > /etc/systemd/network/12-wlan1.network <<EOF
[Match]
Name=wlan1
[Network]
DHCP=yes
EOF
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

# copy over prod config.yml to docker
docker cp config.yml eagledaddy:/home/config.yml

# settings for production:
docker exec eagledaddy cat config.yml

# activate shutdown_signal script as service
chmod +x wait_for_shutdown_signal.sh
cp shutdown_signal_eagledaddy.service /etc/systemd/system/
systemctl daemon-reload
systemctl enable shutdown_signal_eagledaddy.service
systemctl start shutdown_signal_eagledaddy.service
systemctl status shutdown_signal_eagledaddy.service
write "Please reboot for changes to take effect"
# reboot now
exit 0


