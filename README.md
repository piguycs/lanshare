# LAN-Share VPN

This is a simple VPN for playing LAN games with your peers across the internet.

The core vpn does work, but I have some serious performance issues to fix.

Right now, this vpn only "works" on Linux. It can technically be made to work
on MacOS and Windows too, but it wont use the platform native tools for those
platforms and needs third party libraries during runtime. Checkout docs for
github.com/meh/rust-tun

## Technical info (Linux)

**RELAY-SERVER**: Makes use of QUIC (via s2n-quic) to establish connections with
peers. The main reason for QUIC was to have encrypted connections over UDP when
establishing bi-directional streams between peer and relay for transmitting 
ipv4 packets, but it is also used to communicate with the peers for auth.

**LS-DAEMON**: A D-Bus daemon that keeps hold of the TUN device on Linux. This
needs to be run as root, and runs on the system bus. A simple policy file for
this daemon is located in `config/dbus/me.piguy.lanshare.conf`.

**LS-CLIENT**: Communicates with the D-Bus daemon to login and turn on/off the
VPN. This does not need to run as root, and the D-Bus policy file can be
modified to control which users/groups can communicate with the daemon.
Currently a CLI, would use Iced or cosmic in the future.

## Technical info (MacOs)

TBD: I am probably going to use "XPC". Daemon will be written in Swift, and
I will have to export my rust binary as a dynamic library to be called via the
Swift version. Client will use SwiftUI.


## Technical info (Windows)

TBD: A lot of options, probably going to use RPC. There are other options, but
I will have to do my research and see. Probably going to be C#, but I dont mind
C++ as long as there are any real advantages over C# for this. Again, core would
be my rust daemon exported as a dynamic library. I think WinUI3 is the most
modern toolkit for windows? idk... my university forced me to use winforms and
called it a semester.

## Current state of the project

It "works" as of commit `6d8fa7fc7534d7d699fb7da309d61010963e4957`. I suspect
something is wrong with the device_task, which I am attempting to rewrite. Any
help would be appreciated, honestly I am at the end of my wits here. There might
be some sort of a deadlock going on, but it frees up occasionally letting very
few packets through. This is an absolute tragedy, but I learnt a lot!
