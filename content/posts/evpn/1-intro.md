+++
title = 'Network multi-tenancy - 1. Intro'
date = 2026-08-01T21:12:37+02:00
tags = ['Network', 'EVPN', 'VXLAN', 'Multi-tenancy', 'Infrastructure', 'Cloud provider', 'Datacenter']
description = "First post of a series that explains, from a software engineer's perspective, how network multi-tenancy can be achieved using EVPN/VXLAN."
draft = true
[params]
    enableComments = true
+++

This is the first post of a series that explains, from a software engineer's perspective, how network multi-tenancy can be achieved using EVPN/VXLAN.

I had to train myself on the subject for unStack, a start-up I co-founded. I read many informative books like [`Cloud Native Data Center Networking`](https://www.oreilly.com/library/view/cloud-native-data/9781492045595/) by Dinesh G. Dutt. But, as these topics interest a restricted number of people, mostly composed of network engineers, deep technical books assume a way of reasoning about the technology which is quite different from software engineer's mental model. Moreover Network architectures tend to evolve by adding new protocols and extensions while preserving compatibility with existing infrastructure. As a result, modern datacenter networks combine technologies from different generations to achieve requirements that weren't necessarily part of their original design. 

It is assumed that the reader has an understanding of basic network concepts taught in engineering schools like:

- The OSI network model
- Mac/IP addresses
- The difference between L2/L3 aka bridging and routing.
- Details about how L2 and L3 work and interact: ARP, default gateways, routing tables etc.

Now that the context is set, let's start with the technical stuff! An overview of the network topology most modern datacenters use: *the Clos topology*.

## Clos topology

Named after Charles Clos, it is a type of non-blocking, multistage switching network architecture first described in 1953, designed to minimize the number of crosspoints while maintaining high connectivity. For simplicity, this series uses a two-tier leaf-spine Clos. Larger datacenter networks can add additional tiers to scale the fabric further.

Worth knowing, initially datacenters used an other topology called [hierarchical tree design (core/aggregation/access)](https://en.wikipedia.org/wiki/Data_center_network_architectures) optimised for bandwith and north-south bound network trafic. One of the chore reasons behind the change is that west-east trafic increased dramatically with the advent of micro-services architectures as an answer to scalability issues.

{{< figure src="images/clos-border-leaf-gateway.svg" alt="2-tier Clos fabric with spine/leaf tiers, border leaves dual-homed to border gateways connected to the internet" >}}

The network equipment in this topology divides into 4 functional categories:

1. **Leaf**: switches which the servers are attached to.
1. **Spine**: ensures redundant connectivity across all leaves.
1. **Border gateway (BGW)**: connect the datacenter to external networks (eg. internet).
1. **Border leaf**: bridges the leaves (through the spines) and other internal or external networks (through the border gateways).

>Note: Terminology varies between vendors and architectures. In this series, "border leaf" refers to a leaf participating in the fabric that connects to a border gateway, while "border gateway" refers to the device connecting the fabric to an external network.

The important properties:

1. **Redundancy**: leaves and border leaves have multiple paths to reach each other. So if one of them is out of service, only a fraction of the available bandwidth is missing.
1. **Predictability**: the hop count (the number of devices the traffic passes through) for any leaf reaching any other leaf or border leaf is constant across the entire network, providing predictable path length and simplifying capacity planning. It's no longer true with a 3-tier topology: some paths (that cross between pods) gain two extra hops.
1. **Expandability**: capacity can be increased incrementally by adding leaves and, when necessary, spines without redesigning the entire topology.

## Network multi-tenancy

Network multi-tenancy is a networking design where multiple independent customers, teams, or organizations ("tenants") share the same physical network infrastructure while keeping their traffic and resources logically separated.

```
                 Shared physical network
                         │
             ┌───────────┼───────────┐
             │           │           │
          Tenant A    Tenant B    Tenant C
             │           │           │
          Network      Network      Network
          10.0.0.0     10.0.0.0     192.168.0.0
```

Something important to understand is that from the servers' point of view they simply belong to their tenant's network. They send/receive packets exactly the same way regardless of the underlying fabric. *They do not participate in the system that enables tenant logical separation*. This is achieved by having each network equipment category use protocols that together provide it as a **single coherent distributed system**. If you're a software engineer, you can think of the fabric as a distributed system whose nodes are network devices and whose state is propagated through control-plane protocols.

That's why it is heavily used by bare-metal cloud providers: it lets them provide secure tenant isolation at the network level without ever having to control or trust the servers themselves. In a way, it can be seen as the network infrastructure equivalent of compute virtualization. Below is a list of the main problems that such a system must solve:

1. Tenants' servers must be able to reach other networks (internet, shared services) or expose their public IP addresses to them.  
1. A tenant's set of servers must be able to span any combination of leaf ports (distributed multi-tenancy). Consequently, the system must be able to:
    1. Simulate L2 networks that span multiple leaves.
    1. Enforce isolation at the leaf level between attached servers that belong to different tenants, and forward traffic between servers belonging to the same tenant.
    1. Forward tenants' leaf<->leaf and leaf<->border leaf traffic through the right path according to each tenant's configuration. 
1. Leverage the Clos topology's 3 important properties: **Redundancy**, **Predictability** and **Expandability**.

At this point the reader should have enough understanding of both the system's topology and the main requirements it must meet. The next section describes, at a high level, how such a distributed system behaves, along with the network protocols involved at each step.


## Protocols architecture overview

Quick definitions of the protocols mentioned below:

- **VRF** (Virtual Routing and Forwarding): a virtual, isolated routing table. Several VRFs can coexist on the same physical device, each with its own routes.
- **VLAN** (Virtual Local Area Network): splits a single physical L2 network into multiple isolated broadcast domains. Frames are tagged with a VLAN ID so switches know which domain they belong to.
- **BGP** (Border Gateway Protocol): a routing protocol used to exchange reachability information between devices.
- **EVPN** (Ethernet VPN): a BGP extension that advertises L2 (MAC) and L3 (IP) tenant reachability as routes instead of relying on flood-and-learn. It acts as the control plane.
- **VXLAN** (Virtual Extensible LAN): a data plane protocol that encapsulates L2 frames over an IP network, letting a single L2 segment span multiple leaves. It relies on a control plane, here EVPN, to know where to send that encapsulated traffic.

### Leaf

Manage local L2 tenants servers and distributed multi-tenancy:

1. Isolate local L2 tenants and act as the default gateway for them. _Protocols_: **VRF, VLAN**
1. Have a unique IP address, known throughout the fabric, called the VTEP IP (VXLAN Tunnel Endpoint).
1. _Control plane traffic_:
   1. Advertise its VTEP IP to the connected spines. _Protocol_: **BGP**
   1. Learn other leaves' VTEP IPs. _Protocol_: **BGP**
   1. Broadcast its local tenants' L2 and L3 reachability information to other leaves, to tell them what is reachable through it. _Protocols_: **BGP, EVPN**
   1. Learn tenants' L2 and L3 reachability information from other leaves. Cache it if local configuration requires it. _Protocols_: **VRF, BGP, EVPN**
1. _Data plane traffic_:
   1. **Attached servers outbound**: Look up attached servers' packets and compute the destination, which is either a local tenant's server or another leaf. If it is another leaf, encapsulate the packet with its own VTEP IP as the source and the computed leaf's VTEP IP as the destination. _Protocols_: **VXLAN**.
   1. **Attached servers inbound**: For locally originating traffic, simply forward it; otherwise, decapsulate the packet and compute the local destination. _Protocol_: **VRF, VLAN, VXLAN**

### Border leaf

Connect the fabric to external networks (internet, inter-az, shared services):

1. Have a unique IP address, known throughout the fabric, called the VTEP IP.
1. _Control plane traffic_:
   1. Advertise its VTEP IP to the connected spines. _Protocol_: **BGP**
   1. Learn other leaves' VTEP IPs. _Protocol_: **BGP**
   1. Learn tenants' L2 and L3 reachability information from other leaves. Cache it if local configuration requires it. _Protocols_: **VRF, BGP, EVPN**
   1. Advertise a subset of the fabric's "public" routes to connected gateways, for scalability reasons. _Protocol_: **BGP**
   1. Advertise external network routes learnt from the gateways to its subset of the fabric's tenants. _Protocols_: **BGP, EVPN**
1. _Data plane traffic_:
   1. **Fabric outbound**: Decapsulate packets from other leaves, perform a route look-up, and forward them to the proper gateway. _Protocols_: **VXLAN, VRF**
   1. **Fabric inbound**: Compute the destination leaf by performing a route look-up, and encapsulate the packets accordingly. _Protocols_: **VXLAN, VRF**

### Spine

Act as a route reflector (RR):

1. _Control plane traffic_:
   1. Learn and readvertise connected leaves' VTEP IPs to provide redundant reachability between them. _Protocol_: **BGP**
   1. Learn and readvertise tenants' L2 and L3 reachability information from connected leaves. _Protocols_: **BGP, EVPN**
1. _Data plane traffic_: Forward leaves' encapsulated traffic to destination leaves. _Protocol_: **VXLAN**

Also, two terms come up constantly in this space and are worth introducing now: the **underlay**, the plain IP network that gives every switch reachability to every other switch (no tenant awareness at all), and the **overlay**, the tenant-aware layer built on top of it that carries the actual L2/L3 reachability information and encapsulated tenant traffic. VXLAN defines how tenant traffic is encapsulated over the underlay. It does not define how switches discover where that traffic should be sent. EVPN provides this control plane by distributing tenant reachability information using BGP.

End of the introduction! Enough concepts have been covered to understand each of the remaining articles of the series. Each one of them deep dives on one or a grouping of steps enumerated in the current section. The [next one]({{< ref "2-leaf-reachability" >}}) is about leaf reachability: how switches connected to the servers (leaves) and to external networks (border leaves) communicate with each other.
