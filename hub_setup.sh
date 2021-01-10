#!/bin/bash

#
#
# Installs necessary packages and setups production ready EagleDaddy
#
#

HOME=/home/eagledaddy

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

cd $HOME

# update system
apt update && apt upgrade -y

# install git
apt install -y git libffi-dev libssl-dev python3-dev python3 python3-pip

# install docker
curl https://get.docker.com | sh 

# install docker-compose
pip3 install docker-compose

# # add new user
useradd --groups dialout,docker,sudo -d /home/eagledaddy -s /bin/bash -p $(echo eagledaddy | openssl passwd -1 -stdin) eagledaddy
# # lock pi user, so we can't access account anymore
passwd -l pi

write "Remember to change password with 'passwd' for security reasons."

git clone https://github.com/duysqubix/EagleDaddy.git; cd EagleDaddy

# build and run docker
docker-compose up -d --build

exit 0


