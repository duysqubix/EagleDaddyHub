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
curl https://get.docker.com | sh 

# install docker-compose
pip3 install docker-compose

# # # add new user
# useradd --groups dialout,docker,sudo -d /home/$USER -s /bin/bash -p $(echo eagledaddy | openssl passwd -1 -stdin) eagledaddy
# # # lock pi user, so we can't access account anymore
# passwd -l pi
# write "Remember to change password with 'passwd' for security reasons."

# add user to docker group
usermod -aG docker $USER

mkdir -p $EG_DIR
mkdir -p $EG_TMP
tar -xzvf eagledaddy.tar.gz -C / # moves to /tmp folder
mv $EG_TMP/* $EG_DIR
mv .env $EG_DIR
chown -R $USER $EG_DIR
echo "EG_DIR: $EG_DIR, EG_TMP: $EG_TMP, USER: $USER"


# reload 
# build and run docker as eagledaddy user
docker-compose -f $EG_DIR/docker-compose.yml --project-name eagledaddy_hub up --build --detach
exit 0


