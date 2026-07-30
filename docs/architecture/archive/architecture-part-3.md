# Mindstrata — World Expansion, PESTLE Development, and Cellular Decay Architectural Plan

**Implementation order:** This plan is intended to begin **after** the human-scale biological, psychological, relational, and cultural/noospheric upgrades are in place.  
**Architectural role:** Systems-architect plan for expanding Mindstrata from a single settlement simulation into a multi-scale, interdependent world simulation.

---

## 1. Mandate

The next expansion must transform Mindstrata from:

> a deeply simulated settlement of embodied minds

into:

> a multi-scale civilization simulation where biology, decay, environment, economy, politics, law, science, religion, military conflict, infrastructure, culture, and noospheric meaning evolve together across villages, cities, nations, planets, solar systems, and eventually galactic-scale abstractions.

This expansion must not become a set of disconnected “empire systems.”

It must remain grounded in Mindstrata’s core principle:

> Macro-history must emerge from locally bounded agents, material constraints, biological processes, social relationships, institutions, and symbolic fields.

The cellular decay system, the PESTLE development tree, and multi-settlement dynamics must therefore be integrated into one unified architecture.

---

## 2. Relationship to the Human-Scale Plan

The previous plan deepened the agent into:

```text
Genome
  ↓
Organs / Hormones / Body
  ↓
Nervous System / Affect
  ↓
Cognition / Memory / Identity
  ↓
Relationships / Attachment / Status
  ↓
Culture / Noosphere / Institutions
```

This world-expansion plan adds the outer layers:

```text
Cellular / Microbial Substrate
  ↓
Organism / Body
  ↓
Household / Site
  ↓
Settlement / District
  ↓
Region / Trade Network
  ↓
Polity / Nation
  ↓
Civilization / Planetary System
  ↓
Stellar / Interstellar System
  ↓
Galactic Strategic Abstraction
```

The two plans are not separate.

They are vertically integrated:

```text
Cellular decay
  → disease
    → household stress
      → economic loss
        → rumor
          → religious interpretation
            → legal response
              → political legitimacy
                → diplomatic consequence
                  → technological adaptation
```

---

## 3. Core Architectural Principle: One Interdependent World

Mindstrata must avoid fragmented “system modules” that merely exchange numbers.

Instead, the expanded architecture should be organized around **five universal fields**.

### 3.1 The Five Fields

```text
1. Material Field
   matter, energy, resources, waste, temperature, moisture, chemistry

2. Biological Field
   cells, organisms, pathogens, ecosystems, decay, immunity, reproduction

3. Social Field
   relationships, kinship, households, factions, status, institutions

4. Noospheric Field
   memes, knowledge, law, religion, propaganda, science, narratives

5. Institutional Field
   political, economic, legal, military, administrative, religious, scientific organizations
```

Every system must attach to these fields.

For example:

| System | Field Attachment |
|---|---|
| Cellular decay | Material + Biological |
| Epidemic | Biological + Social + Institutional |
| Trade route | Material + Social + Institutional |
| Propaganda | Noospheric + Institutional |
| Law | Noospheric + Institutional + Social |
| Diplomacy | Institutional + Social + Noospheric |
| Technology | Noospheric + Material + Institutional |
| Religion | Noospheric + Social + Institutional |
| Military conflict | Institutional + Biological + Material + Social |

This prevents fragmentation.

---

# 4. Multi-Scale World Architecture

Mindstrata should support nested scale layers.

## 4.1 Scale Tiers

```text
Micro Scale
  cells, tissues, pathogens, organic matter, decay

Meso Scale
  bodies, agents, households, buildings, sites

Local Scale
  villages, towns, city districts, forts, monasteries

Regional Scale
  counties, river basins, trade corridors, provinces

Polity Scale
  city-states, kingdoms, republics, empires, federations

Civilizational Scale
  culture areas, religions, scientific traditions, economic zones

Planetary Scale
  biosphere, climate, global trade, planetary institutions

Stellar Scale
  solar systems, colonies, orbital habitats, interstellar routes

Galactic Scale
  strategic abstraction of galactic civilizations, communication lag, macro-factions
```

Important architectural rule:

> Lower scales are simulated in greater detail. Higher scales are simulated with increasing abstraction.

A galaxy should not be simulated as billions of agents.

Instead:

- local sites are agent-based,
- regions are household/settlement-aggregated,
- polities are institutionally aggregated,
- civilizations are noospheric/statistical,
- stellar/galactic layers are strategic and event-driven.

---

## 4.2 World Graph

The world should be represented as a nested graph, not just a grid.

```rust
pub enum WorldNodeKind {
    Tile,
    Site,
    District,
    Settlement,
    Region,
    Polity,
    Civilization,
    Planet,
    StarSystem,
    GalacticSector,
}
```

Each node has:

```rust
pub struct WorldNode {
    pub id: WorldNodeId,
    pub kind: WorldNodeKind,
    pub parent: Option<WorldNodeId>,
    pub children: Vec<WorldNodeId>,
    pub position: SpatialPosition,
    pub environment: EnvironmentState,
    pub population: PopulationAggregate,
    pub institutions: Vec<InstitutionId>,
    pub material_stocks: MaterialStocks,
    pub biological_state: BiologicalFieldState,
    pub cultural_state: CulturalFieldState,
    pub infrastructure: InfrastructureState,
    pub connectivity: Vec<WorldEdge>,
}
```

Edges represent flows:

```rust
pub enum WorldEdgeKind {
    Road,
    River,
    SeaRoute,
    TradeRoute,
    MigrationRoute,
    PilgrimageRoute,
    DiplomaticChannel,
    MilitaryCorridor,
    InformationNetwork,
    Pathway,
    OrbitalLane,
    Wormhole,       // optional late-game/sci-fi
    RelayNetwork,    // interstellar communication
}
```

Each edge has:

```rust
pub struct WorldEdge {
    pub from: WorldNodeId,
    pub to: WorldNodeId,
    pub kind: WorldEdgeKind,
    pub capacity: Fixed,
    pub danger: Fixed,
    pub travel_time: u64,
    pub maintenance_cost: Fixed,
    pub control: Option<InstitutionId>,
    pub toll: Fixed,
    pub contamination_risk: Fixed,
    pub cultural_friction: Fixed,
}
```

This allows trade, diplomacy, disease, rumors, armies, and technology to propagate through the same spatial structure.

---

## 4.3 Time Dilation by Scale

Different scales update at different frequencies.

Assuming the human-scale plan uses:

```text
1 local tick = 10 simulated minutes
```

Then:

| Scale | Update Frequency |
|---|---|
| Cellular / biological fast processes | every local tick |
| Agent behavior | every local tick |
| Household/site | every local tick |
| Settlement | every local tick or hourly phase |
| Regional trade/migration | daily |
| Diplomatic messages | based on travel time |
| Polity administration | weekly |
| Economic cycles | weekly/monthly |
| Environmental seasons | seasonal |
| Planetary climate | monthly/seasonal/yearly |
| Stellar travel | months/years |
| Galactic strategic shifts | decades/centuries |

Use deterministic event queues for delayed effects:

```rust
pub struct DelayedWorldEvent {
    pub arrival_tick: u64,
    pub origin: WorldNodeId,
    pub destination: WorldNodeId,
    pub payload: WorldEventPayload,
    pub fidelity: Fixed,
    pub mutation_seed: u64,
}
```

This preserves bounded knowledge at scale.

A message from another star system arrives late, may be distorted, and may be politically interpreted before verification is possible.

---

# 5. Multi-Settlement Intra-Sub-Cultural Dynamics

This system must simulate not only neighboring settlements, but the internal cultural complexity within them.

---

## 5.1 Settlement Types

```rust
pub enum SettlementKind {
    Hamlet,
    Village,
    Town,
    City,
    Fort,
    Monastery,
    Port,
    MiningCamp,
    UniversityTown,
    Capital,
    Metropolis,
    Colony,
    OrbitalHabitat,
    Station,
    PlanetaryCapital,
    StellarHub,
}
```

Each settlement has:

```rust
pub struct Settlement {
    pub id: SettlementId,
    pub kind: SettlementKind,
    pub population: u32,
    pub households: Vec<HouseholdId>,
    pub districts: Vec<DistrictId>,
    pub institutions: Vec<InstitutionId>,
    pub economy: SettlementEconomy,
    pub culture: SettlementCulture,
    pub subcultures: Vec<SubcultureId>,
    pub sanitation: SanitationState,
    pub disease_burden: Fixed,
    pub legitimacy: Fixed,
    pub unrest: Fixed,
    pub infrastructure: InfrastructureState,
    pub defense: DefenseState,
    pub trade_connections: Vec<WorldEdge>,
}
```

---

## 5.2 Districts and Local Subcultures

Cities and large settlements should contain districts.

```rust
pub struct District {
    pub id: DistrictId,
    pub settlement: SettlementId,
    pub kind: DistrictKind,
    pub population: u32,
    pub dominant_occupations: Vec<OccupationId>,
    pub wealth: Fixed,
    pub sanitation: Fixed,
    pub crime: Fixed,
    pub religious_affinity: Fixed,
    pub ethnic_affinity: Option<CultureId>,
    pub faction_alignment: Option<FactionId>,
    pub subculture: Option<SubcultureId>,
}
```

District kinds:

- market quarter,
- temple quarter,
- artisan quarter,
- dock quarter,
- noble quarter,
- slum,
- garrison,
- university quarter,
- foreign quarter,
- cemetery district,
- industrial zone,
- orbital ring sector.

Subcultures emerge from:

- occupation,
- class,
- ethnicity,
- religion,
- district,
- age cohort,
- education,
- faction,
- military service,
- migration origin,
- technological adoption,
- legal status.

---

## 5.3 Subculture Model

```rust
pub struct Subculture {
    pub id: SubcultureId,
    pub parent_culture: CultureId,
    pub identity_markers: Vec<IdentityMarker>,
    pub values: ValueProfile,
    pub memes: Vec<MemeId>,
    pub practices: Vec<PracticeId>,
    pub taboos: Vec<TabooId>,
    pub prestige: Fixed,
    pub marginalization: Fixed,
    pub cohesion: Fixed,
    pub boundary_strictness: Fixed,
    pub assimilation_pressure: Fixed,
    pub grievance: Fixed,
    pub network_density: Fixed,
}
```

Identity markers may include:

- dress,
- dialect,
- ritual,
- dietary taboo,
- occupation,
- ancestry myth,
- sacred symbol,
- legal status,
- technological adoption,
- political loyalty.

---

## 5.4 Cultural Dynamics

Subcultures interact through several processes.

### Assimilation

```text
assimilation_pressure =
    dominant_culture_prestige
  + institutional_enforcement
  + economic_incentive
  + intermarriage
  + education
  - subculture_cohesion
  - boundary_strictness
  - grievance
```

### Segregation

Segregation increases when:

- marginalization rises,
- legal discrimination exists,
- religious purity norms are strong,
- economic competition is high,
- fear/contagion rises,
- spatial separation exists.

### Syncretism

Syncretism occurs when:

- sustained contact exists,
- intermarriage is common,
- trade dependence is high,
- religious tolerance is high,
- shared external threat exists,
- prestige flows both ways.

### Radicalization

Radicalization increases when:

- grievance is high,
- humiliation events occur,
- institutions are illegitimate,
- echo chambers form,
- charismatic leaders appear,
- repression closes peaceful channels.

---

## 5.5 Inter-Settlement Dynamics

Settlements relate through:

```rust
pub enum InterSettlementRelation {
    TradePartner,
    Tributary,
    Ally,
    Rival,
    Enemy,
    Vassal,
    Suzerain,
    ReligiousCenter,
    ColonialParent,
    Colony,
    Neutral,
}
```

Each relation has:

```rust
pub struct SettlementRelation {
    pub from: SettlementId,
    pub to: SettlementId,
    pub kind: InterSettlementRelation,
    pub trust: Fixed,
    pub dependence: Fixed,
    pub tribute: Fixed,
    pub trade_volume: Fixed,
    pub cultural_distance: Fixed,
    pub military_threat: Fixed,
    pub diplomatic_history: Vec<DiplomaticEvent>,
}
```

---

## 5.6 Trade Routes

Trade routes are not abstract lines. They are material and biological conduits.

Each route carries:

- goods,
- people,
- rumors,
- pathogens,
- technologies,
- religious ideas,
- refugees,
- taxes,
- military logistics.

```rust
pub struct TradeRoute {
    pub id: TradeRouteId,
    pub nodes: Vec<WorldNodeId>,
    pub goods: Vec<RouteGood>,
    pub volume: Fixed,
    pub profitability: Fixed,
    pub danger: Fixed,
    pub disease_risk: Fixed,
    pub cultural_exchange: Fixed,
    pub control: Option<PolityId>,
    pub maintenance: Fixed,
}
```

Trade route effects:

- wealth increases,
- disease spreads,
- subcultures hybridize,
- dependence increases,
- strategic vulnerability emerges,
- banditry/protection rackets emerge,
- legal jurisdictions conflict.

---

## 5.7 Diplomacy

Diplomacy should be agent-mediated but institutionally grounded.

```rust
pub struct DiplomaticRelation {
    pub a: PolityId,
    pub b: PolityId,
    pub status: DiplomaticStatus,
    pub trust: Fixed,
    pub legitimacy_recognition: Fixed,
    pub treaties: Vec<Treaty>,
    pub grievances: Vec<Grievance>,
    pub tribute: Option<Tribute>,
    pub marriage_alliances: Vec<MarriageAlliance>,
    pub envoys: Vec<AgentId>,
    pub communication_delay: u64,
}
```

Diplomatic statuses:

- war,
- ceasefire,
- cold peace,
- neutrality,
- non-aggression,
- trade agreement,
- alliance,
- tributary,
- vassalage,
- federation,
- imperial integration.

Diplomatic actions:

- send envoy,
- propose treaty,
- demand tribute,
- arrange marriage,
- recognize legitimacy,
- denounce,
- embargo,
- declare war,
- sue for peace,
- form coalition,
- sponsor rebellion,
- exchange hostages,
- establish relay station.

Diplomacy must be affected by:

- distance,
- communication delay,
- rumor distortion,
- leader psychology,
- institutional legitimacy,
- domestic factions,
- religious affinity,
- economic dependence,
- military balance.

---

## 5.8 Conflict and War

War should scale from feud to interstellar conflict.

### Conflict Levels

```text
Individual feud
  ↓
Household rivalry
  ↓
Faction violence
  ↓
Settlement riot
  ↓
Banditry
  ↓
Local raid
  ↓
Regional war
  ↓
National war
  ↓
Coalition war
  ↓
Planetary war
  ↓
Interstellar war
  ↓
Galactic strategic conflict
```

### Military Model

```rust
pub struct MilitaryForce {
    pub id: MilitaryForceId,
    pub owner: PolityId,
    pub manpower: Fixed,
    pub morale: Fixed,
    pub supply: Fixed,
    pub training: Fixed,
    pub technology: Fixed,
    pub leadership: Option<AgentId>,
    pub doctrine: Doctrine,
    pub logistics_route: Vec<WorldNodeId>,
    pub disease_burden: Fixed,
    pub fatigue: Fixed,
}
```

War depends on:

- logistics,
- food,
- disease,
- morale,
- legitimacy,
- terrain,
- weather,
- technology,
- leadership,
- propaganda,
- economic endurance,
- civilian unrest.

This prevents war from becoming a pure numbers game.

An army can lose because:

- supply routes are raided,
- corpses contaminate water,
- fever spreads,
- home faction protests,
- religious legitimacy collapses,
- mercenaries go unpaid,
- harvest fails,
- rumors of defeat cause desertion.

---

# 6. PESTLE+ Development Tree

The user requested a PESTLE development tree with vertical stages and horizontal dynamics.

Traditional PESTLE includes:

- Political,
- Economic,
- Social,
- Technological,
- Legal,
- Environmental.

For Mindstrata, this must be expanded into **PESTLE+** because civilization cannot be understood without:

- Ethical,
- Religious/Spiritual,
- Military/Security,
- Informational/Epistemic,
- Administrative/Bureaucratic,
- Infrastructure/Logistics,
- Health/Medical,
- Cultural/Noospheric,
- Scientific.

---

## 6.1 PESTLE+ Systems

```text
1. Political / Governance
2. Economic / Production
3. Social / Demographic
4. Technological / Material Capability
5. Legal / Normative
6. Environmental / Ecological
7. Ethical / Moral
8. Religious / Spiritual
9. Military / Security
10. Informational / Epistemic
11. Administrative / Bureaucratic
12. Infrastructure / Logistics
13. Health / Medical
14. Cultural / Noospheric
15. Scientific / Research
```

These are not independent trees.

They are coupled developmental systems.

---

## 6.2 Generic Vertical Development Stages

Every PESTLE+ system can be mapped onto a common stage ladder.

```text
Stage 0: Pre-Systemic
Stage 1: Customary / Local
Stage 2: Institutional / Settlement
Stage 3: Bureaucratic / Regional
Stage 4: Rationalized / National
Stage 5: Networked / Planetary
Stage 6: Integrated / Stellar
Stage 7: Adaptive / Galactic
```

Not every system must reach every stage in every civilization.

A civilization may have:

- Stage 4 military,
- Stage 2 legal,
- Stage 5 information,
- Stage 3 health,
- Stage 1 environmental management.

This asymmetry creates historical texture.

---

## 6.3 Macro Development Variables

Each polity/civilization should maintain derived macro capacities.

```rust
pub struct MacroDevelopmentState {
    pub surplus: Fixed,
    pub energy_capture: Fixed,
    pub material_throughput: Fixed,
    pub information_bandwidth: Fixed,
    pub administrative_capacity: Fixed,
    pub legitimacy: Fixed,
    pub coercive_capacity: Fixed,
    pub fiscal_capacity: Fixed,
    pub logistical_capacity: Fixed,
    pub epistemic_integrity: Fixed,
    pub innovation_rate: Fixed,
    pub social_cohesion: Fixed,
    pub inequality: Fixed,
    pub ecological_load: Fixed,
    pub disease_burden: Fixed,
    pub complexity_overhead: Fixed,
    pub resilience: Fixed,
    pub collapse_risk: Fixed,
}
```

These are grounded in lower layers:

```text
fiscal_capacity =
    taxable_surplus
  × administrative_capacity
  × legitimacy
  × compliance
  - corruption
  - elite_evasion

innovation_rate =
    knowledge_stock
  × scientific_institution_quality
  × surplus
  × openness
  × network_connectivity
  - suppression
  - dogmatism

collapse_risk =
    scarcity
  + inequality
  + legitimacy_deficit
  + disease_burden
  + environmental_degradation
  + military_overextension
  + information_pathology
  - resilience
```

---

# 7. Deep Dive: Each PESTLE+ System

---

## 7.1 Political / Governance System

### Core Function

Coordination, collective decision-making, legitimacy, coercion, succession, public authority.

### Horizontal Components

- ruler/executive,
- council,
- bureaucracy,
- factions,
- patronage networks,
- local elites,
- military command,
- intelligence/gossip networks,
- taxation authority,
- legitimacy narratives.

### Vertical Development Stages

```text
0. Kin-based consensus
1. Elder council
2. Chiefdom
3. City-state council
4. Territorial monarchy/republic
5. Bureaucratic state
6. Constitutional/national state
7. Planetary federation
8. Stellar commonwealth
9. Galactic coordination authority
```

### Collective → Individual

Politics shapes agents through:

- law,
- taxation,
- conscription,
- public works,
- propaganda,
- punishment,
- welfare,
- recognition,
- censorship,
- emergency powers.

### Individual → Collective

Agents shape politics through:

- leadership,
- rebellion,
- voting if present,
- patronage,
- assassination,
- reform movements,
- bureaucratic sabotage,
- charisma,
- faction formation,
- court intrigue.

### Couplings

- Economy supplies fiscal capacity.
- Law stabilizes authority.
- Military enforces sovereignty.
- Religion legitimizes rule.
- Information shapes legitimacy.
- Environment constrains surplus.
- Health crises create emergency politics.

### Key Metrics

- legitimacy,
- state capacity,
- corruption,
- succession stability,
- factional polarization,
- repression,
- public trust.

---

## 7.2 Economic / Production System

### Core Function

Production, distribution, consumption, exchange, accumulation, investment.

### Horizontal Components

- households,
- farms,
- workshops,
- markets,
- guilds,
- banks,
- trade routes,
- labor pools,
- resource stocks,
- consumption demand,
- inequality structure.

### Vertical Development Stages

```text
0. Household subsistence
1. Reciprocity/redistribution
2. Local market
3. Monetary economy
4. Banking/credit
5. Mercantile capitalism/state economy
6. Industrial economy
7. Welfare/planned mixed economy
8. Planetary logistics economy
9. Stellar resource economy
10. Galactic exchange network
```

### Collective → Individual

Economy shapes agents through:

- prices,
- wages,
- scarcity,
- debt,
- property rights,
- labor discipline,
- consumption norms,
- class position.

### Individual → Collective

Agents shape economy through:

- innovation,
- entrepreneurship,
- labor choices,
- smuggling,
- strikes,
- consumption,
- investment,
- household formation,
- migration.

### Couplings

- Technology changes productivity.
- Law defines property/contracts.
- Politics taxes/regulates.
- Environment supplies resources.
- Military protects/disrupts trade.
- Culture shapes consumption and prestige goods.
- Decay/spoilage constrains storage.

### Key Metrics

- surplus,
- inequality,
- trade volume,
- price volatility,
- debt burden,
- labor participation,
- storage capacity,
- spoilage rate.

---

## 7.3 Social / Demographic System

### Core Function

Population structure, kinship, class, mobility, household formation, social cohesion.

### Horizontal Components

- households,
- kinship networks,
- classes/estates,
- age cohorts,
- gender/sex structures abstractly,
- migration flows,
- marriage markets,
- education pipelines,
- urban/rural divide.

### Vertical Development Stages

```text
0. Kin bands
1. Clan society
2. Estate/stratified society
3. Class society
4. Mass society
5. Networked society
6. Planetary society
7. Multi-world society
8. Galactic diaspora society
```

### Collective → Individual

Society shapes agents through:

- status assignment,
- marriage constraints,
- education,
- stigma,
- class opportunity,
- kin obligations,
- mobility ceilings.

### Individual → Collective

Agents shape society through:

- marriage,
- childbirth,
- migration,
- conversion,
- rebellion,
- education,
- household formation,
- cultural innovation.

### Couplings

- Economy shapes class.
- Politics shapes citizenship.
- Religion shapes marriage/kinship.
- Health shapes demography.
- War disrupts sex ratios and households.
- Environment shapes carrying capacity.

### Key Metrics

- population growth,
- age structure,
- household size,
- mobility,
- inequality,
- social trust,
- marriage rate,
- urbanization.

---

## 7.4 Technological / Material Capability System

### Core Function

Tools, techniques, energy use, production methods, material mastery.

### Horizontal Components

- tools,
- techniques,
- craft knowledge,
- machines,
- energy systems,
- materials,
- workshops,
- laboratories,
- supply chains,
- maintenance capacity.

### Vertical Development Stages

```text
0. Stone/bone tools
1. Fire/cooking/shelter
2. Agriculture
3. Metallurgy
4. Writing/measurement
5. Machinery/printing
6. Steam/industrialization
7. Electricity/chemicals
8. Computing/automation
9. Biotechnology/materials science
10. Fusion/advanced energy
11. Interstellar propulsion
12. Stellar engineering
13. Galactic-scale infrastructure
```

### Collective → Individual

Technology shapes agents through:

- labor productivity,
- health tools,
- communication speed,
- surveillance capacity,
- weapon lethality,
- daily routines.

### Individual → Collective

Agents shape technology through:

- invention,
- tinkering,
- apprenticeship,
- sabotage,
- adoption/resistance,
- scientific discovery,
- institutional funding.

### Couplings

- Science feeds technology.
- Economy funds adoption.
- Military demands innovation.
- Law regulates dangerous tech.
- Culture accepts/resists tech.
- Environment constrains resources.
- Infrastructure enables deployment.

### Key Metrics

- knowledge stock,
- adoption rate,
- maintenance capacity,
- energy capture,
- tool complexity,
- innovation rate,
- technological dependence.

---

## 7.5 Legal / Normative System

### Core Function

Rules, adjudication, property, contracts, punishment, rights, obligations.

### Horizontal Components

- customary norms,
- codified laws,
- courts,
- enforcement bodies,
- legal professions,
- prisons,
- contracts,
- property registries,
- appeals,
- legal legitimacy.

### Vertical Development Stages

```text
0. Customary taboo
1. Elder adjudication
2. Codified local law
3. Professional courts
4. Constitutional law
5. Administrative law
6. International law
7. Planetary law
8. Stellar law
9. Galactic normative frameworks
```

### Collective → Individual

Law shapes agents through:

- punishment,
- property security,
- contract enforcement,
- marriage law,
- inheritance law,
- citizenship,
- legal stigma.

### Individual → Collective

Agents shape law through:

- litigation,
- reform movements,
- legal scholarship,
- corruption,
- precedent,
- revolution,
- judicial interpretation.

### Couplings

- Politics enforces law.
- Economy requires contract/property.
- Religion may supply sacred law.
- Military may suspend law.
- Science creates new legal domains.
- Health crises create emergency law.

### Key Metrics

- rule of law,
- legal legitimacy,
- corruption,
- enforcement capacity,
- access to justice,
- legal complexity,
- rights protection.

---

## 7.6 Environmental / Ecological System

### Core Function

Climate, biomes, resources, waste absorption, carrying capacity, ecological stability.

### Horizontal Components

- climate,
- weather,
- hydrology,
- soil,
- flora/fauna,
- pathogens,
- waste sinks,
- pollution,
- biodiversity,
- natural hazards.

### Vertical Development Stages

```text
0. Passive adaptation
1. Foraging/fire use
2. Agriculture/irrigation
3. Deforestation/land modification
4. Resource depletion awareness
5. Conservation/sanitation
6. Environmental engineering
7. Climate management
8. Planetary biosphere management
9. Terraforming/ecopoiesis
10. Stellar-scale ecological engineering
```

### Collective → Individual

Environment shapes agents through:

- food availability,
- disease exposure,
- disaster risk,
- migration pressure,
- labor conditions,
- seasonal mood/stress.

### Individual → Collective

Agents shape environment through:

- farming,
- deforestation,
- irrigation,
- pollution,
- urbanization,
- conservation,
- waste disposal,
- terraforming.

### Couplings

- Economy extracts resources.
- Technology alters environmental impact.
- Law regulates pollution.
- Politics manages disasters.
- Health depends on sanitation.
- Decay contaminates soil/water.
- Religion may sacralize nature.

### Key Metrics

- carrying capacity,
- soil fertility,
- water quality,
- pollution,
- deforestation,
- climate stability,
- biodiversity,
- ecological resilience.

---

## 7.7 Ethical / Moral System

### Core Function

Moral intuitions, obligations, rights, purity, fairness, loyalty, sacred values.

### Horizontal Components

- moral foundations,
- ethical traditions,
- reform movements,
- moral entrepreneurs,
- shame/honor systems,
- guilt cultures,
- rights cultures,
- purity systems.

### Vertical Development Stages

```text
0. Kin loyalty morality
1. Honor/shame ethic
2. Customary duty ethic
3. Universalist religious ethic
4. Rights-based ethic
5. Welfare/utilitarian ethic
6. Planetary responsibility ethic
7. Multi-sentient ethical frameworks
8. Galactic ethical coordination
```

### Collective → Individual

Ethics shapes agents through:

- moral education,
- stigma,
- praise,
- guilt/shame,
- sacred boundaries,
- moral panic.

### Individual → Collective

Agents shape ethics through:

- moral innovation,
- prophecy,
- reform,
- dissent,
- martyrdom,
- philosophical argument,
- scandal.

### Couplings

- Religion often encodes ethics.
- Law formalizes some ethics.
- Politics uses moral legitimacy.
- Economy creates new moral dilemmas.
- Technology creates novel ethical problems.
- Health crises create triage ethics.

### Key Metrics

- moral cohesion,
- moral polarization,
- sacredness intensity,
- tolerance,
- hypocrisy perception,
- reform pressure.

---

## 7.8 Religious / Spiritual System

### Core Function

Meaning, ritual, sacred order, legitimacy, afterlife narratives, community binding.

### Horizontal Components

- temples/churches/sects,
- priesthoods,
- rituals,
- scriptures/oral traditions,
- sacred sites,
- relics,
- pilgrimages,
- heresies,
- moral calendars.

### Vertical Development Stages

```text
0. Animistic practice
1. Ancestor cult
2. Local shrine/temple
3. Organized priesthood
4. Scriptural religion
5. Reform/confessional religion
6. Civil religion
7. Planetary spiritual networks
8. Interstellar religious orders
9. Galactic meaning systems
```

### Collective → Individual

Religion shapes agents through:

- ritual comfort,
- moral discipline,
- community belonging,
- fear of punishment,
- meaning-making,
- grief processing.

### Individual → Collective

Agents shape religion through:

- conversion,
- heresy,
- prophecy,
- schism,
- saintly charisma,
- ritual innovation,
- apostasy.

### Couplings

- Politics gains legitimacy.
- Ethics is reinforced.
- Law may be sacred.
- Education is often religious.
- Health crises produce apocalyptic meaning.
- Decay/corpses become purity concerns.

### Key Metrics

- religiosity,
- orthodoxy,
- sectarian fragmentation,
- ritual participation,
- religious legitimacy,
- heresy pressure.

---

## 7.9 Military / Security System

### Core Function

Organized violence, defense, deterrence, coercion, territorial control.

### Horizontal Components

- armies,
- navies,
- militias,
- fortifications,
- logistics,
- intelligence,
- doctrine,
- military industry,
- veteran populations.

### Vertical Development Stages

```text
0. Kin warband
1. Levy/militia
2. Professional army
3. Standing army
4. Mass conscript army
5. Industrial military
6. Aerospace/planetary military
7. Interstellar force projection
8. Galactic strategic deterrence
```

### Collective → Individual

Military shapes agents through:

- conscription,
- trauma,
- discipline,
- status,
- disability,
- widowhood,
- occupation.

### Individual → Collective

Agents shape military through:

- leadership,
- desertion,
- innovation,
- mutiny,
- heroism,
- protest,
- veteran politics.

### Couplings

- Economy funds war.
- Technology changes lethality.
- Politics directs strategy.
- Law constrains conduct.
- Disease weakens armies.
- Religion motivates sacrifice.
- Logistics depends on infrastructure.

### Key Metrics

- manpower,
- morale,
- supply,
- technology gap,
- fortification,
- war exhaustion,
- veteran unrest.

---

## 7.10 Informational / Epistemic System

### Core Function

Knowledge creation, storage, transmission, verification, propaganda, communication.

### Horizontal Components

- oral networks,
- writing,
- archives,
- schools,
- printing,
- telecommunication,
- digital networks,
- censorship,
- rumor graphs,
- trusted authorities.

### Vertical Development Stages

```text
0. Oral tradition
1. Scribal culture
2. Manuscript culture
3. Print culture
4. Mass media
5. Digital network
6. Planetary information infrastructure
7. Interstellar archive/relay network
8. Galactic knowledge commons
```

### Collective → Individual

Information systems shape agents through:

- education,
- propaganda,
- rumor exposure,
- censorship,
- prestige narratives,
- scientific paradigms.

### Individual → Collective

Agents shape information through:

- invention,
- whistleblowing,
- heresy,
- journalism,
- gossip,
- forgery,
- scholarship.

### Couplings

- Science depends on information bandwidth.
- Politics depends on legitimacy narratives.
- Economy depends on price information.
- Military depends on intelligence.
- Religion depends on transmission.
- Law depends on records.

### Key Metrics

- literacy,
- bandwidth,
- trust in sources,
- misinformation load,
- echo chamber strength,
- archive integrity,
- propaganda effectiveness.

---

## 7.11 Administrative / Bureaucratic System

### Core Function

Implementation of collective decisions, record-keeping, coordination, taxation, regulation.

### Horizontal Components

- offices,
- roles,
- records,
- procedures,
- audits,
- corruption controls,
- staffing,
- jurisdiction.

### Vertical Development Stages

```text
0. Personal rule
1. Household administration
2. Patrimonial office
3. Professional bureaucracy
4. Meritocratic administration
5. Regulatory state
6. Planetary administration
7. Stellar administrative coordination
8. Algorithmic/galactic administration
```

### Collective → Individual

Administration shapes agents through:

- permits,
- taxes,
- registration,
- conscription lists,
- welfare distribution,
- surveillance.

### Individual → Collective

Agents shape administration through:

- corruption,
- reform,
- sabotage,
- expertise,
- bureaucratic entrepreneurship.

### Couplings

- Politics directs administration.
- Law formalizes procedure.
- Economy supplies revenue.
- Information enables records.
- Military may bypass administration.

### Key Metrics

- administrative capacity,
- corruption,
- procedural legitimacy,
- record integrity,
- response latency,
- compliance cost.

---

## 7.12 Infrastructure / Logistics System

### Core Function

Movement, storage, energy distribution, communication, sanitation.

### Horizontal Components

- roads,
- bridges,
- canals,
- ports,
- warehouses,
- granaries,
- sewers,
- power grids,
- data relays,
- orbital elevators.

### Vertical Development Stages

```text
0. Paths/trails
1. Roads/wells
2. Canals/ports
3. Rail/telegraph
4. Electric grids
5. Highway/air logistics
6. Digital logistics
7. Planetary supply networks
8. Orbital infrastructure
9. Stellar logistics lanes
10. Galactic relay infrastructure
```

### Collective → Individual

Infrastructure shapes agents through:

- travel time,
- food storage,
- disease exposure,
- market access,
- communication speed.

### Individual → Collective

Agents shape infrastructure through:

- construction,
- maintenance/neglect,
- sabotage,
- usage patterns,
- political demand.

### Couplings

- Economy depends on logistics.
- Military depends on supply lines.
- Health depends on sanitation.
- Decay depends on storage.
- Politics depends on public works legitimacy.

### Key Metrics

- connectivity,
- maintenance backlog,
- storage capacity,
- spoilage loss,
- transit time,
- sanitation quality.

---

## 7.13 Health / Medical System

### Core Function

Disease prevention, healing, reproduction, disability management, public health.

### Horizontal Components

- healers,
- hospitals,
- pharmacies,
- public health boards,
- quarantine,
- sanitation,
- medical knowledge,
- caregiving households.

### Vertical Development Stages

```text
0. Household care
1. Herbal/folk medicine
2. Temple medicine
3. Humoral/early rational medicine
4. Sanitation movement
5. Germ theory
6. Biomedicine
7. Public health state
8. Genetic/precision medicine
9. Planetary health monitoring
10. Stellar biosecurity
```

### Collective → Individual

Health systems shape agents through:

- treatment access,
- vaccination if available,
- quarantine,
- medical stigma,
- disability support.

### Individual → Collective

Agents shape health through:

- caregiving,
- medical innovation,
- compliance/resistance,
- hygiene practices,
- protest against quarantine.

### Couplings

- Decay creates pathogens.
- Environment shapes disease.
- Economy funds care.
- Law enables quarantine.
- Religion interprets illness.
- Military spreads disease.

### Key Metrics

- disease burden,
- life expectancy,
- infant mortality abstracted,
- healer capacity,
- sanitation,
- epidemic risk,
- public compliance.

---

## 7.14 Cultural / Noospheric System

### Core Function

Meaning, identity, symbols, memory, aesthetic value, collective imagination.

### Horizontal Components

- myths,
- memes,
- art,
- language/dialect,
- rituals,
- monuments,
- education,
- cultural heroes,
- taboos.

### Vertical Development Stages

```text
0. Oral micro-tradition
1. Local customary culture
2. Monumental/temple culture
3. Literary culture
4. Print/public culture
5. Mass culture
6. Networked culture
7. Planetary culture
8. Multi-world culture
9. Galactic symbolic ecology
```

### Collective → Individual

Culture shapes agents through:

- identity formation,
- aesthetic taste,
- moral categories,
- language,
- ritual participation.

### Individual → Collective

Agents shape culture through:

- art,
- heresy,
- fashion,
- reform,
- scholarship,
- scandal,
- innovation.

### Couplings

- Religion supplies sacred symbols.
- Politics uses legitimacy myths.
- Economy commodifies culture.
- Technology changes media.
- War creates heroic/traumatic memory.
- Decay creates purity/pollution symbolism.

### Key Metrics

- cultural cohesion,
- symbolic polarization,
- prestige distribution,
- memory intensity,
- innovation openness,
- sacred boundary strength.

---

## 7.15 Scientific / Research System

### Core Function

Systematic knowledge production, verification, instrumentation, theory formation.

### Horizontal Components

- researchers,
- laboratories,
- academies,
- universities,
- observatories,
- archives,
- peer review,
- funding bodies,
- instrumentation,
- paradigms.

### Vertical Development Stages

```text
0. Practical craft knowledge
1. Empirical observation
2. Natural philosophy
3. Experimental method
4. Institutional science
5. Industrial R&D
6. Big science
7. Computational science
8. Planetary science networks
9. Stellar science programs
10. Galactic-scale research coordination
```

### Collective → Individual

Science shapes agents through:

- education,
- technical roles,
- worldview change,
- medical intervention,
- surveillance tools.

### Individual → Collective

Agents shape science through:

- discovery,
- theory formation,
- paradigm challenge,
- fraud,
- mentorship,
- institutional reform.

### Couplings

- Technology depends on science.
- Economy funds research.
- Politics funds/suppresses science.
- Religion may conflict or integrate.
- Law regulates experimentation.
- Information systems determine verification speed.

### Key Metrics

- knowledge stock,
- research throughput,
- paradigm stability,
- instrumentation quality,
- scientific legitimacy,
- suppression/openness.

---

# 8. PESTLE+ Development Tree Structure

The development tree should not be a simple technology tree.

It should be a **multi-tree institutional capability graph**.

## 8.1 Development Node

```rust
pub struct DevelopmentNode {
    pub id: DevelopmentNodeId,
    pub system: PestleSystem,
    pub stage: u32,
    pub name: String,
    pub prerequisites: Vec<DevelopmentNodeId>,
    pub soft_prerequisites: Vec<SoftPrerequisite>,
    pub research_cost: Fixed,
    pub administrative_cost: Fixed,
    pub legitimacy_cost: Fixed,
    pub material_cost: Vec<ResourceAmount>,
    pub effects: Vec<DevelopmentEffect>,
    pub risks: Vec<DevelopmentRisk>,
    pub cultural_resistance: Fixed,
    pub institutional_capacity_required: Fixed,
}
```

## 8.2 Soft Prerequisites

Not everything can be expressed as a hard tech prerequisite.

```rust
pub enum SoftPrerequisite {
    MinimumLiteracy(Fixed),
    MinimumUrbanization(Fixed),
    MinimumLegitimacy(Fixed),
    MinimumAdministrativeCapacity(Fixed),
    MinimumTradeConnectivity(Fixed),
    MinimumScientificInstitutionQuality(Fixed),
    MinimumLegalComplexity(Fixed),
    MinimumEnergyCapture(Fixed),
    MinimumSocialCohesion(Fixed),
    AbsenceOfCollapseRisk(Fixed),
}
```

## 8.3 Example Development Tree Branch

```text
Political:
  KinCouncil
    → ElderCouncil
      → Chiefdom
        → CityStateAdministration
          → TerritorialBureaucracy
            → ConstitutionalState
              → PlanetaryFederation
                → StellarCommonwealth

Economic:
  HouseholdSubsistence
    → RedistributionEconomy
      → LocalMarket
        → MonetaryEconomy
          → BankingSystem
            → IndustrialEconomy
              → PlanetaryLogistics
                → StellarTradeNetwork

Scientific:
  CraftKnowledge
    → EmpiricalObservation
      → NaturalPhilosophy
        → ExperimentalScience
          → InstitutionalScience
            → ComputationalScience
              → PlanetaryScienceNetwork
                → StellarScienceProgram

Health:
  FolkMedicine
    → TempleMedicine
      → SanitationMovement
        → GermTheory
          → PublicHealthAdministration
            → Biomedicine
              → PlanetaryBiosecurity
                → StellarBiosecurity

Environmental:
  PassiveAdaptation
    → Irrigation
      → LandManagement
        → Conservation
          → SanitationEngineering
            → ClimateMonitoring
              → PlanetaryEcosystemManagement
                → Terraforming
```

---

## 8.4 Horizontal and Vertical Integration

Horizontal integration means systems interact at the same stage.

Example:

```text
Stage 3 Economic: Monetary economy
requires
Stage 2 Legal: Codified contract law
and
Stage 2 Administrative: Record-keeping
and
Stage 2 Infrastructure: Roads/markets
```

Vertical integration means higher stages require lower stages.

Example:

```text
Stage 5 Health: Public health administration
requires
Stage 4 Scientific: Germ theory
Stage 3 Administrative: Bureaucratic capacity
Stage 3 Legal: Emergency/quarantine law
Stage 3 Infrastructure: Sanitation systems
```

---

# 9. Cellular Decay Simulation Mechanism

This must be a complete micro-biological system, not merely “food spoils.”

It must simulate what happens when life is no longer flowing through organic matter.

---

## 9.1 Core Principle

Living organic matter is maintained by:

```text
circulation
+ metabolism
+ immune regulation
+ cellular repair
+ waste removal
+ boundary integrity
```

When these stop or fail:

```text
hypoxia
→ cellular stress
→ necrosis/apoptosis
→ autolysis
→ microbial proliferation
→ putrefaction
→ structural collapse
→ ecological recycling
```

This applies to:

- corpses,
- severed limbs,
- food,
- hides,
- wood,
- waste,
- sewage,
- medical samples,
- relics,
- compost,
- battlefield remains,
- spoiled grain,
- contaminated water.

---

## 9.2 Organic Matter Component

```rust
pub struct OrganicMatter {
    pub id: OrganicMatterId,
    pub kind: OrganicMatterKind,
    pub owner: Option<EntityId>,
    pub location: OrganicLocation,

    pub mass: Fixed,
    pub water_content: Fixed,
    pub nutrient_density: Fixed,
    pub structural_matrix: Fixed,

    pub living_cell_mass: Fixed,
    pub stressed_cell_mass: Fixed,
    pub necrotic_mass: Fixed,
    pub autolysis_level: Fixed,

    pub enzymatic_activity: Fixed,
    pub microbial_load: Fixed,
    pub pathogen_load: Fixed,
    pub toxin_load: Fixed,

    pub oxygenation: Fixed,
    pub temperature: Fixed,
    pub ph: Fixed,
    pub salinity: Fixed,

    pub preservation: PreservationState,
    pub decay_stage: DecayStage,
    pub integrity: Fixed,
    pub odor: Fixed,
    pub contagion_risk: Fixed,
}
```

Kinds:

```rust
pub enum OrganicMatterKind {
    Corpse,
    BodyPart,
    FoodPlant,
    FoodMeat,
    Grain,
    Hide,
    Wood,
    Leather,
    Waste,
    Sewage,
    Compost,
    MedicalSample,
    Relic,
    Carrion,
}
```

Location:

```rust
pub enum OrganicLocation {
    OnGround,
    Buried { depth: Fixed },
    Submerged { water_kind: WaterKind },
    Indoors,
    StorageContainer { container_id: ContainerId },
    Inventory { agent_id: AgentId },
    River,
    Well,
    Cemetery,
    Battlefield,
    Temple,
    Hospital,
    Market,
    OrbitalCargo,
}
```

---

## 9.3 Decay Stages

```rust
pub enum DecayStage {
    Fresh,
    CellularStress,
    Autolysis,
    Bloat,
    ActivePutrefaction,
    AdvancedDecay,
    Desiccation,
    Skeletonization,
    Mineralization,
    Fossilization,
}
```

Not all matter follows the same path.

Possible trajectories:

```text
Fresh corpse
  → autolysis
    → bloat
      → active decay
        → skeletonization
          → mineralization

Dry corpse
  → desiccation
    → mummification

Waterlogged corpse
  → anaerobic decay
    → adipocere-like preservation
      → slow decomposition

Salted meat
  → inhibited microbial growth
    → long preservation

Unstored grain
  → mold
    → toxin
      → spoilage
```

---

## 9.4 Decay Formula

A deterministic abstract model:

```text
decay_rate =
    base_decay_rate
  × temperature_factor
  × moisture_factor
  × oxygen_factor
  × nutrient_factor
  × enzymatic_factor
  × microbial_factor
  × pathogen_factor
  × scavenger_factor
  × mechanical_factor
  × (1 - preservation_factor)
  × (1 - immune_factor if alive)
```

### Temperature Factor

Use a Q10-like curve:

```text
temperature_factor = 2^((temperature - 20) / 10)
```

Clamped to safe bounds.

Cold slows decay. Heat accelerates it.

### Moisture Factor

```text
moisture_factor =
    too dry: low
    optimal: high
    waterlogged: variable depending on oxygen
```

### Oxygen Factor

```text
oxygen_factor =
    aerobic: high microbial putrefaction
    anaerobic: slower, different byproducts
```

### Preservation Factor

```text
preservation_factor =
    drying
  + salting
  + smoking
  + freezing
  + embalming
  + fermentation
  + chemical treatment
  + refrigeration
  + vacuum
  + irradiation
  + sterile containment
```

---

## 9.5 Living Tissue Integration

For living agents, cellular decay is not separate from biology.

It integrates with:

```rust
pub struct TissueHealth {
    pub oxygenation: Fixed,
    pub nutrient_supply: Fixed,
    pub waste_removal: Fixed,
    pub immune_activity: Fixed,
    pub inflammation: Fixed,
    pub necrosis: Fixed,
    pub regeneration: Fixed,
    pub infection: Fixed,
    pub trauma: Fixed,
}
```

If circulation fails:

```text
oxygenation ↓
→ cellular stress ↑
→ necrosis ↑
→ immune activation ↑
→ inflammation ↑
→ systemic shock risk ↑
```

If infection spreads:

```text
pathogen_load ↑
→ immune response ↑
→ tissue damage ↑
→ toxin_load ↑
→ sepsis risk ↑
→ organ failure risk ↑
```

This connects directly to the human-scale biological systems:

- cardiovascular,
- immune,
- respiratory,
- nervous,
- endocrine,
- digestive,
- muscular.

---

## 9.6 Corpses and Public Health

Unmanaged corpses produce:

```text
odor
→ disgust
→ purity concern
→ rumor
→ fear of curse/plague
→ scavengers
→ pathogen load
→ water contamination
→ disease outbreak
→ labor shortage
→ economic loss
→ political legitimacy decline
```

Corpse management becomes a civilizational issue.

Responses:

- burial,
- cremation,
- mass graves,
- embalming,
- relic preservation,
- quarantine zones,
- sanitation law,
- temple ritual,
- public health boards.

---

## 9.7 Food Spoilage and Economy

Food is organic matter.

```rust
pub struct FoodStock {
    pub resource_id: ResourceId,
    pub quantity: Fixed,
    pub freshness: Fixed,
    pub contamination: Fixed,
    pub preservation: PreservationState,
    pub storage: StorageCondition,
}
```

Spoilage affects:

- hunger,
- disease,
- prices,
- trade,
- storage technology,
- military logistics,
- household stress,
- inequality.

Poor households suffer more from spoilage because they lack:

- granaries,
- salt,
- ice houses,
- refrigeration,
- secure storage.

---

## 9.8 Waste, Sewage, and Sanitation

Waste is organic matter too.

```rust
pub struct WasteAccumulation {
    pub quantity: Fixed,
    pub toxicity: Fixed,
    pub pathogen_load: Fixed,
    pub odor: Fixed,
    pub containment: Fixed,
}
```

If sanitation fails:

```text
waste accumulates
→ pathogens rise
→ water quality falls
→ disease spreads
→ settlement health declines
→ legitimacy declines
```

This creates demand for:

- latrines,
- sewers,
- waste collectors,
- public health law,
- zoning,
- quarantine,
- burial regulation.

---

## 9.9 Environmental Integration

Decay feeds back into ecology.

```text
organic decay
→ nutrients returned to soil
→ microbial population changes
→ scavenger population changes
→ water contamination
→ soil fertility change
→ disease reservoirs
```

A battlefield may become:

- fertile through decomposition,
- cursed through trauma and rumor,
- diseased through contamination,
- politically symbolic through memory.

---

# 10. Cross-System Interdependence

This is the most important part.

The systems must not be siloed.

---

## 10.1 Example Cascade: Corpse → Empire

```text
1. Battle occurs near river.
2. Corpses remain unburied.
3. Cellular decay begins.
4. Temperature and moisture accelerate putrefaction.
5. Scavengers and insects proliferate.
6. Pathogens enter water supply.
7. Nearby settlement experiences fever outbreak.
8. Labor participation falls.
9. Harvest delayed.
10. Food prices rise.
11. Rumor spreads: “The river is cursed.”
12. Temple performs purification ritual.
13. Council imposes quarantine.
14. Quarantine disrupts trade.
15. Merchants resent council.
16. Faction forms against council.
17. Rival polity funds dissent.
18. Legitimacy declines.
19. Protest becomes riot.
20. Military sent to restore order.
21. Army suffers disease.
22. Border defense weakens.
23. Neighbor declares war.
24. War accelerates sanitation technology.
25. Public health institution emerges.
```

This is the target level of interconnection.

---

## 10.2 Example Cascade: Printing Press → Reformation → State Formation

```text
1. Artisan improves press.
2. Information bandwidth rises.
3. Religious pamphlets spread.
4. Heretical meme gains traction.
5. Temple legitimacy fractures.
6. Subcultures form around pamphlet networks.
7. Political factions exploit religious dissent.
8. Legal conflicts over censorship emerge.
9. Wars of religion become possible.
10. States develop bureaucracies to manage confession.
11. Education expands.
12. Scientific networks emerge.
13. Administrative capacity rises.
14. National identity strengthens.
```

---

## 10.3 Example Cascade: Refrigeration → Planetary Logistics

```text
1. Scientific knowledge of thermodynamics rises.
2. Refrigeration technology emerges.
3. Food spoilage declines.
4. Urban population capacity rises.
5. Disease burden declines.
6. Military logistics improve.
7. Trade routes lengthen.
8. Colonial extraction intensifies.
9. Environmental load rises.
10. Planetary supply chains emerge.
11. Climate impact increases.
12. Planetary governance pressure rises.
```

---

# 11. Unified Data Architecture

## 11.1 Proposed Crates/Modules

Initially inside `mindstrata-sim`, later separate crates:

```text
mindstrata-world/
  scale.rs
  node.rs
  edge.rs
  settlement.rs
  district.rs
  region.rs
  polity.rs
  civilization.rs
  planet.rs
  stellar.rs
  galactic.rs
  routes.rs
  diplomacy.rs
  war.rs
  migration.rs
  subculture.rs

mindstrata-pestle/
  mod.rs
  development_node.rs
  political.rs
  economic.rs
  social.rs
  technological.rs
  legal.rs
  environmental.rs
  ethical.rs
  religious.rs
  military.rs
  informational.rs
  administrative.rs
  infrastructure.rs
  health.rs
  cultural.rs
  scientific.rs
  macro_state.rs

mindstrata-decay/
  mod.rs
  organic_matter.rs
  tissue.rs
  corpse.rs
  food.rs
  waste.rs
  preservation.rs
  pathogen.rs
  sanitation.rs
  decay_ecology.rs

mindstrata-environment/
  climate.rs
  weather.rs
  hydrology.rs
  soil.rs
  biome.rs
  pollution.rs
  disaster.rs
```

---

## 11.2 Cross-System Event Bus

Add new event categories:

```rust
pub enum WorldEvent {
    SettlementFounded { ... },
    RouteOpened { ... },
    RouteDisrupted { ... },
    DiplomaticRelationChanged { ... },
    TreatySigned { ... },
    WarDeclared { ... },
    BattleOccurred { ... },
    CorpseDecayed { ... },
    WaterContaminated { ... },
    EpidemicStarted { ... },
    QuarantineEnacted { ... },
    TechUnlocked { ... },
    InstitutionReformed { ... },
    SubcultureFormed { ... },
    CulturalSchism { ... },
    ColonyFounded { ... },
    MessageDelayed { ... },
    StarRouteOpened { ... },
    GalacticFactionShifted { ... },
}
```

All events feed provenance.

---

# 12. Specification Expansion

Add RON files:

```text
specs/
  world/
    nodes.ron
    edges.ron
    settlements.ron
    regions.ron
    polities.ron
    civilizations.ron
    planets.ron
    stellar.ron
    galactic.ron
  pestle/
    political_tree.ron
    economic_tree.ron
    social_tree.ron
    technological_tree.ron
    legal_tree.ron
    environmental_tree.ron
    ethical_tree.ron
    religious_tree.ron
    military_tree.ron
    informational_tree.ron
    administrative_tree.ron
    infrastructure_tree.ron
    health_tree.ron
    cultural_tree.ron
    scientific_tree.ron
  decay/
    organic_profiles.ron
    preservation.ron
    pathogens.ron
    sanitation.ron
    waste.ron
  environment/
    climate.ron
    weather.ron
    biomes.ron
    hydrology.ron
    disasters.ron
```

Example development node:

```ron
(
    id: "public_health_administration",
    system: Health,
    stage: 5,
    prerequisites: [
        "germ_theory",
        "sanitation_engineering",
        "bureaucratic_medicine",
    ],
    soft_prerequisites: [
        MinimumAdministrativeCapacity(0.55),
        MinimumLegalComplexity(0.45),
        MinimumUrbanization(0.4),
    ],
    effects: [
        ReduceDiseaseBurden(0.2),
        EnableQuarantineLaw(),
        EnableVitalStatistics(),
        IncreaseLegitimacyIfSuccessful(),
    ],
    risks: [
        AuthoritarianAbuse,
        PublicResistance,
        BureaucraticOverreach,
    ],
)
```

---

# 13. Simulation Loop Expansion

The expanded tick loop should include:

```text
Local Fast Phase
  cellular decay
  agent biology
  agent cognition
  local interactions

Local Slow Phase
  household decisions
  site inventory
  sanitation
  disease transmission

Settlement Phase
  district updates
  market updates
  unrest
  rituals
  local institutions

Regional Phase
  trade routes
  migration
  rumor propagation
  pathogen spread
  diplomatic messages

Polity Phase
  taxation
  administration
  law
  military logistics
  development investment

Civilizational Phase
  cultural drift
  scientific progress
  religious movements
  macro indicators

Planetary Phase
  climate
  biosphere
  global trade
  pandemics
  planetary institutions

Stellar Phase
  interstellar travel
  colony updates
  communication relays
  stellar economics

Galactic Phase
  strategic faction shifts
  long-horizon technological paradigms
  galactic communication delays
```

---

# 14. Level-of-Detail Strategy

This expansion is impossible without LOD.

## 14.1 Simulation Tiers

### Full Agent Simulation

For:

- focal agents,
- leaders,
- inventors,
- diplomats,
- generals,
- prophets,
- key household members.

### Aggregate Simulation

For:

- households,
- districts,
- regiments,
- caravans,
- monasteries,
- universities.

### Statistical Field Simulation

For:

- planetary populations,
- stellar colonies,
- galactic sectors.

Promote/demote entities dynamically:

```text
If a colony becomes politically important → promote to settlement simulation.
If a district becomes crisis-relevant → promote to agent-level sampling.
If a war becomes distant and stable → demote to strategic abstraction.
```

---

# 15. Testing Strategy

## 15.1 Unit Tests

- decay rate increases with temperature,
- preservation reduces decay,
- burial reduces scavenging,
- water contamination increases disease risk,
- trade route transmits goods and rumors,
- diplomatic insult reduces trust,
- development node requires prerequisites,
- subculture divergence increases with isolation,
- military supply decays over long routes,
- message delay respects distance.

## 15.2 Integration Tests

- unburied corpses can trigger epidemic,
- epidemic can reduce legitimacy,
- quarantine requires legal/administrative capacity,
- printing technology increases meme spread,
- refrigeration reduces food spoilage,
- trade route connects subcultures,
- war disrupts trade and increases disease,
- scientific institution increases innovation,
- environmental degradation reduces surplus,
- collapse risk rises from combined stressors.

## 15.3 Statistical Emergence Tests

Over many seeds:

- larger settlements have more subcultures,
- trade connectivity increases wealth and disease,
- isolated cultures diverge,
- state capacity correlates with legal/administrative development,
- sanitation reduces epidemic frequency,
- military overextension increases collapse risk,
- scientific openness increases innovation,
- inequality increases unrest,
- environmental degradation reduces long-term population.

---

# 16. Implementation Roadmap

## Phase W0: Unified World Kernel

### Goal

Prepare the architecture without breaking existing simulation.

### Tasks

- introduce `WorldNode`,
- introduce `WorldEdge`,
- introduce scale scheduler,
- introduce cross-system event bus,
- introduce macro development state,
- introduce organic matter component.

### Acceptance

- existing tests pass,
- single settlement still works,
- world graph can represent parent/child nodes.

---

## Phase W1: Multi-Settlement Regional Layer

### Goal

Enable neighboring settlements and regional flows.

### Systems

- settlement graph,
- trade routes,
- migration,
- regional rumors,
- regional disease spread,
- diplomacy v1.

### Acceptance

- two settlements trade,
- disease can travel route,
- rumor mutates over distance,
- migration responds to scarcity,
- diplomatic relations affect trade.

---

## Phase W2: PESTLE+ Development Tree v1

### Goal

Add vertical development stages.

### Systems

- development nodes,
- macro capacities,
- political/economic/legal/tech/scientific trees,
- institutional reform,
- innovation adoption.

### Acceptance

- prerequisites block invalid unlocks,
- administrative capacity affects taxation,
- scientific institutions increase innovation,
- legal complexity enables contracts/quarantine,
- political legitimacy affects reform success.

---

## Phase W3: Cellular Decay and Sanitation

### Goal

Make organic decay a first-class system.

### Systems

- organic matter,
- decay stages,
- preservation,
- pathogens,
- sanitation,
- waste,
- food spoilage,
- corpse management.

### Acceptance

- food spoils,
- corpses decay,
- poor sanitation increases disease,
- burial/cremation reduces contamination,
- preservation tech reduces spoilage,
- epidemics can emerge from decay.

---

## Phase W4: Environmental and Health Integration

### Goal

Connect decay, environment, health, and institutions.

### Systems

- weather,
- climate,
- hydrology,
- soil,
- pollution,
- public health,
- quarantine,
- hospitals,
- sanitation law.

### Acceptance

- weather affects decay and harvest,
- water contamination causes disease,
- public health law can contain epidemics,
- environmental degradation reduces carrying capacity.

---

## Phase W5: Polity, Law, Military, Diplomacy Depth

### Goal

Make nations and wars systemic.

### Systems

- polities,
- law courts,
- taxation,
- conscription,
- logistics,
- treaties,
- war exhaustion,
- rebellion,
- state formation.

### Acceptance

- war depends on logistics,
- law enables property/contracts,
- taxation depends on administration,
- diplomacy responds to legitimacy and threat,
- military disease affects outcomes.

---

## Phase W6: Planetary Layer

### Goal

Enable planetary-scale dynamics.

### Systems

- climate system,
- global trade,
- pandemics,
- planetary institutions,
- world religions,
- scientific networks,
- environmental management.

### Acceptance

- planetary climate affects regions,
- pandemics can spread through trade,
- planetary institutions emerge under pressure,
- global culture diverges/converges.

---

## Phase W7: Stellar and Galactic Abstraction

### Goal

Support long-horizon expansion without destroying performance.

### Systems

- colonies,
- interstellar travel time,
- communication delay,
- stellar economies,
- galactic strategic factions,
- cultural divergence across star systems.

### Acceptance

- messages arrive late,
- colonies diverge culturally,
- interstellar trade is strategically meaningful,
- galactic layer remains abstract but causally connected.

---

# 17. Risks and Mitigations

## Risk 1: Infinite Scope

**Risk:** Villages to galaxies is too large.

**Mitigation:** Implement scale gates. Do not enable stellar/galactic layers until regional/planetary systems are stable.

---

## Risk 2: Fragmentation

**Risk:** Systems become disconnected modules.

**Mitigation:** Require every system to attach to material, biological, social, noospheric, or institutional fields. Use cross-system cascade tests.

---

## Risk 3: Performance Collapse

**Risk:** Too many entities and scales.

**Mitigation:** Strict LOD, aggregate simulation, dynamic promotion/demotion, slow-phase schedulers.

---

## Risk 4: Macro Systems Become Ungrounded

**Risk:** Polity variables float free from agents.

**Mitigation:** Derive macro variables from agents, households, institutions, and material flows.

---

## Risk 5: Decay Becomes Gross but Shallow

**Risk:** Decay is only aesthetic.

**Mitigation:** Connect decay to disease, economy, law, religion, politics, infrastructure, and environment.

---

## Risk 6: PESTLE Becomes Static Tech Tree

**Risk:** Development is just unlock buttons.

**Mitigation:** Use soft prerequisites, institutional capacity, legitimacy costs, cultural resistance, and regression/collapse risk.

---

# 18. Immediate Next Steps After Human-Scale Plan

Once the human-scale plan has delivered embodied agents, psychology, relationships, and culture:

1. **Introduce `WorldNode` and `WorldEdge`.**
2. **Introduce `OrganicMatter` and `DecayStage`.**
3. **Introduce `MacroDevelopmentState`.**
4. **Introduce `DevelopmentNode` graph.**
5. **Introduce regional trade routes.**
6. **Introduce settlement sanitation and disease coupling.**
7. **Introduce PESTLE+ tree v1.**
8. **Add cross-system cascade tests.**

The first demonstrable milestone should be:

> A corpse left unburied after a local conflict can contaminate water, cause disease, generate rumor, trigger religious ritual, pressure the council into sanitation law, and affect regional trade legitimacy — all without scripted events.

---

# 19. Final Architectural Summary

The expanded Mindstrata world should become:

```text
A deterministic, multi-scale simulation
where cellular processes affect bodies,
bodies affect households,
households affect settlements,
settlements affect regions,
regions affect polities,
polities affect civilizations,
civilizations affect planets,
planets affect stellar networks,
and all of these are mediated by
material flows,
biological realities,
social relationships,
institutional power,
and noospheric meaning.
```

The key invariant is:

> Nothing is purely macro.  
> Nothing is purely micro.  
> Every system is embedded in matter, life, mind, society, and meaning.

Cellular decay is not a spoilage feature.  
It is the micro-ecological consequence of interrupted life.

PESTLE is not a tech tree.  
It is the developmental anatomy of civilization.

Multi-settlement expansion is not a bigger map.  
It is the emergence of history across scales.
