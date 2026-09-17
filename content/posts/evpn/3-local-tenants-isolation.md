+++
title = 'Network multi-tenancy - 3. Leaf local tenants isolation'
date = 2026-08-09T19:18:22+02:00
tags = ['Network', 'EVPN', 'VXLAN', 'BGP', 'VRF', 'VLAN', 'Multi-tenancy', 'Infrastructure', 'Cloud provider', 'Datacenter']
description = "Third post of a series on a deep, software-engineer-oriented exploration of how a modern datacenter network provides multi-tenancy, starting from the physical topology and progressively explaining the protocols and mechanisms that make the system work."
[params]
    enableComments = true
+++

Third post of a series on a deep, software-engineer-oriented exploration of how a modern datacenter network provides multi-tenancy, starting from the physical topology and progressively explaining the protocols and mechanisms that make the system work. It is strongly advised to read the [intro]({{< ref "1-intro" >}}) first if not familiar with concepts such as clos topology and network multi-tenancy.

This one explains how leaf switches manage local isolation between attached servers that belongs to different tenants. It is also the detailed explanation of the [leaf architecture overview]({{< ref "1-intro#leaf" >}}) behavior 1.

## VRF

The virtual routing and forwarding (VRF) is the network equipment's equivalent of Virtual Machine: a VRF is a logical isolated router. Most network operating system (NOS) like SONiC, Juniper or Cisco instantiates a "default" VRF and a "management" one (sometimes labeled "mgmt") when the equipment has a dedicated management ethernet port. Inter VRF communications is possible but not enabled by default with explicit configuration (as it's the case with two distinct network equipments). Consequently the VRF abstraction is the perfect fit on leaf switches to logically isolate each servers that belong to different tenant from each other. Binding servers to different VRFs provides the same routing isolation between them as if they were attached to separate routers.

The VRF abstraction is used by leaf switches according to the following rules:

1. The default VRF is used for [leaves reachability]({{< ref "2-leaf-reachability" >}}) BGP instance, inter-switch links ports and VTEP IP.
1. One VRF is instantiated with a **switch-wide** unique identifier for each distinct tenant whose servers are attached to the switch.

## VLAN

A Virtual LAN (VLAN) is a mechanism for partitioning a Layer 2 network into separate broadcast domains. Thorough VLAN is not mandatory to achieve tenant isolation when using VRF, it is required by some NOS when two port (or more) need to share the same L2 broadcast domain (same subnet and gateway IP) whenever they belong to the same leaf or are distributed through the fabric (EVPN l2vpn cf. [next article]({{< ref "4-distributed-multi-tenancy" >}})).

Consequently VLANs for local tenants isolation are configured with the following rules:

1. One VLAN MUST be instantiated with a **switch-wide** unique identifier for each leaf switch distinct tenant's subnet and bound to the proper VRF.
1. Every server's port MUST be bound to the proper VLAN accordingly to the tenant and subnet they belong.

> Note: As the vast majority of tenants have more than one server, provisioning VLAN even when not strictly required (only one server in a tenant) avoid service disruption when adding an other one later. It is the best trade-offs as VLANs introduce negligible overhead.

## Configuration

Below is the local isolation SONiC configuration of a leaf switch having the following tenants topology:

1. Blue has one server locally attached
1. Red has two servers on different subnets locally attached

```sh
vrf blue # unique vrf id in the context of the whole switch
    exit
vlan 10 # unique vlan id in the context of the whole switch
    exit
interface vlan 10
    vrf blue # bind the vlan to the blue vrf
    ip address 192.168.10.1/24 # ipv4 default gateway for ethernet interfaces bound to it
    nd ra prefix prefix 2a12:e0b:e67:10::/64 # ipv6 default gateway for ethernet interfaces bound to it
    nd ra autonomous on # required to enable ipv6 default gateway
    no ipv6 nd suppress-ra # required to enable ipv6 default gateway
    exit
interface ethernet 1
    switchport access vlan 10 # bind the ethernet port to the vlan 10 and indirectly to the blue vrf

vrf red
    exit
vlan 20
    exit
interface vlan 20
    vrf red
    ip address 192.168.20.1/24
    nd ra prefix prefix 2a12:e0b:e67:20::/64
    nd ra autonomous on
    no ipv6 nd suppress-ra
    exit
interface vlan 30
    vrf red
    ip address 192.168.30.1/24
    nd ra prefix prefix 2a12:e0b:e67:30::/64
    nd ra autonomous on
    no ipv6 nd suppress-ra
    exit
interface ethernet 2
    switchport access vlan 20
interface ethernet 3
    switchport access vlan 30
```

The [next article]({{< ref "4-distributed-multi-tenancy" >}}) is about distributed multi-tenancy: how servers belonging to the same tenant can be attached to different leaf switches in a clos topology and still being able to communicate among each other through the same L2 broadcast domain.
