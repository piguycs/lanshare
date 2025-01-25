# LAN-Share VPN

**CLEANUP BRANCH**:
this branch is for basic cleanup, and more experimentation with other quic
implimentations

This is a simple VPN for playing LAN games with your peers across the internet.

The core vpn does work, but I have some serious performance issues to fix.

Right now, this vpn only "works" on Linux. It can technically be made to work
on MacOS and Windows too, but it wont use the platform native tools for those
platforms. This can potentially lead to a sub-par experience.

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

## Other technical stuff

I have tried to use channels as much as possible. These seem to be decently
fast on linux, but they also mean that I can easily cause deadlocks. Especially
due to the tun device constantly reading in a loop (my guess). I will be
cleaning up a lot of this code in the `cleanup` branch.

I am also going to try to use some principles from rocket-rs, where my types
should not be Clone/Copy by default. I might turn back on this "philosophy" if
it gets too difficult. I want to enjoy writing this at the end of the day.
