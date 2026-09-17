+++
title = 'Network multi-tenancy - 4. Distributed multi-tenancy'
date = 2026-08-10T15:52:46+02:00
tags = ['Network', 'EVPN', 'VXLAN', 'BGP', 'VRF', 'VLAN', 'Multi-tenancy', 'Infrastructure', 'Cloud provider', 'Datacenter']
description = "Fourth post of a series on a deep, software-engineer-oriented exploration of how a modern datacenter network provides multi-tenancy, starting from the physical topology and progressively explaining the protocols and mechanisms that make the system work."
draft = true
[params]
    enableComments = true
+++

Fourth post of a series on a deep, software-engineer-oriented exploration of how a modern datacenter network provides multi-tenancy, starting from the physical topology and progressively explaining the protocols and mechanisms that make the system work. It is strongly advised to read the [intro]({{< ref "1-intro" >}}) first if not familiar with concepts such as clos topology and network multi-tenancy.

This one explains how servers belonging to the same tenant can be attached to different leaf switches in a clos topology and still being able to communicate among each other through the same L2 broadcast domain. This system's feature enables [leaf architecture overview]({{< ref "1-intro#leaf" >}}) control-plane behavior number 3.3, 3.4 and data-plane 4.1, 4.2.

The design elements enabling this feature are:

1. EVPN for control plane: a BGP address family that enables leaf switches to advertise, to one another, the L2 (MAC) and L3 (IP) reachability information of their locally attached servers.
2. VXLAN for data plane: the encapsulation protocol that enables leaf switches to exchange packets on behalf of their attached servers.

## Control plane: EVPN

As stated earlier EVPN is a BGP address family, an instance is then required and the [leaves reachability]({{< ref "2-leaf-reachability" >}}) one is the perfect fit as it already connect all the spines and leaves. It means than this instance will be used to both advertise VTEP IP and EVPN information.

The additional configuration to activate the EVPN address family in BGP is straightforward. But a deep knowledge of the underlying implication is required to enable proper observability of the fabric.

### Tenant identifier

As explained in the previous [Leaf local tenants isolation]({{< ref "3-local-tenants-isolation" >}}) article, on each leaf switch a tenant is represented by a VRF and by as many VLANs as it has distinct L2 broadcast domains. A tenant distributed across multiple leaf switches can have different VRF identifiers on each switch due to local constraints, and the same applies to VLANs. Because the control plane needs to identify tenants and their subnets consistently, these heterogeneous, per-leaf local identifiers cannot be used by EVPN.

To address this, each tenant's VRF and VLANs must additionally be assigned an identifier that is unique across the whole fabric, configured on every leaf switch alongside their local one. This allows leaves to cache locally only the reachability information of the tenants they're interested in, and to forward tenants' data-plane packets to the correct leaves.

Since the data plane already requires a 24-bit identifier — the VNI (VXLAN Network Identifier) — for VRFs and VLANs, the control plane reuses this same VNI as its fabric-unique identifier, rather than introducing a separate one.

To sum up, the VNI assignment rules across the fabric are:

1. EVPN identifies tenants with VNIs. Tenants' VRFs on leaf switches must have their VNIs set up accordingly.
1. EVPN identifies tenants' distinct L2 broadcast domains with VNIs. Tenants' VLANs on leaf switches must have their VNIs set up accordingly.

### Route types

As stated in the previous [BGP protocol overview]({{< ref "2-leaf-reachability#protocol-overview" >}}) section non-IPv4 unicast routes are advertised via the MP_REACH_NLRI. For most AFI/SAFI combinations, the structure and content of the reachability information carried in an UPDATE message is the same across that AFI/SAFI. This is not the case with EVPN. For example, the update can be reachability to a specific MAC address, or it could be reachability to an entire virtual network. Route type identifies what kind of EVPN information a MP_REACH_NLRI is carrying:

| Route Type | What it carries                 | Primary use                                                                                       |
| ---------- | ------------------------------- | ------------------------------------------------------------------------------------------------- |
| RT-1       | Ethernet Segment Auto Discovery | Supports multihomed endpoints in the data center, used instead of MLAG                            |
| RT-2       | MAC, VNI, IP                    | Advertises reachability to a specific MAC address in a virtual network, and its IP address        |
| RT-3       | VNI/VTEP Association            | Advertises a VTEP’s interest in virtual networks                                                  |
| RT-4       | Designated Forwarder            | Ensures that only a single VTEP forwards multidestination frames to multihomed endpoints          |
| RT-5       | IP prefix, VRF                  | Advertises IP prefixes, such as summarized routes, and the VRF associated with the prefix         |
| RT-6       | Multicast group membership      | Contains information about which multicast groups an endpoint attached to a VTEP is interested in |

#### Broadcast Unicast Multicast (BUM) handling

EVPN route types RT-3 can carry a BGP attribute called Provider Multicast Service Interface (PMSI), which identifies the kind of BUM packet handling supported by this device. The PMSI attribute is defined in a completely different standard (RFC 6514) from the usual EVPN standards.

To handle BUM traffic there is two options:

- Head-end replication: the ingress VTEP itself make a separate unicast copy of the BUM packet for each remote VTEP in that VNI's flood list.
- Routed multicast: the ingress VTEP maps each VNI to a multicast group in BGP and relies on PIM to replicate the packet efficiently at each fanout point in the network.

Routed multicast is more performant because replication happens once, at the actual branching points of the tree, rather than entirely at the ingress. But it is a really complicated protocol that tends to appear mainly in larger-scale or bandwidth-sensitive designs where BUM volume genuinely justifies the added PIM complexity. Head-end replication scale effectively if BUM traffic is a fairly low percentage of the trafic. Hopefully kubernetes CNI like Cilium is optimized to use BUM traffic as few as possible which should represents a huge part of the traffic.

#### Routing

Routing happens in EVPN when communication is required either between subnets within the same tenant, or between different tenants. At the time of writing, this type of communication doesn't occur: each tenant is currently configured with only a single subnet, and inter-tenant communication only happens via border leaves for external connections. However, as the product evolves — for example, to support two clusters within the same tenant — inter-subnet communication within a single tenant would become active.

The EVPN standard is to use _distributed routing_ over _centralized_, and _symmetric routing_ over _asymmetric_, and this is well suited to the system's requirements.

The only configuration implication is for distributed routing: since every leaf must be able to route locally for any subnet it hosts, each leaf carrying a given subnet must be configured with the same anycast gateway IP/MAC for that subnet. In FRR distributed routing happens naturally with this configuration because each leaf independently becomes capable of routing for its own directly-connected tenants/subnets.

For Symmetric routing with FRR, it is configured once a VRF is associated to an L3VNI. The L2VNI-to-L3VNI (VRF) association is based on whether an SVI for a VLAN is enslaved to a VRF that itself has an L3VNI configured.

### Route distinguisher

EVPN allows network information to overlap between tenants (e.g., IP addresses). But as spines (typically acting as route reflectors) aggregate routes from all leaves, their BGP tables could end up with structurally identical NLRIs advertised by different tenants. Since BGP has no native way to distinguish these as separate routes, it would treat them as duplicates of the same prefix, running best-path selection between them and discarding all but one — silently losing one tenant's reachability information.

To solve this issue, EVPN uses a **Route Distinguisher (RD)** to make these otherwise identical routes unique within the BGP routing table. An RD is an eight-byte value included in every *EVPN NLRI*. It makes otherwise identical routes from different tenants or route origins distinguishable within the BGP table. RDs can be computed in different ways depending on the fabric configuration. In our case (EVPN/VXLAN), RDs are computed as follows:

1. Type field (2 bytes) set to Type 1: Administrator subfield = 4-byte IPv4 address, Assigned Number subfield = 2 bytes
1. Administrator: VTEP IP of the leaf advertising the route
1. Assigned Number: a value tied to the VRF or VLAN's VNI

A VNI is 24 bits (3 bytes) long, which doesn't fit directly into the RD's 2-byte Assigned Number field. For example FRR provides a feature to auto-generate the Assigned Number using an internal sequential index assigned to each VNI in local learning order — meaning the same VNI can receive a different index (and therefore a different RD) on different leaves. The issue with relying on this is observability: performing a lookup in the BGP route table does not allow identifying which entry maps to which VNI (i.e., tenant). For example, it makes assessing whether a configuration transaction was successfully applied difficult, since doing so would require knowing every leaf switch's internal VNI index mapping. Consequently, when automation is in place, the preferred solution is to avoid relying on auto-generated RDs and instead have a central authority maintain the VNI → Assigned Number mapping table across the whole fabric.

### Route target

EVPN also needs to control which leaves import which tenants' routes into their local BGP/VRF tables. A given leaf may host only a subset of a fabric's tenants, so it should only need to import and process routes relevant to those tenants — not every route the fabric carries. The Route Distinguisher can't help with this: as we've seen, its sole job is making otherwise-identical routes unique in the BGP table, so multiple tenants' routes don't collide (and can be advertised at all) — it says nothing about who should receive and use a given route.

To solve this, EVPN uses a **Route Target (RT)**, an 8-byte **extended community** attribute of the UPDATE message attached to every tenant-advertised route, that acts as a membership tag: it marks which VRF (tenant) or bridge domain (subnet) a route belongs to, independent of which leaf originated it. Unlike the RD, the RT is not required to be unique per-route — routes belonging to the same tenant/subnet across different leaves are expected to carry the same RT, since it's the very mechanism that lets leaves group them together.
Each leaf is configured with:

- an export RT: attached to routes it originates for a given VRF/VNI, announcing "this route belongs to this tenant/subnet."
- an import RT (or set of RTs): a filter applied to received routes, accepting only those whose RT matches one of the leaf's configured import RTs, and installing the matching ones into the appropriate local VRF/bridge domain.

In our case (EVPN/VXLAN), RTs are typically auto-derived as:

1. Administrator: the local AS number (2 bytes)
1. Assigned Number: the tenant's VRF or VLAN's VNI (4 bytes)

Because the Assigned Number here uses the full VNI directly (not an internal index like the RD), RTs don't suffer from the RD's observability problem: looking at a route's RT immediately tells you which VNI/tenant it belongs to. But in fabrics using eBGP for the EVPN's BGP instance, auto-derived export RT still uses the local leaf's own AS (<local-AS>:VNI), so the RT value itself differs per originating leaf for the same VNI. To keep import filtering correct in this case, FRR treats the auto-derived import RT as *:VNI — wildcarding the AS portion and matching on the VNI alone — rather than requiring an exact RT match. This restores correct tenant-scoped filtering, but means RT values are not fabric-consistent for observability purposes in an eBGP-overlay design; verification tooling should match on the VNI portion only, not the full RT string.

### Encapsulation

EVPN was designed as a control plane decoupled from any specific data-plane encapsulation and the MP_REACH_NLRI says nothing about how to actually reach the advertised next-hop at the data-plane level. The Encapsulation Extended Community solves this by attaching the tunnel type (e.g., VXLAN, MPLS, NVGRE, GRE) as a separate, optional attribute alongside the route — reusing the existing, already-transitive Extended Communities mechanism. As this extended community attribute is stored alongsite the route, it keeps the control plane encapsulation-agnostic and even allows heterogeneous fabrics to interoperate over the same BGP sessions.

### Example

Below is an eBGP UPDATE message example for Leaf1's hostA route Type 2, VNI 10010, tenant RED and how it propagates through the fabric with the EVPN control plane. The goal is to provide to the reader what is happening in BGP instances to be able to use them in observability tools.

#### Leaf1 → Spine1: full UPDATE message

| Field                       | Bytes | Value                       |
| --------------------------- | ----- | --------------------------- |
| Marker                      | 16    | `FF...FF` (all 1s, no auth) |
| Length                      | 2     | 106 (total message length)  |
| Type                        | 1     | `02` (UPDATE)               |
| Withdrawn Routes Length     | 2     | `00 00` (none)              |
| Withdrawn Routes            | 0     | —                           |
| Total Path Attribute Length | 2     | `00 53` (83 bytes)          |

**ORIGIN**

```
Flags: 40 (well-known, transitive)   Type: 01   Length: 01   Value: 00 (IGP)
```

**AS_PATH**

```
Flags: 40   Type: 02   Length: 06
Value: 02 01 00 00 FD E9
       │  │  └─────┬────┘
       │  │        AS 65001 (4-byte ASN)
       │  └─ segment length = 1
       └──── segment type = 2 (AS_SEQUENCE)
```

**EXTENDED_COMMUNITIES**

```
Flags: C0 (optional, transitive)   Type: 10 (16)   Length: 10 (16 bytes = 2 communities)

  Community 1 — Route Target:
  00 02 FD E9 00 00 27 1A
  │  │  └─┬─┘ └───┬──────┘
  │  │ AS 65001  Local Admin = 10010 (VNI)
  │  └─ sub-type 0x02 (Route Target)
  └──── type 0x00 (2-byte AS specific)

  Community 2 — Encapsulation:
  03 0C 00 00 00 00 00 08
  │  │                 └─ Tunnel Type = 8 (VXLAN)
  │  └─ sub-type 0x0c
  └──── type 0x03 (Opaque Extended Community)
```

**MP_REACH_NLRI** (optional non-transitive — carries next-hop + the actual EVPN NLRI)

```
Flags: 80 (optional, non-transitive)   Type: 0E (14)   Length: 30 (48 bytes)

  AFI:            00 19        (25 = L2VPN)
  SAFI:           46           (70 = EVPN)
  NH Length:      04
  Next Hop:       0A 00 00 01  (10.0.0.1 — Leaf1's VTEP)
  Reserved:       00

  NLRI (Type 2 route, 39 bytes):
    Route Type:   02
    Length:       25           (37 bytes)
    RD:           00 01 0A 00 00 01 27 1A     (Type1: 10.0.0.1:10010)
    ESI:          00 00 00 00 00 00 00 00 00 00
    Eth Tag ID:   00 00 00 00
    MAC Len:      30           (48 bits)
    MAC:          00 11 11 11 11 01
    IP Len:       20           (32 bits)
    IP:           0A 0A 0A 0B  (10.10.10.11)
    MPLS Label1:  00 27 1A     (VNI 10010)
```

#### Tables state after this UPDATE

**Leaf1 — Adj-RIB-out toward Spine1**

```
NLRI: [2]:[10.0.0.1:10010]:[0]:[48]:[0011.1111.1101]:[32]:[10.10.10.11]
Next-hop: 10.0.0.1   AS-Path: 65001   RT: 65001:10010   Encap: VXLAN
```

**Spine1 — Adj-RIB-in (from Leaf1)**

Identical to what Leaf1 sent — spine hasn't modified anything yet:

```
NLRI: [2]:[10.0.0.1:10010]:...:[10.10.10.11]
Next-hop: 10.0.0.1   AS-Path: 65001   RT: 65001:10010   Encap: VXLAN
```

**Spine1 — Loc-RIB**

Same route, now the "active"/selected path (no competing path exists, RD makes it unique):

```
* NLRI: [2]:[10.0.0.1:10010]:...:[10.10.10.11]
  Next-hop: 10.0.0.1   AS-Path: 65001   RT: 65001:10010
```

**Spine1 → Leaf2: re-advertisement UPDATE**

This is a **new** UPDATE message the spine constructs, with two changes from what it received:

```
AS_PATH:   02 02 00 00 FD E8 00 00 FD E9
           │  │  └───┬──────┘└──┬───────┘
           │  │    AS 65000   AS 65001     ← spine's AS prepended
           │  └─ segment length = 2
           └──── segment type = 2

Next-Hop:  0A 00 00 01 (10.0.0.1 — UNCHANGED, preserved only because
                         "neighbor Leaf2 next-hop-unchanged" is configured
                         on the spine, without it, this would become the
                         spine's own IP, breaking VXLAN decap on Leaf2)

Ext-Communities: RT 65001:10010, Encap VXLAN — PRESERVED only because
                  "neighbor Leaf2 send-community extended" is configured,
                  without it, the spine strips both communities on
                  re-advertisement across this eBGP hop, and Leaf2 would
                  receive the NLRI with no RT at all — making it
                  unimportable into any VRF.
```

**Leaf2 — Adj-RIB-in (from Spine1)**

```
NLRI: [2]:[10.0.0.1:10010]:...:[10.10.10.11]
Next-hop: 10.0.0.1   AS-Path: 65000 65001   RT: 65001:10010   Encap: VXLAN
```

**Leaf2 — Loc-RIB (after import policy evaluation)**

Leaf2's auto-derived import RT is `*:10010` (eBGP wildcard, per FRR's behavior). The RT `65001:10010` matches on VNI regardless of AS → **installed**:

```
* NLRI: [2]:[10.0.0.1:10010]:...:[10.10.10.11]
  Next-hop: 10.0.0.1 → installed into VLAN10 EVPN-MAC-table
  AS-Path: 65000 65001   RT: 65001:10010 (matched via *:10010 wildcard)
```

This entry now drives the data plane: Leaf2 knows hostA (`00:11:11:11:11:01` / `10.10.10.11`) sits behind VTEP `10.0.0.1`, and will encapsulate traffic toward it using VXLAN, VNI 10010, destination underlay IP `10.0.0.1` — all derived from the information carried by this BGP UPDATE, together with the existing reachability to the VTEP.

#### Summary of what changed at each hop

|                | RD                                 | RT/Encap                                                        | Next-hop                                                   | AS_PATH                       |
| -------------- | ---------------------------------- | --------------------------------------------------------------- | ---------------------------------------------------------- | ----------------------------- |
| Leaf1 → Spine1 | fixed, set at origination          | present                                                         | 10.0.0.1 (origin)                                          | `65001`                       |
| Spine1 → Leaf2 | **unchanged** (still part of NLRI) | **unchanged**, but only if `send-community extended` configured | **unchanged**, but only if `next-hop-unchanged` configured | **prepended** → `65000 65001` |

### Configuration

The example is a follow up of the [leaves reachability]({{< ref "2-leaf-reachability#configuration" >}}) and [leaf local tenants isolation]({{< ref "3-local-tenants-isolation#configuration" >}}) configurations.

#### Leaves reachability

**Spine**

```sh
# same configuration

router bgp 65000
    # same configuration
    address-family l2vpn evpn # Specify the AFI/SAFI l2vpn evpn network protocol between peers
        neighbor ISL activate # Activate l2vpn evpn AFI/SAFI for each interface of peer group ISL (neighbors) to accept and broadcast incoming EVPN route types
    exit-address-family
exit
```

**Leaf**

```sh
# same configuration

router bgp 65011
    # same configuration
    neighbor ISL activate
        advertise-all-vni # tells the BGP process to automatically discover and advertise every locally configured VNI (both L2 and L3) into EVPN. Avoid having to manually define per-VNI route targets or route distinguishers in the BGP config itself.
        advertise-svi-ip # activate symetric routing by advertising IP address of the local SVI (the switch's own IRB/anycast gateway interface for that VLAN/VNI) as a Type 2 (MAC/IP) route — i.e., the leaf advertises its own gateway MAC+IP pair, not just IPs it learns from attached hosts.
        # rd 10.10.10.1:100 # Overwrite audo-derived route distinguisher generated by advertise-all-vni. Usefull for custom route distinguishers configuration generated by an orchestrator as mentionned in the previsous Route distinguisher dedicated section
    exit-address-family
exit
```

#### Leaf local tenants isolation

The example below is the follow up of [leaf local tenants isolation]({{< ref "3-local-tenants-isolation#configuration" >}}) blue tenant configuration that extends its L2 broadcast domain to an other leaf.

**Leaf1** (configuration added to previous example)

```sh
vrf blue
    # same configuration
    vni 100 # tenant fabric identifier
    exit
vlan 10
    # same configuration
    vni 10 # tenant's subnet fabric identifier
    exit
    # same configuration
    exit
```

**Leaf2** (new leaf)

```sh
vrf blue # unique vrf id in the context of the whole switch
    vni 100 # tenant fabric identifier
    exit
vlan 11 # unique vlan id in the context of the whole switch
    vni 10 # tenant's subnet fabric identifier
    exit
interface vlan 11
    vrf blue # bind the vlan to the blue vrf
    ip address 192.168.10.1/24 # same ipv4 default gateway because of symetric routing
    nd ra prefix prefix 2a12:e0b:e67:10::/64 # ipv6 default gateway for ethernet interfaces bound to it
    nd ra autonomous on # required to enable ipv6 default gateway
    no ipv6 nd suppress-ra # required to enable ipv6 default gateway
    exit
interface ethernet 1
    switchport access vlan 11 # bind the ethernet port to the vlan 10 and indirectly to the blue vrf
```
