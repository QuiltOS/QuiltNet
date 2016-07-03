QuiltNet
================================================================================

A network stack in pure Rust designed from the ground up for both asynchronous and synchronous usage.

Libraries
--------------------------------------------------------------------------------

```
/
├── cyclic_order       -- Generic Code used for keeping track of out-of-order
│                         packets in TCP. Should be stable enough to move out of
│                         tree.
├── data_link
│   ├── interface      -- Presents the Interface link-layer drivers should
│   │                     implement to work with the Network Layer.
│   └── udp_mock       -- A mock link-layer driver built on UDP. (Requires
│                         libstd.)
├── misc               -- Some random crap used by everything else.
├── network            -- Currently Just IPv4. Should contain a interface, and
│                         IPv4 and Ipv6 implementations.
└── transport
    ├── brown_rip      -- A modified/simplified RIP, implemented on top of IPv4
    │                     instead of UDP.
    ├── static_routing -- A dummy routing package that learns no routes -- You
    │                     can only talk to immediate neighbors.
    └── tcp            -- Currently incomplete.
```
