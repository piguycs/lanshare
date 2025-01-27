# LAN-Share VPN

This is a simple VPN for playing LAN games with your peers across the internet.

The vpn does work, but I have some serious performance issues to fix.
(~15-30ms delay with the current MVP, goal is to be sub 0.5ms)

Right now, this vpn only "works" on Linux. It can technically be made to work
on MacOS and Windows too, but it wont use the platform native tools for those
platforms and needs third party libraries during runtime. Checkout docs for
github.com/meh/rust-tun. I do plan on fully supporting other operating systems.

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

The vpn works, and can be used on linux machines. The procedure for testing it
is very manual for now, but I am working on an automatic testing suite based on
top of testcontainers, but manual testing is required for now. My main focus
when writing the code for this was to learn and to figure things out. Now that
I have done that, I will be making this more "production ready".

Here is the (currently manual) process:
run `just run-server` on a designated server node.
**edit `ls-daemon/src/main.rs` to point to the IP of this server.**

Run `just run-daemon-root` in 2 different client nodes. One of these can host
the server too.

Now we need to setup a dbus policy, OR you can try running ls-client as root.
Move ./config/dbus/me.piguy.lanshare.conf to the appropriate location on your
system. For me, it is `/usr/share/dbus-1/system.d/me.piguy.lanshare.conf`.

Finally, run `just run-client` on your client nodes. This starts a repl, a gui
based on iced-rs is being worked on right now. You need to run:
```
> name ANY_UNIQUE_NAME_HERE
> upgrade
> up
```
Now, you should get an IP assigned to your node. You can view it by running
`ip a`. This can be pinged from any other node on your device.
