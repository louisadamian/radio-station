#!/bin/bash
deps="nginx"
CHOICES=$(whiptail --title "Install Radio Station Options" \
  --checklist "Choose something" 10 40 4 \
  "ADS-B" "Airplane tracking" OFF \
  "GOES" "NOAA Satellite Weather" OFF \
   3>&1 1>&2 2>&3)
exitstatus=$?
if [ $exitstatus != 0 ]; then
    exit 1
fi
if [[  -z "${#CHOICES[@]}" ]]; then
  echo "no options were selected please select at least 1 system by navigating to it with the arrow keys and selecting using the space bar"
  exit 1
fi
ADSB=false
GOES=false
mkdir -p "$HOME/dev" && cd "$HOME/dev" || exit
for CHOICE in $CHOICES; do
    case "$CHOICE" in
    "ADS-B")
        ADSB=true
        wget https://www.flightaware.com/adsb/piaware/files/packages/pool/piaware/f/flightaware-apt-repository/flightaware-apt-repository_1.2_all.deb
        sudo dpkg -i flightaware-apt-repository_1.2_all.deb
        deps="${deps} dump1090-fa dump978-fa"
    ;;
    "GOES")
        GOES=true
        deps="${deps} git build-essential cmake libopencv-dev libproj-dev zlib1g-dev"
        (git clone https://github.com/pietern/goestools --recursive && cd goestools) || (echo  failed to download goestools; exit 1)
        mkdir -p build && cd build || exit
        cmake ../ -DCMAKE_INSTALL_PREFIX=/usr/local
#        if [ -z "$ls ./src/goesdec"] || [-z`` "$ls ./src/goesproc"]; then
#            echo "failed to install goestools"
#            exit 1
#        fi
        make -j
        sudo make install
    ;;
    esac
done
sudo apt update
sudo apt install "$deps" -y
if $ADSB; then
    sudo systemctl start dump1090-fa.service
    sudo systemctl start dump978-fa.service
    sudo bash -c "$(wget -nv -O - https://github.com/wiedehopf/tar1090/raw/master/install.sh)"
fi
if $GOES; then
    echo "configuring goes-tools"

fi
