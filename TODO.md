- D-Bus signals for
    1. Sending errors
    2. Sending updates to the state

We will be using signals for the above 2, because the daemon will be mostly
asynchronous. Once client sends an event, the client's job is done. The daemon
will do whatever with the event, and emit any state changes or errors. Then the
client will do whatever it wants with that info. Neither the daemon or the 
client is dependent on each other to complete their task.

# distribution
These are the ways that I envision the distribution, once a stable-ish beta is
released. Main focus will ofc be linux distros, since MacOS and Windows have
relatively few means of distribution.

Linux:
(-> means putting it on official repos. this usually requires a "sponsor", who
acts as a mentor)
1. AUR -> arch official repos
2. Fedora .rpm -> RPM fusion -> Official repos (I dont know how this works)
3. Ubuntu -> PPA -> Official repos (I dont know how this works)
4. NixOS self-hosted repo -> NixOS Unstable -> NixOS Stable (next release)
5. Just a tarball with instructions on how to install it properly

MacOS:
1. Homebrew
2. Custom .dmg image with the required files

Windows:
1. Choco
2. WinGet
3. Custom .msi installer
