I will just write down my mind in here, for future reference

- Hamachi uses 25.0.0.0/8 for their vpn. This is because this block is reserved
by the UK ministry of defence[^1], with devices that will never be open to the
internet. We can potentially do the same, as it gives us access to a lot of IPv4
addresses.
- Our alternatives are 10.0.0.0/8, 192.188.0.0/16 and 172.16.0.0/12. These are
*technically* what we should be using, as they are reserved for private ranges.
But this also means that there is a 1/3 chance that the user's private network
is using this range, and there could be clashes.

[^1]: [List of reserved /8 ranges](https://en.wikipedia.org/wiki/List_of_assigned_/8_IPv4_address_blocks#List_of_assigned_/8_blocks_to_the_regional_Internet_registries)

# naming convention

I am known to be more of a "write first, test later" kinda guy, but I want to
change it. I am also going to name modules containing unit tests as "unit_test"
and integration tests would be in their own folder.
