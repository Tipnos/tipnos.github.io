+++
title = 'Network multi-tenancy - 1. Intro'
date = 2026-08-01T21:12:37+02:00
tags = ['Network', 'EVPN', 'VXLAN', 'Multi-tenancy', 'Infrastructure', 'Cloud provider', 'Datacenter']
description = 'First post of a series that explains from a Software Engineer perspective how network multi-tenancy can be achieved using EVPN/VXLAN.'
[params]
    enableComments = true
+++

This is the first post of a series that explains from a Software Engineer perspective how network multi-tenancy can be achieved using EVPN/VXLAN. 

I had to self-train myself on the subject for unStack, a start-up I co-founded. I read many informative books like [`Cloud Native Data Center Networking`](https://www.oreilly.com/library/view/cloud-native-data/9781492045595/) by Dinesh G. Dutt. But, as these topics interest a restrict number of people, mostly composed of network engineers, deep technical books assume a way of reasoning about the technology which is counter intuitive. Especially because network protocols stack-up without any breaking change for decades. Consequently to achieve behaviors required by modern architecture a lots of them are used way differently from their initial purposes. Diving in its field as a newbie is somewhat close to learning a 40 years old legacy codebase that had never been refactored. 

As you can guess by now, the resulting system is overly complex compared to what an optimal solution could be. That's why companies that have the budget ends-up designing their own protocols and hardwares like [AWS with Nitro](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/ena-nitro-perf.html). 

It is assumed that the reader has an understanding of basic network concept taught in engineering schools like: 

- The OSI network model
- Mac/IP addresses
- The difference between L2/L3 aka bridging and routing.
- Details about how L2 and L3 works and interacts: ARP, default gateways, routing tables etc.

Context is now exposed let's start the technical stuff! An overview with the network topology most modern datacenter used: *the clos topology*.

# Clos topology

Named after Charles Clos, it is a type of non-blocking, multistage switching network architecture first described in 1953, designed to minimize the number of crosspoints while maintaining high connectivity. To keep things simple a 2-tier Clos topology is used in this series of articles. At time of writing datacenters used mostly 3 tiers and more for huge one (like cloud providers). 

Worth knowing, initially datacenters used an other topology called [fat tree](https://en.wikipedia.org/wiki/Fat_tree) optimised for bandwith and north-south bound network trafic. One of the chore reasons behind the change is that west-east trafic increased dramatically with the advent of micro-services architectures as an answer to scalability issues.

{{< figure src="images/clos-border-leaf-gateway.svg" alt="2-tier Clos fabric with spine/leaf tiers, border leaves dual-homed to border gateways connected to the internet" >}}

Topology network equipements functional scope divides in 4 categories:

1. **Leaf**: switches which the servers are attached to.
1. **Spine**: ensure redundant connectivity across every leaves.
1. **Border gateway (BGW)**: connect the datacenter to external networks (eg. internet).
1. **Border leaf**: does the bridge between the leaves (through the spines) and other internal or external networks (through the border gateways).

The important properties:

1. **Redundancy**: leaves and border leaves have multiple path to reach each other. Then if one of them is out of service, only a fraction of available bandwith is missing.
1. **Predictability**: the hops count (equipements count the traffic goes through) for any leaf reaching any other leaf or border leaf is constant across the entire network, providing predictable performance. It's no more true with a 3-tier, only one hop is added in certain pathes.
1. **Expandable**: the network grows naturally without requiring an upfront important investment or any later reconfiguration.

# Network multi-tenancy

Network multi-tenancy is a networking design where multiple independent customers, teams, or organizations (“tenants”) share the same physical network infrastructure while keeping their traffic, resources logically separated.

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

Something important to understand is that from the servers point of view they belong to their tenant's network. They send/receive packets exactly the same way. *They do not participate in the system that enables tenants logic seperation*. It is how each network equipement category used protocols that provides it as a **single coherent distributed system** often known as a **fabric**. That's why it is heavily used by bare-metal cloud providers, they can provide secure tenant isolation at network level without ever having to control/trust servers. In a way it can be seen as an equivalent of computers virtualization for networks infrastructure. Below is a list of the main problems that such a system must solve:

1. Servers' tenants must be able to reach other networks (internet, shared services) or expose to them their public IP addresses.  
1. A tenant's servers set must be able to span any combination of leaves ports (distributed multi-tenancy). Consequently, the system must be able to:
    1. Simulate L2 networks that spans multiple leaves.
    1. Enforce isolation at leaf level between attached servers that belongs to different tenants and forward traffic when belonging to the same.
    1. Forward tenants leaf<->leaf and leaf<->border leaf trafic through the right path according to tenants configuration. 
1. Leverage the 3 clos topology important properties: **Redundancy**, **Predictability** and **Expandable**.

At this point the reader should have enough understanding of both the system's topology and the main requirements it must met. The next section aims to describe at a high level how such a distributed system behaves along the network protocols involved at each step.

# Protocols architecture overview

## Leaf

Manage local L2 tenants servers and distributed multi-tenancy:

1. Isolate local L2 tenants and act as the default gateway for them. _Protocols_: **VRF, VLAN**
1. Have a unique IP address through the fabric called VTEP IP.
1. _Control plane traffic_:
   1. Advertise its VTEP IP to the connected spines. _Protocol_: **BGP**
   1. Learn other leaves VTEP IPs. _Protocol_: **BGP**
   1. Broadcast its local tenants L2 and L3 reachability information to other leaves to tell them what is reachable through it. _Protocols_: **BGP, EVPN**
   1. Learn tenants L2 and L3 reachability information from other leaves. Cache them if local configuration requires it. _Protocols_: **VRF, BGP, EVPN**
1. _Data plane traffic_:
   1. **Attached servers outbound**: Look-up attached servers packets and compute destination which is either local tenants servers or other leaves. If it is other leaves, encapsulate it with its own VTEP IP as the source and computed leaves VTEP IP as the destination. _Protocols_: **BGP, VXLAN**.
   1. **Attached servers inbound**: For local originating traffic simply forward it, otherwise decapsulate packets and compute local destination. _Protocol_: **VRF, VLAN, VXLAN**

## Border leaf

Connect the fabric to external networks (internet, inter-az, shared services):

1. Have a unique IP address through the fabric called VTEP IP.
1. _Control plane traffic_:
   1. Advertise its VTEP IP to the connected spines. _Protocol_: **BGP**
   1. Learn other leaves VTEP IPs. _Protocol_: **BGP**
   1. Learn tenants L2 and L3 reachability information from other leaves. Cache them if local configuration requires it. _Protocols_: **VRF, BGP, EVPN**
   1. Advertise a subset of the fabric "public" routes for scalability reason to connected gateways. _Protocol_: **BGP**
   1. Advertise external networks routes learnt from the gateways to its subset of the fabric's tenants. _Protocols_: **BGP, EVPN**
1. _Data plane traffic_:
   1. **Fabric outbound**: Decapsulate packets from other leaves, perform a route look-up and forward it at the proper gateway. _Protocols_: **BGP, VXLAN, VRF**
   1. **Fabric inbound**: Compute destination leaf by performing a route look-up and encapsulate the packets accordingly. _Protocols_: **BGP, VXLAN, VRF**

## Spine

Act as a route reflector (RR):

1. _Control plane traffic_:
   1. Learn and readvertise connected leaves VTEP IP to provide redundant reachability betweem them. _Protocol_: **BGP**
   1. Learn and readvertise tenants L2 and L3 reachability information from connected leaves. _Protocols_: **BGP, EVPN**
1. _Data plane traffic_: Forward leaves encapsulated traffic to destination leaves. _Protocol_: **BGP**

End of the introduction! Enough concepts have been covered to understand each remaining articles of the serie. Each one of them deep dive on one or a groupment of steps enumerated in the current secion. The next one is about leaves reachability: how switches connected to the servers (leaves) and to external networks (border leaves) communicate among each other.
