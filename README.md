## NightOwl: Navigation Compiler 

This project aims to solve the simple question: what's the most efficient way to get from my home to work across PATH, CitiBike, Hudson-Bergen Light Rail, and walking.

## System Architecture

This diagram visualizes how raw transit schedules and live streams are compiled into an optimized, in-memory graph.

```mermaid
graph LR
    subgraph External ["External Sources"]
        PATH["PATH Rail (GTFS)"]
        CB["CitiBike (GBFS)"]
        HBLR["Light Rail (GTFS)"]
    end

    subgraph Backend ["Backend (Rust)"]
        ING["Ingestion & Compiler"]
        REDIS[("Redis Cache")]
        GRAPH["In-Memory Graph"]
        ROUTE["Dijkstra Router"]
        
        ING <--> REDIS
        ING --> GRAPH
        ROUTE <--> GRAPH
    end

    subgraph Client ["Client (Next.js)"]
        UI["UI & Map Display"]
    end

    PATH & CB & HBLR --> ING
    UI <--> ROUTE
```

## Project Roadmap

Here is the step-by-step implementation plan for **NightOwl**:

### Stage 1: Core Engine & Spatial Primitives
- [x] Set up Rust workspace and Dijkstra pathfinding primitives
- [x] Implement Manhattan grid distance ($L_1$ norm) and urban walking pace (2.5 mph)
- [x] Formulate initial time-dependent search states
```
Edge: Newport PATH <-> Grove St PATH (0.6711 miles)
Edge: Grove St PATH <-> Exchange Place PATH (0.5788 miles)
Edge: Newport PATH <-> Exchange Place PATH (0.7366 miles)

SUCCESS: Shortest path distance from Newport PATH to Exchange Place PATH is 0.7366 miles!

--- Testing Edge Weight At Specific Departure Times ---
Arriving at 8:00 AM (480 mins): 1.53 minutes total cost
Arriving at 8:10 AM (490 mins): 21.53 minutes total cost
Arriving at 9:00 AM (540 mins): inf minutes total cost
-------------------------------------------------------
```

### Stage 2: PATH Rail & Multi-Modal Routing Engine
- [x] Parse PATH GTFS static feeds (`stops.txt`, `stop_times.txt`)
- [x] Construct self-contained `Location` architecture (`Station` vs `Point`)
- [x] Implement multi-modal Dijkstra search: `Walk` $\rightarrow$ `Transit` $\rightarrow$ `Walk`
- [x] Verify coordinate-to-coordinate routing output against real-world Google Maps queries
```
============================================================
  ROUTE: Point (40.7300, -74.0346) ➔ Point (40.7406, -73.9858)
  Departure:      10:22 EST
  Total Duration: 31 mins
============================================================
Leg 1: [🚶 Walk (0.21 mi, 6 mins)]
   Start:   Point (40.7300, -74.0346)      @ 10:22 EST
   End:     Newport                        @ 10:28 EST
------------------------------------------------------------
Leg 2: [🚆 Transit - Trip ID: t_6004238_b_none_tn_2]
   Get On:  Newport                        @ 10:28 EST
   Get Off: Christopher Street             @ 10:36 EST
------------------------------------------------------------
Leg 3: [🚆 Transit - Trip ID: t_6004238_b_none_tn_2]
   Get On:  Christopher Street             @ 10:36 EST
   Get Off: 9th Street                     @ 10:37 EST
------------------------------------------------------------
Leg 4: [🚆 Transit - Trip ID: t_6004238_b_none_tn_2]
   Get On:  9th Street                     @ 10:37 EST
   Get Off: 14th Street                    @ 10:39 EST
------------------------------------------------------------
Leg 5: [🚆 Transit - Trip ID: t_6004238_b_none_tn_2]
   Get On:  14th Street                    @ 10:39 EST
   Get Off: 23rd Street                    @ 10:40 EST
------------------------------------------------------------
Leg 6: [🚶 Walk (0.52 mi, 13 mins)]
   Start:   23rd Street                    @ 10:40 EST
   End:     Point (40.7406, -73.9858)      @ 10:53 EST
============================================================
```

### Stage 3: Micro-Mobility & Real-Time Streams (CitiBike GBFS & Delays)
- [ ] Ingest CitiBike GBFS live feeds (real-time dock availability & bike locations)
- [ ] Add `Leg::Bike` modality with dock pick-up/drop-off constraints
- [ ] Incorporate live delay feeds to adjust active transit edges dynamically

### Stage 4: Light Rail Expansion (Hudson-Bergen Light Rail)
- [ ] Ingest Hudson-Bergen Light Rail (HBLR) GTFS static schedules
- [ ] Support PATH $\leftrightarrow$ Light Rail transfer stations (Exchange Place, Newport, Hoboken)
- [ ] Incorporate transfer penalty buffers for switching transit lines

### Stage 5: Web UI & Delivery Pipeline (Next.js & Map Visualizer)
- [ ] Build high-performance Rust web API endpoint (`GET /route?origin=...&destination=...`)
- [ ] Create Next.js interactive map frontend with Leaflet.js / Mapbox
- [ ] Render multi-modal route step polylines on the map
