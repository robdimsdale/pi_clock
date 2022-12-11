# pi_clock
Show the time and weather using a Raspberry Pi and an LCD display

Usage:

## Build and run locally

```sh
cargo run -- --uri='http://some-cache.local'
```

## Build and deploy to remote sever

There is a `deploy` script provided to facilitate cross-compilation and deployment.

It requires [`cross`](https://github.com/cross-rs/cross) to be installed

```sh
./deploy --target 10.0.1.10 --release
```

## Raspberry Pi config

Some useful configuration for Raspberry Pis:

### Setting up SSH and Wifi

The official Raspberry Pi Imager tool supports initial config like hostname, wifi config, enabling SSH, etc.

The 64-bit lite OS is a good option: modern hardware (e.g. Pi 3, Pi 4, Pi Zero
2) supports 64-bit instructions and there is no need to install the full desktop OS.

The Pi Zero W only supports 2.4GHz; the Pi 4B also supports 5GHz.

### Enabling Pulse-width Modulation (PWM)

PWM is used for the brightness of the LCD and TFT displays.

* Add the line `dtoverlay=pwm-2chan` to the file `/boot/config.txt` and reboot.

### Enabling I2C

I2C is used to connect to the VEML7700 light sensor.

* Enable it via the `raspi-config` utility.

### Enabling programs to run on startup

* Add the command (with a trailing `&` to `/etc/rc.local`) and restart.


### GPIO permissions

On the first run, we often see GPIO permissions issues. Subsequent executions seem to be fine.
