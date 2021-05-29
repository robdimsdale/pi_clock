# pi_clock
Show the time and weather using a Raspberry Pi and an LCD display

Usage:

## Build and run locally

```sh
cargo run -- --uri='http://some-cache.local'
```

## Build and deploy to remote sever

```sh
TARGET_HOST=10.0.1.10 ./deploy
```

## Raspberry Pi config

Some useful configuration for Raspberry Pis

### Setting up SSH and Wifi

* Create an empty file at `/boot/ssh` to enable SSH.
* Add Wifi details to `/boot/wpa_supplicant.conf` as follows (Raspbian Stretch and later):

```
ctrl_interface=DIR=/var/run/wpa_supplicant GROUP=netdev
network={
    ssid="YOUR_SSID"
    psk="YOUR_WIFI_PASSWORD"
    key_mgmt=WPA-PSK
}
```

The Pi Zero W only supports 2.4GHz; the Pi 4B also supports 5GHz.

### Enabling Pulse-width Modulation (PWM)

PWM is used for the brightness of the LCD and TFT displays.

* Add the line `dtoverlay=pwm-2chan` to the file `/boot/config.txt` and reboot.

### Enabling programs to run on startup

* Add the command (with a trailing `&` to `/etc/rc.local`) and restart.
