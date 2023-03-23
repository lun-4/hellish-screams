# hellish-screams

Scream protocol for PulseAudio systems.

Problem statement: pulseaudio-rtp sucks lol. playback is massively spotty and
audio gets pitch shifted for some accursed reason, also latency grows so
hard after a while i have to `pulseaudio -k` on **BOTH SIDES**.

i dont want this

i dont want this at all

however, in my dualbooting escapades i have found Scream: https://github.com/duncanthrax/scream

it works really well when my secondary dualboot machine runs windows,
but not for linux, as there's only a windows driver

so i make do:

```sh
env ADDR=192.168.2.1:6970 make prod
```

while the receiver side still keeps using the Unix receiver from the Scream repo:

```sh
./scream -v -u -i 100.123.32.83 -p 6970
```

this is *NOT* multicast. i will not implement it. i dont care

thx @cyanreg for pointing me towards the PulseAudio Simple API
