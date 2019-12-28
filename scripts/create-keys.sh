#!/usr/bin/env sh

if [ ! -f .ssh/id_rsa ]; 
then
	mkdir -p ~/.ssh
	ssh-keygen -b 4096 -t rsa -f ~/.ssh/id_rsa -q -N ""
	echo "Please add this public key where you need access!"
	echo ""
	cat ~/.ssh/id_rsa.pub
fi
