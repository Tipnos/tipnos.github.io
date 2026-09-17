+++
title = 'Network multi-tenancy - 2. Leaf reachability'
date = 2026-08-07T10:31:01+02:00
tags = ['Network', 'EVPN', 'VXLAN', 'Multi-tenancy', 'Infrastructure', 'Cloud provider', 'Datacenter']
description = "Second post of a series that explains, from a software engineer's perspective, how network multi-tenancy can be achieved using EVPN/VXLAN."
draft = true
[params]
    enableComments = true
+++

Second post of a series that explains, from a software engineer's perspective, how network multi-tenancy can be achieved using EVPN/VXLAN. It is strongly advised to read the [intro]({{< ref "1-intro" >}}) first if not familiar with concepts such as clos topology and network multi-tenancy.

This one explains how switches connected to the servers and external networks communicate among each other. This feature enables every behavior described in the [protocols architecture overview]({{< ref "1-intro#protocols-architecture-overview" >}}) except the one that:

- Are 100% local to switches: [leaf 1]({{< ref "1-intro#leaf" >}})
- Are about fabric connectivity to other networks: [border leaf 2.4]({{< ref "1-intro#border-leaf" >}})

The design elements enabling this feature are:

1. The clos topology which enables leaves to reach each other through multiple paths and to scale the fabric relatively slowly and safely.
1. The use of the Border Gateway Protocol (BGP) to peer spines and leaves to provide a simple, scalable, and standards-based way to exchange routing information and build redundant end-to-end connectivity across the fabric.

Concerning BGP, it is widely used inside modern data centers that follow a Clos architecture. Its datacenter configuration is well known and standard. To not reinvent the wheel, BGP is used by the system in the standard way. However how the protocol works and is configured is critical, the rest of the section below explicits the configuration.

## Peer IP addresses

BGP requires to set-up bilateral IP addresses between peers. Initially static /32 ipv4 addresses were used. But since ipv6 release, static address assignment is not required anymore thanks to:

1. Link-local address
1. Router Advertisement protocol to learn the peer’s link-local address
1. RFC 5549 to announce an IPv4 address with the IPv6 link-local address as the next hop
1. Populating the routing table with RFC 5549 address announcement

## eBGP vs iBGP

BGP can be operated in two modes iBGP (internal) or eBGP (external). In datacenter eBGP offers multiple advantages:

1. **Scalability**: Supports fabrics ranging from a few switches to thousands of endpoints.
1. **Simplicity**: No additional protocols are required, BGP is self-sufficient.
1. **Fast convergence**: Link failures are quickly detected, and alternate paths are installed.
1. **Equal-Cost Multi-Path (ECMP)**: Traffic can be load-balanced across multiple spine switches, fully utilizing available bandwidth.
1. **Operational consistency**: The same routing protocol can be used within the data center and at its external boundaries.

Consequently every switches are configured to have one BGP instance handling multiple eBGP session, one per peering which is equal to one per inter-switch link.

## AS numbering scheme

Each BGP's instance must have an AS number configured. How AS number are attributed through the fabric is called the ASN numbering scheme. It greatly influences the behavior especially how the [best-path](#best-path) is computed by each switch.

The standard, to have the best properties for datacenters, is that:

1. All the switches uplinks are connected to switches with the same AS number which represents **a single routing domain**, simplifying configuration and enabling easy expansion.
1. For downlinks different AS are used to have usefull properties through the AS_PATH attribute like automatic loop detection or better control over route propagation.
1. Each switch must peer with a different AS number than itself to enable a [multiple eBGP session](#ebgp-vs-ibgp).

Applied to the clos topology, the numbering scheme can be computed this way:

- Each leaf gets its own ASN.
- All spines in a two-tier Clos get a single ASN. In a three-tier Clos, all spines within a pod get the same ASN, but the ASN is different for each pod.
- In a three-tier Clos, all super-spines get the same ASN.

Private ASNs are ASNs that are not visible in the global internet. Private ASNs come in both the two-byte and four-byte ASN variants. The two-byte ASNs support 1,023 private ASNs (64512–65534), whereas four-byte ASNs come with support for almost 95 million private ASNs (4200000000–4294967294), more than enough to satisfy a data center of any size in operation today.

> Note: At time of writing not every vendors do support four-byte ASNs well (eg. Asterfusion's SONiC switches). It is not an issue for a 2 tier clos but it can be for a 3 tier.

## Protocol overview

{{< figure src="images/bgp-state-machine.svg" alt="BGP finite state machine: Idle, Connect, Active, OpenSent, OpenConfirm and Established states grouped by TCP connection establishment, capability exchange, and route exchange phases" >}}

| Message type  | Use                                                                        | Periodicity                    |
| ------------- | -------------------------------------------------------------------------- | ------------------------------ |
| Open          | Sent on session establishment to identify router and exchange capabilities | Once                           |
| Update        | Used to exchange route advertisement and withdrawal                        | Only when information changes  |
| Keepalive     | Heartbeat, used to signal the remote peer that we’re alive and kicking     | Configured, usually 60 seconds |
| Notification  | Sent on error or when administratively closing the session                 | On error or close              |
| Route Refresh | Request remote peer toresend all the routes                                | Only as needed                 |

BGP can advertise how to reach not just IP addresses, but also other information such as MAC addresses. Each network protocol supported by BGP has its own identifier, called the Address Family Indicator (AFI). However, even within an AFI, there is a need for further distinctions. For example, unicast and multicast reachability information differ significantly. BGP distinguishes these cases by using separate Subsequent Address Family Indicator (SAFI) numbers for unicast and multicast addresses.
The AFI/SAFI list that is of interest to a BGP speaker is advertised using BGP capabilities in the BGP OPEN message. Two BGP peers exchange information about a network address only if both sides advertise an interest in its AFI/SAFI.

The workhorse BGP message is Update, which carries the list of advertised routes and the list of withdrawn routes. Update message contains different kind of attribute some mandatory and other optional depending on the protocol used (eg: EVPN).

For the IPV4 unicast AFI/SAFI, the message looks like the following:

| Component        | IPv4 UPDATE                                       |
| ---------------- | ------------------------------------------------- |
| Header           | Marker (16B), Length, Type=2                      |
| Withdrawn Routes | Optional IP prefixes (len + prefix)               |
| Path Attributes  | ORIGIN, AS_PATH, NEXT_HOP, MED; Total len=25B     |
| NLRI             | IPv4 prefixes, e.g., 1.1.1.1/32 (len=32 + prefix) |

Tram example:

```
Path Attributes: ORIGIN=IGP, AS_PATH=1, NEXT_HOP=192.168.12.1, MED=0
NLRI: 1.1.1.1/32
```

> Note: The NLRI is a top level field carrying prefixes for only the specific IPv4 unicast address family, it's a legacy. MP_REACH_NLRI is a path attribute sitting inside the path attributes section alongside ORIGIN, AS_PATH, etc. It was introduced specifically to let BGP carry reachability information for any other address family (IPv6 L2VPN/EVPN etc.).

## Best-path

A BGP instance computes the next-hop for each advertised routes via an algorithm called the **best-path**. It is computed when a new UPDATE message is received from one or more of its peers. BGP uses 8 metrics during the algorithm computation. Some of them are from the UPDATE message **Path Attributes** component, others are defined locally. The variables are:

| Mnemotic   | BGP metric         |
| ---------- | ------------------ |
| Wise       | Weight             |
| Lip        | LOCAL_PREFERENCE   |
| Lovers     | Locally originated |
| Apply      | AS_PATH            |
| Oral       | ORIGIN             |
| Medication | MED                |
| Every      | eBGP over iBGP     |
| Night      | Nexthop IGP Cost   |

In the data center, only two of these metrics are used: `locally originated` and `AS_PATH`. In other words, a prefix that is local to a node is preferred to one learned via BGP, and a shorter AS_PATH length route is preferred over a route with a longer AS_PATH length. If the AS_PATH lengths are equal, the paths are considered equal cost.

By default BGP implementation not only requires the AS_PATH lengths to be the same to be considered equal cost, but the individual ASNs in the AS_PATH must be identical. A specific knob (`bgp bestpath as-path multipath-relax` in FRR) must be turned on to relax this restriction and only uses the AS_PATH length in determining equal cost.

## Advertisement

To enable leaves reachability between each other through the clos topology, every switches must have its BGP routing table populated with one entry for each leaf. An entry requires:

- A unique IP address across the fabric
- Computed paths with the best-path algorithm

To achieve this:

- Each leaf switch must have assigned a unique IP address across the fabric
- This IP address must be advertised on the leaf [multiple eBGP session](#ebgp-vs-ibgp) to reach every connected spines acting as route reflector (RR) which will further re-advertise them to its connected leaves.

Thanks to the clos topology and the network configuration described previously: All leaves can reach each other via multiple paths and if one of them fails the system automatically redirect traffic to healthy remaining paths.

> Note: This leaf IP address is called a VTEP IP, the reason of this acronym is explained further.

## Convergence time

There are four timers that typically govern how fast BGP converges when either a failure occurs or when it is recovering from a failure:

- **Advertisement interval**: Waits the duration between sending successive updates to a peer. Default is 30 seconds for internet stability, in datacenters it is set to 0 seconds.
- **Keepalive** and **Hold timers**: If the remote peer doesn’t receive a Keepalive message for a duration that is equal to the Hold timers value it declares the peer dead and terminates the peering session. Default is Keepalive 60 seconds, in datacenters it is set to 3 seconds and Hold timers to 9.
- **Connect timer**: After failing to connect with a peer, waits for the Connect timer value before attempting to connect again. Default is 60 seconds, in datacenters it is set to 10 seconds.

In addition, BGP Bidirectional Forwarding Detection (BFD) is a lightweight protocol used to rapidly detect failures between two directly connected network devices. Instead of relying on Keepalive and Hold timers, BFD continuously exchanges small control packets and can detect link or neighbor failures in a few milliseconds. When it detects a failure, it immediately notifies BGP, which tears down the session and quickly reroutes traffic over alternate Equal-Cost Multi-Path (ECMP) links.

## Router id

BGP requires to have a 32-bit unique identifier assigned to a BGP speaker called Router ID. It identifies the router within the BGP control plane and is used for several protocol operations, but it is not used to forward data traffic.

The Router ID is usually configured as an IPv4 address that is matching the IP peer address. But when using IPv6 link-local address, this convention doesn't make sense anymore. Any arbitrary 32-bit identifier is compliant.

## Configuration

Below are BGP FRR configurations explained for both a spine and a leaf. Comments explain each command and link them with precedent explanations.

> Note: FRR (Free Range Routing) is an open-source, Linux-based routing protocol suite implementing BGP, OSPF, IS-IS, EVPN and more. It ships as the routing daemon on many datacenter switch operating systems (e.g. SONiC, Cumulus Linux), and its Cisco-like CLI is what the configuration snippets below use.

### Spine

```sh
# Inter-switch links
interface ethernet 1
    ipv6 use-link-local
    no ipv6 nd suppress-ra
    ipv6 nd ra-interval 5 # increase Router Advertisements frequency, allowing IPv6 hosts to detect gateway and network changes more quickly, thereby improving convergence in data center networks.
    exit

router bgp 65000
    no bgp ebgp-requires-policy # turn off in/outbound policy required by RFC 8212. This is a safety feature designed primarily for Internet-facing BGP, where accidentally leaking routes can have serious consequences.
    no bgp default ipv4-unicast # No automic advertisement of IPv4 routes to have 100% control on it
    bgp router-id 10.0.1.21 # Unique BGP speaker identifier
    bgp log-neighbor-changes # log BGP neighbor status change without impacting performance: important for monitoring and forensic analysis
    timers bgp 3 9 # cf. Convergence time section: Keepalive and Hold timers
    bgp bestpath as-path multipath-relax # cf. Best-path section.
    neighbor ISL peer-group # neighbors peer-group definition to simpilfy configuration
    neighbor ISL capability extended-nexthop # Advertise the support for RFC 5589: use of IPV6 RA for IP peering while advertising IPv4
    neighbor ISL bfd # cf. Convergence time section.
    neighbor ISL advertisement-interval 0 # cf. Convergence time section: Advertisement interval
    neighbor ISL remote-as external # cf. Best-path and ASN numbering scheme sections. ASNs are specified in the datacenter only for BGP's loop detection via AS_PATH. As each switch will connect to a different ASN we're in the case of an "external" connexion from a BGP perspective.
    neighbor ethernet 1 interface peer-group ISL # Apply peer-group ISL conf to the neighbor connected to ethernet 1 interface
    address-family ipv4 unicast # Specify the AFI/SAFI ipv4 unicast network protocol between peers along with its configuration below
        neighbor ISL activate # Activate ipv4 unicast AFI/SAFI for each interface of peer group ISL (neighbors)
        maximum-paths 8 # activate ECMP (Equal-Cost Multi-Path) to allow load-balancing instead of picking just one best path. Count depends on inter-switch links count in each tier of the clos fabric and hardware limitation.
    exit-address-family
    # as spines only re-advertise leaves VTEP IPv4, address-family ipv6 unicast is useless
exit
```

## Leaf

```sh
# Inter-switch links
interface ethernet 9
    ipv6 use-link-local
    no ipv6 nd suppress-ra
    ipv6 nd ra-interval 5
    exit

# VTEP IP
interface loopback 0 # Use loopback as the VTEP IP must be advertised through multiple interfaces.
    ip address 10.0.0.11/32
    exit

# Advertisement safety net, to avoid having leaves advertising unauthorized or unrequired addresses
route-map ADVERTISE_VTEP permit 10
    match interface Loopback0
exit

router bgp 65011
    no bgp ebgp-requires-policy
    no bgp default ipv4-unicast
    bgp router-id 10.0.1.11 # Set to the VTEP IP to ease observability
    bgp log-neighbor-changes
    timers bgp 3 9
    bgp bestpath as-path multipath-relax
    neighbor ISL peer-group
    neighbor ISL capability extended-nexthop
    neighbor ISL bfd
    neighbor ISL advertisement-interval 0
    neighbor ISL remote-as external
    neighbor ethernet 9 interface peer-group ISL
    address-family ipv4 unicast
        neighbor ISL activate
        redistribute connected route-map ADVERTISE_VTEP # apply safety net
        maximum-paths 8
    exit-address-family
    # as leaves only advertise its own VTEP IPv4, address-family ipv6 unicast is useless
exit
```

The next article is about leaf local tenants isolation:  how leaf switches manage local isolation between attached servers that belongs to different tenants.
