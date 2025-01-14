# Config files for LAN-Share

LAN-Share requires the following to work as intended. Note that these files are
highly configurable, allowing sysadmins to control access to the LAN-Share
daemon effectively, by utilising systems that are already part of most distros.

## Prerequisites

LAN-Share currently relies on D-Bus. LAN-Share also officially only supports
Systemd, but support tickets from users of alternative systems will not be
disregarded. Any contributions to make these dependencies less mandatory are
always welcome.

## Configs

1. Systemd Unit: Used to start the LAN-Share daemon with access to managing TUN
devices
2. D-Bus Policy: Controls access to the LAN-Share daemon. Default config allows
the root user to start the LAN-Share daemon, and all other users can connect to
it. This can be modified to, for example, only allow users in certain groups to
start/access the daemon over D-Bus
