# Mindstrata — Human-Scale Deepening Architectural Plan

---

## 1. Architectural Mandate

Mindstrata already has a strong foundation:

- deterministic simulation,
- bounded agent knowledge,
- appraisal-based emotion,
- belief updating,
- gossip,
- norms,
- institutions,
- factions,
- market/ecology/demography,
- causal provenance.

The next evolutionary step is to make each human agent feel less like a “simulation unit” and more like a **living, embodied, developmental, socially embedded, meaning-making mind**.

The target is:

> **Each agent should become a locally bounded, embodied cognitive system whose biology, psychology, relationships, and cultural exposure interact to produce lifelike history.**

This plan treats the human agent as a multi-scale system:

```text
Genes
  ↓
Organs / Hormones / Body
  ↓
Nervous System / Arousal / Pain / Interoception
  ↓
Perception / Attention / Memory
  ↓
Emotion / Appraisal / Motivation
  ↓
Identity / Belief / Morality / Narrative
  ↓
Decision / Action / Speech
  ↓
Relationships / Attachment / Status / Kinship
  ↓
Groups / Factions / Institutions / Culture
  ↓
Noospheric Field: rumors, memes, propaganda, collective meaning
```

The agent should not merely “have traits.” The agent should have:

- a body that regulates itself,
- a nervous system that modulates cognition,
- a mind that predicts and interprets,
- a self-model that defends identity,
- a social graph that shapes belonging,
- a cultural field that supplies meaning,
- a history that changes future behavior.

---

## 2. Current Baseline Assessment

### 2.1 What Mindstrata Already Does Well

Mindstrata’s current architecture is unusually strong for an emergent simulation:

| Area | Current Strength |
|---|---|
| Determinism | Fixed-point math, seeded RNG, golden replay |
| Agent cognition | Needs, traits, emotion, appraisal, beliefs, memory, attention |
| Social layer | Relationships, gossip, norms, witnesses |
| Institutions | Council, Temple, Market with legitimacy and collective psychology |
| Emergence | Revolt, panic, trust erosion, inequality, moral panic possible |
| Debuggability | Provenance, journal, tests, spec linting |

The current agent is already more psychologically modeled than most game agents.

### 2.2 What Is Missing for “Lifelike” Human Simulation

The major missing dimensions are:

#### Biological

The current body is abstract:

```rust
BodyState {
    health,
    energy,
    hunger,
    thirst,
    fatigue,
    sickness,
    injury,
    fertility,
}
```

This is useful, but not embodied enough.

Missing:

- genetics,
- hormones,
- organ-system dynamics,
- metabolism,
- nervous-system arousal,
- pain,
- puberty,
- pregnancy,
- childbirth,
- aging as developmental process,
- sexual/reproductive bonding,
- physiological individuality.

#### Psychological

The current psychology is strong but still modular rather than integrated.

Missing:

- self-model,
- theory of mind,
- identity defense,
- narrative meaning-making,
- attachment system,
- developmental psychology,
- trauma as embodied and relational,
- moral self-concept,
- imagination/prospection,
- rumination,
- emotional regulation strategies,
- skill/habit formation,
- language as social action,
- cognitive style as learnable policy.

#### Relational

Current relationships have:

- trust,
- affection,
- respect,
- fear,
- obligation,
- kind,
- interaction count.

Missing:

- relationship stages,
- courtship,
- marriage,
- kinship,
- attachment dynamics,
- jealousy,
- pair-bonding,
- patron/client relations,
- master/apprentice relations,
- religious relations,
- household formation,
- clan formation,
- relational power,
- dependence,
- commitment,
- intimacy,
- betrayal,
- reconciliation,
- relational identity.

#### Cultural / Noospheric

Current culture has:

- knowledge diffusion,
- taboos,
- gossip,
- moral panic.

Missing:

- memes as evolving units,
- propaganda campaigns,
- institutional messaging,
- rituals as cohesion technology,
- collective memory,
- narrative frames,
- ideological polarization,
- echo chambers,
- status contests over meaning,
- sacred values,
- cultural identity fusion,
- large-scale belief ecology.

---

## 3. Target Vision: The Agent as an Embodied Cognitive Node

The upgraded agent should behave like a small, deterministic, embodied intelligence.

Not a literal LLM in the modern cloud sense, but an LLM-like mind in the structural sense:

> A context-sensitive inference and generation system that maintains internal representations, predicts outcomes, selects actions, learns from feedback, and produces socially meaningful behavior.

### 3.1 The Agent as Cognitive Runtime

Each agent should have an internal cognitive runtime:

```text
Perceive
  ↓
Attend
  ↓
Interpret
  ↓
Feel
  ↓
Remember
  ↓
Imagine
  ↓
Evaluate
  ↓
Choose
  ↓
Act
  ↓
Communicate
  ↓
Learn
  ↓
Revise Self-Model
```

This is the core loop of a mind-like agent.

### 3.2 The Agent as Locally Bounded Intelligence

Each agent knows only:

- its body,
- its memories,
- its relationships,
- its local perceptions,
- its rumors,
- its institutional memberships,
- its beliefs,
- its emotional state,
- its identity commitments.

This remains non-negotiable.

The world must not leak truth into the agent.

### 3.3 The Agent as Embodied System

The agent’s mind must be shaped by:

- hunger,
- fatigue,
- pain,
- hormones,
- illness,
- arousal,
- age,
- sex,
- reproductive state,
- physical strength,
- sensory acuity,
- sleep debt,
- trauma,
- developmental stage.

A stressed, exhausted, hungry agent should not think like a calm, fed, secure agent.

### 3.4 The Agent as Socially Constituted

The agent’s identity should be partially constituted by:

- relationships,
- roles,
- group memberships,
- status,
- reputation,
- obligations,
- cultural narratives,
- moral communities.

An agent should not be an isolated utility maximizer. It should be a person embedded in a social world.

---

## 4. Core Architectural Principles

### 4.1 Determinism Remains Non-Negotiable

All new systems must preserve:

- seed reproducibility,
- fixed-point math,
- deterministic RNG streams,
- replayability,
- snapshot/load integrity.

No runtime nondeterministic external AI service may be authoritative over simulation state.

### 4.2 Biology Modulates Psychology, But Does Not Replace It

Biology should create pressures and biases:

- cortisol increases threat sensitivity,
- oxytocin increases bonding,
- fatigue reduces planning horizon,
- hunger increases irritability,
- puberty increases status/sexual salience,
- pain narrows attention.

But psychology still appraises, interprets, and chooses.

### 4.3 Psychology Is Predictive, Not Just Reactive

Agents should not only react to events.

They should:

- anticipate,
- worry,
- rehearse,
- imagine,
- plan,
- ruminate,
- simulate social outcomes,
- construct narratives.

### 4.4 Relationships Are First-Class Systems

Relationships should not be just numeric edges.

They should be developmental systems with:

- stages,
- memories,
- obligations,
- attachment patterns,
- power dynamics,
- public labels,
- private feelings,
- social witnesses.

### 4.5 Culture Is an Emergent Field

Culture should not be a global variable.

It should emerge from:

- agent communication,
- institutional broadcasting,
- rituals,
- gossip,
- memory distortion,
- emotional amplification,
- identity-protective cognition,
- network topology.

### 4.6 Use Level-of-Detail Cognition

To remain performant:

- focal agents get full cognitive processing,
- background agents use simplified heuristics,
- dormant relationships update slowly,
- memes can be aggregated when not personally relevant,
- biological systems update on different timescales.

---

# 5. High-Level Target Architecture

## 5.1 Revised Layer Cake

```text
┌──────────────────────────────────────────────────────┐
│ Debug / TUI / Inspector / Replay                     │
├──────────────────────────────────────────────────────┤
│ Scenario / Experiment / Shock Layer                  │
├──────────────────────────────────────────────────────┤
│ Simulation Orchestrator                              │
├──────────────────────────────────────────────────────┤
│ Provenance / Explanation / Metrics                   │
├──────────────────────────────────────────────────────┤
│ Noospheric Layer                                     │
│ memes, rumors, propaganda, collective memory,        │
│ narrative frames, legitimacy, moral panic            │
├──────────────────────────────────────────────────────┤
│ Cultural Layer                                       │
│ practices, taboos, rituals, knowledge, education     │
├──────────────────────────────────────────────────────┤
│ Institutional Layer                                  │
│ council, temple, market, guild, household, clan      │
├──────────────────────────────────────────────────────┤
│ Social Layer                                         │
│ relationships, attachment, status, kinship, factions │
├──────────────────────────────────────────────────────┤
│ Behavioral Layer                                     │
│ utility AI, intentions, habits, skills, speech acts  │
├──────────────────────────────────────────────────────┤
│ Psychological Layer                                  │
│ self-model, theory of mind, identity, morality,      │
│ emotion regulation, narrative, imagination           │
├──────────────────────────────────────────────────────┤
│ Cognitive Layer                                      │
│ attention, memory, belief, inference, learning       │
├──────────────────────────────────────────────────────┤
│ Biological Layer                                     │
│ genome, hormones, organs, nervous system, metabolism │
├──────────────────────────────────────────────────────┤
│ World Layer                                          │
│ space, terrain, sites, resources, time, ecology      │
├──────────────────────────────────────────────────────┤
│ Core Kernel                                          │
│ ECS-like stores, RNG, fixed-point, IDs, events       │
└──────────────────────────────────────────────────────┘
```

---

## 5.2 Recommended Crate / Module Evolution

At first, implement inside `mindstrata-sim` as modules. Later split into crates if complexity grows.

### New modules under `mindstrata-sim/src/`

```text
biology/
  mod.rs
  genome.rs
  endocrine.rs
  metabolism.rs
  cardiovascular.rs
  respiratory.rs
  musculoskeletal.rs
  nervous.rs
  immune.rs
  reproductive.rs
  circadian.rs
  pain.rs
  development.rs

psychology/
  mod.rs
  cognitive_runtime.rs
  interoception.rs
  attention_v2.rs
  memory_v2.rs
  appraisal_v2.rs
  emotion_regulation.rs
  motivation.rs
  identity.rs
  self_model.rs
  theory_of_mind.rs
  moral_cognition.rs
  narrative.rs
  imagination.rs
  attachment.rs
  developmental.rs
  psychopathology.rs
  skill.rs
  habit.rs
  language.rs

social/
  mod.rs
  relationship_v2.rs
  relationship_stages.rs
  attraction.rs
  courtship.rs
  marriage.rs
  kinship.rs
  household.rs
  status.rs
  hierarchy.rs
  patronage.rs
  group_formation.rs
  clan.rs
  cult.rs
  faction_v2.rs

culture/
  mod.rs
  meme.rs
  rumor_v2.rs
  propaganda.rs
  ritual.rs
  collective_memory.rs
  narrative_frame.rs
  ideology.rs
  sacred.rs
  education.rs

noosphere/
  mod.rs
  field.rs
  belief_ecology.rs
  echo_chamber.rs
  legitimacy_field.rs
  moral_panic.rs
```

---

# 6. Multi-Timescale Simulation Model

Human systems operate on different timescales. Mindstrata should formalize this.

## 6.1 Tick Meaning

Define one tick as a fixed unit of simulated time.

Recommended:

```text
1 tick = 10 simulated minutes
144 ticks = 1 day
1008 ticks = 1 week
4320 ticks = 30-day month
51840 ticks = 360-day year
```

This gives routines, sleep, digestion, hormones, and rituals meaningful granularity.

If performance requires, use abstract tick scaling, but keep it deterministic.

## 6.2 System Frequencies

| System | Frequency |
|---|---|
| Perception / attention | every tick |
| Action execution | every tick |
| Autonomic nervous system | every tick |
| Acute emotion | every tick |
| Hunger/thirst/fatigue pressure | every tick |
| Digestion | every tick or every 2 ticks |
| Hormonal fast modulation | every 6 ticks |
| Hormonal slow regulation | daily |
| Memory consolidation | daily, especially during sleep |
| Relationship decay/growth | per interaction + daily slow pass |
| Attachment activation | event-triggered |
| Status recalibration | daily/weekly |
| Cultural meme propagation | per interaction + daily aggregate |
| Institutional propaganda | weekly or campaign-triggered |
| Developmental aging | daily/weekly |
| Puberty checks | weekly/monthly |
| Fertility cycle | daily |
| Pregnancy progression | daily |
| Childhood development | weekly/monthly |
| Ecological fertility | seasonal |

Use scheduler phases:

```text
FastPhase      // every tick
HourlyPhase    // every 6 ticks
DailyPhase     // every 144 ticks
WeeklyPhase    // every 1008 ticks
SeasonalPhase  // every season
YearlyPhase    // every year
EventPhase     // triggered by events
```

---

# 7. Biological Systems Upgrade

The biological layer should not become a medical simulator. It should be a **playable embodied substrate**.

The goal is not anatomical perfection. The goal is causal richness.

---

## 7.1 Biological Design Principles

1. **Organs as regulatory systems**, not detailed anatomy.
2. **Hormones as global modulators** of psychology and behavior.
3. **Genes as probabilistic predispositions**, not destiny.
4. **Body states feed interoception**, which feeds emotion.
5. **Development matters**: child, adolescent, adult, elder should differ deeply.
6. **Sex and reproduction are abstract, age-gated, non-explicit.**
7. **Biology creates affordances and constraints**, not scripted outcomes.

---

## 7.2 Biological Systems to Implement

### 7.2.1 Genetic System

#### Purpose

Generate heritable individuality and developmental predispositions.

#### Core Components

```rust
pub struct Genome {
    pub sex: Sex,
    pub genetic_potential: GeneticPotential,
    pub trait_predispositions: TraitPredispositions,
    pub health_predispositions: HealthPredispositions,
    pub metabolic_predispositions: MetabolicPredispositions,
    pub fertility_predispositions: FertilityPredispositions,
    pub temperament_predispositions: TemperamentPredispositions,
    pub mutation_markers: Vec<MutationMarker>,
}
```

#### Genetic influences

Genes should bias:

- baseline personality,
- stress reactivity,
- disease resistance,
- fertility,
- metabolism,
- physical strength potential,
- endurance potential,
- sensory acuity,
- attachment vulnerability,
- addiction risk,
- depression risk,
- aggression threshold,
- openness to novelty.

#### Important rule

Genes are not direct traits.

They are potentials moderated by environment:

```text
expressed_trait = genetic_potential
                × developmental_environment
                × nutrition
                × stress_history
                × cultural_shaping
                × random_epigenetic_noise
```

#### Emergent effects

- children resemble parents but are not clones,
- malnourished children develop lower physical potential,
- high-stress childhoods increase anxiety predisposition,
- noble households may produce healthier children through nutrition,
- inbreeding risk can be modeled via kinship coefficient,
- hereditary disease risk can emerge.

---

### 7.2.2 Hormonal / Endocrine System

This is one of the highest-impact biological upgrades because it connects body to mind.

#### Purpose

Hormones should modulate emotion, motivation, trust, aggression, bonding, appetite, sleep, and reproduction.

#### Abstract Hormonal Axes

Do not simulate every hormone. Simulate functional axes.

```rust
pub struct EndocrineState {
    pub stress_axis: StressAxis,       // cortisol-like
    pub bonding_axis: BondingAxis,     // oxytocin-like
    pub dominance_axis: DominanceAxis, // testosterone-like
    pub fertility_axis: FertilityAxis, // estrogen/androgen/reproductive state
    pub metabolic_axis: MetabolicAxis, // insulin/glucagon/leptin-like
    pub arousal_axis: ArousalAxis,     // adrenaline/noradrenaline-like
    pub growth_axis: GrowthAxis,       // development, repair, aging
}
```

#### Stress Axis

Inputs:

- threat appraisal,
- pain,
- hunger,
- sleep debt,
- social humiliation,
- status loss,
- uncertainty,
- trauma triggers.

Effects:

- increases fear/anger salience,
- reduces planning horizon,
- increases heuristic bias,
- increases vigilance,
- impairs memory accuracy under chronic stress,
- increases disease vulnerability,
- reduces fertility if chronic.

#### Bonding Axis

Inputs:

- positive touch,
- comforting,
- caregiving,
- sex/romance abstractly,
- childbirth,
- nursing,
- ritual synchronization,
- shared meals,
- trusted gossip.

Effects:

- increases trust,
- increases attachment activation,
- increases grief after loss,
- increases pair-bond strength,
- increases parental bonding,
- increases in-group favoritism.

#### Dominance Axis

Inputs:

- status victory,
- status defeat,
- competition,
- public respect,
- public humiliation,
- feud outcomes,
- leadership challenges.

Effects:

- modulates confidence,
- modulates aggression,
- modulates risk tolerance,
- modulates status-seeking behavior,
- affects posture/social signaling abstractly,
- affects susceptibility to submission or defiance.

#### Fertility Axis

Inputs:

- age,
- sex,
- health,
- nutrition,
- stress,
- season,
- pair-bond state,
- pregnancy,
- lactation.

Effects:

- libido,
- fertility probability,
- pregnancy progression,
- menstrual/estrus-like cycle abstracted,
- menopause/andropause,
- parental motivation.

#### Metabolic Axis

Inputs:

- food intake,
- hunger,
- body reserves,
- activity,
- illness.

Effects:

- energy availability,
- appetite,
- satiety,
- fat storage,
- weakness,
- growth,
- recovery.

#### Arousal Axis

Inputs:

- sudden events,
- danger,
- anger,
- fear,
- excitement.

Effects:

- immediate action readiness,
- heart rate analog,
- attention narrowing,
- memory flashbulb encoding,
- fight/flight/freeze bias.

#### Growth Axis

Inputs:

- age,
- nutrition,
- sleep,
- illness,
- injury.

Effects:

- childhood development,
- muscle adaptation,
- bone maturation,
- aging decline,
- wound healing.

#### Example formula

```text
stress_axis.level =
    clamp(
        stress_axis.level
        + acute_stress_input * stress_reactivity
        + chronic_stress_input * 0.1
        - recovery_rate * parasympathetic_tone,
        0,
        1
    )
```

#### Emergent effects

- chronic stress causes infertility,
- bonding after shared trauma,
- status loss causes depression risk,
- hunger causes irritability,
- sleep loss causes impulsivity,
- ritual increases collective bonding,
- parental bonding changes risk tolerance.

---

### 7.2.3 Skeletal System

#### Purpose

Represent structural body capacity, injury, aging, and physical limitation.

#### Core State

```rust
pub struct SkeletalState {
    pub frame_size: Fixed,
    pub bone_density: Fixed,
    pub structural_integrity: Fixed,
    pub fracture_risk: Fixed,
    pub chronic_pain: Fixed,
    pub mobility_penalty: Fixed,
    pub developmental_stage: SkeletalMaturity,
}
```

#### Mechanics

- childhood growth,
- adolescent maturation,
- adult peak,
- elder frailty,
- injury from combat/falls/labor,
- malnutrition reduces bone density,
- chronic injury reduces labor capacity.

#### Emergent effects

- elders become respected but physically limited,
- injured workers become dependent,
- child malnutrition produces lifelong weakness,
- combat injuries create chronic pain and bitterness.

---

### 7.2.4 Muscular System

#### Purpose

Represent strength, endurance, fatigue, adaptation, and physical agency.

#### Core State

```rust
pub struct MuscularState {
    pub strength: Fixed,
    pub endurance: Fixed,
    pub fatigue: Fixed,
    pub soreness: Fixed,
    pub atrophy: Fixed,
    pub conditioning: Fixed,
    pub injury: Fixed,
}
```

#### Mechanics

- labor increases conditioning,
- overwork increases fatigue/injury,
- rest restores,
- starvation causes atrophy,
- age reduces peak strength,
- training improves efficiency.

#### Behavioral effects

- carrying capacity,
- work output,
- combat effectiveness,
- migration ability,
- escape ability,
- intimidation potential.

---

### 7.2.5 Neurological / Nervous System

This is central to making agents feel alive.

#### Purpose

Represent arousal, autonomic regulation, sensory processing, pain, trauma, and neuroplasticity.

#### Core State

```rust
pub struct NervousSystemState {
    pub sympathetic_arousal: Fixed,
    pub parasympathetic_tone: Fixed,
    pub baseline_arousal: Fixed,
    pub sensory_acuity: SensoryAcuity,
    pub pain_level: Fixed,
    pub pain_sensitivity: Fixed,
    pub trauma_load: Fixed,
    pub dissociation_risk: Fixed,
    pub startle_sensitivity: Fixed,
    pub neuroplasticity: Fixed,
    pub sleep_pressure: Fixed,
    pub circadian_phase: Fixed,
}
```

#### Autonomic Model

Use a simplified sympathetic/parasympathetic balance:

```text
arousal = sympathetic_activation - parasympathetic_recovery
```

High sympathetic arousal:

- narrows attention,
- increases threat appraisal,
- increases aggression/fear,
- reduces empathy,
- reduces long-term planning.

High parasympathetic tone:

- enables social engagement,
- improves trust,
- improves digestion,
- improves sleep,
- improves emotional regulation.

#### Pain

Pain should be more than injury.

```rust
pub struct PainState {
    pub acute_pain: Fixed,
    pub chronic_pain: Fixed,
    pub pain_attention: Fixed,
    pub pain_meaning: PainMeaning,
}
```

Pain meaning matters:

- pain from battle may produce pride,
- pain from punishment may produce resentment,
- pain from childbirth may produce bonding/trauma,
- chronic pain may produce bitterness/depression.

#### Trauma

Trauma should be embodied:

```text
trauma_load = repeated_high_stress_events
            + insufficient_social_support
            + low_coping_potential
            + developmental_vulnerability
```

Trauma effects:

- hypervigilance,
- triggers,
- nightmares,
- emotional flashbacks,
- avoidance,
- aggression,
- dissociation,
- attachment anxiety/avoidance.

#### Emergent effects

- abused children become anxious or avoidant,
- veterans startle easily,
- chronic pain makes agents irritable,
- sleep-deprived agents become paranoid,
- calm agents recover socially faster.

---

### 7.2.6 Sexual / Reproductive System

This must be handled carefully, abstractly, and age-appropriately.

#### Purpose

Represent puberty, fertility, attraction, pair-bonding, reproduction, pregnancy, birth, parental investment.

#### Core State

```rust
pub struct ReproductiveState {
    pub sex: Sex,
    pub sexual_maturity: SexualMaturity,
    pub puberty_stage: PubertyStage,
    pub fertility: Fixed,
    pub libido: Fixed,
    pub attraction_orientation: AttractionOrientation,
    pub pair_bond_strength: Fixed,
    pub pregnancy: Option<PregnancyState>,
    pub lactation: Option<Fixed>,
    pub parental_drive: Fixed,
    pub reproductive_history: ReproductiveHistory,
}
```

#### Rules

- no sexual content involving minors,
- romantic/reproductive behavior only for adults,
- sexual activity is abstract/fade-to-black,
- consent and social norms matter,
- reproduction is probabilistic,
- pregnancy is a biological and social event.

#### Puberty

Puberty should trigger:

- fertility axis activation,
- status sensitivity,
- attraction salience,
- identity experimentation,
- peer sensitivity,
- risk-taking.

#### Attraction

Attraction should be multi-factor:

```text
attraction =
    physical_compatibility
  + personality_compatibility
  + status_admiration
  + familiarity
  + reciprocity
  + social_approval
  + attachment_resonance
  + fertility_compatibility
  + proximity
  + novelty
  - kinship_penalty
  - moral_disgust
  - social_cost
```

#### Pair-Bonding

Pair-bonding emerges from:

- repeated positive interaction,
- intimacy,
- sexual/romantic exclusivity abstractly,
- shared vulnerability,
- social recognition,
- cohabitation,
- children,
- economic cooperation.

#### Pregnancy

```rust
pub struct PregnancyState {
    pub father: Option<AgentId>,
    pub conception_tick: u64,
    pub gestation_progress: Fixed,
    pub health_risk: Fixed,
    pub nutrition_demand: Fixed,
    pub emotional_state: Fixed,
    pub social_recognition: Fixed,
}
```

Pregnancy affects:

- metabolism,
- fatigue,
- vulnerability,
- social treatment,
- household economics,
- pair-bond dynamics,
- inheritance politics.

#### Birth

Birth can trigger:

- infant entity,
- maternal bonding,
- paternal bonding,
- kinship network expansion,
- inheritance claims,
- status changes,
- health crisis,
- death risk,
- ritual naming.

#### Emergent effects

- courtship,
- jealousy,
- illegitimacy conflicts,
- marriage politics,
- fertility anxiety,
- parental favoritism,
- widowhood,
- inheritance disputes,
- clan formation through children.

---

### 7.2.7 Digestive System

#### Purpose

Turn eating from a need decrement into a biological process.

#### Core State

```rust
pub struct DigestiveState {
    pub stomach_contents: Fixed,
    pub nutrient_absorption: Fixed,
    pub gut_health: Fixed,
    pub food_safety: Fixed,
    pub nausea: Fixed,
    pub satiety: Fixed,
    pub cravings: Vec<Craving>,
    pub microbiome_health: Fixed, // optional, abstract
}
```

#### Mechanics

- food takes time to digest,
- spoiled food causes illness,
- poor diet causes malnutrition even with calories,
- stress impairs digestion,
- shared meals increase bonding,
- fasting changes mood and cognition.

#### Emergent effects

- food poisoning spreads through a feast,
- famine changes personality,
- communal eating builds trust,
- dietary taboos become identity markers,
- hunger riots emerge from scarcity.

---

### 7.2.8 Respiratory System

#### Purpose

Represent exertion limits, environmental health, and disease vulnerability.

#### Core State

```rust
pub struct RespiratoryState {
    pub oxygenation: Fixed,
    pub breathlessness: Fixed,
    pub lung_health: Fixed,
    pub irritation: Fixed,
    pub disease_load: Fixed,
    pub endurance_modifier: Fixed,
}
```

#### Inputs

- exertion,
- smoke,
- cold,
- damp housing,
- epidemic disease,
- age,
- injury.

#### Emergent effects

- winter illness,
- labor exhaustion,
- elderly respiratory decline,
- epidemic spread through crowded sites,
- poor housing harms health.

---

### 7.2.9 Cardiovascular System

#### Purpose

Represent stamina, shock, blood loss, fitness, and acute mortality risk.

#### Core State

```rust
pub struct CardiovascularState {
    pub cardiac_capacity: Fixed,
    pub circulation: Fixed,
    pub blood_volume: Fixed,
    pub shock_risk: Fixed,
    pub fitness: Fixed,
    pub blood_pressure_analog: Fixed,
    pub recovery_rate: Fixed,
}
```

#### Mechanics

- fitness improves with healthy labor,
- overexertion under starvation causes collapse,
- combat injury can cause blood loss,
- chronic stress raises long-term risk,
- aging reduces capacity.

#### Emergent effects

- elders die from shock/injury more easily,
- soldiers with high fitness survive combat,
- famine causes collapse during labor,
- fear can cause flight or freeze.

---

## 7.3 Additional Biological Cross-Systems

Although not explicitly requested, these are necessary for coherence.

### 7.3.1 Immune System

```rust
pub struct ImmuneState {
    pub resistance: Fixed,
    pub inflammation: Fixed,
    pub infection_load: Fixed,
    pub recovery_capacity: Fixed,
    pub autoimmune_risk: Fixed,
}
```

Immunity interacts with:

- nutrition,
- stress,
- sleep,
- age,
- injury,
- hygiene,
- crowding.

### 7.3.2 Circadian System

```rust
pub struct CircadianState {
    pub phase: Fixed,
    pub sleep_pressure: Fixed,
    pub rhythm_stability: Fixed,
    pub chronotype: Chronotype,
}
```

Effects:

- morning/evening preferences,
- sleep deprivation,
- irritability,
- ritual timing,
- work schedules.

### 7.3.3 Thermoregulation

```rust
pub struct ThermalState {
    pub body_temperature: Fixed,
    pub cold_stress: Fixed,
    pub heat_stress: Fixed,
}
```

Effects:

- winter hardship,
- housing quality,
- clothing needs,
- migration pressure.

---

## 7.4 Integrated Body Architecture

Replace the current `BodyState` with a richer structure, but preserve a compatibility facade.

```rust
pub struct EmbodiedState {
    pub genome: Genome,
    pub endocrine: EndocrineState,
    pub nervous: NervousSystemState,
    pub skeletal: SkeletalState,
    pub muscular: MuscularState,
    pub cardiovascular: CardiovascularState,
    pub respiratory: RespiratoryState,
    pub digestive: DigestiveState,
    pub immune: ImmuneState,
    pub reproductive: ReproductiveState,
    pub circadian: CircadianState,
    pub thermal: ThermalState,
    pub pain: PainState,
    pub metabolic: MetabolicState,
}
```

Compatibility:

```rust
impl EmbodiedState {
    pub fn derived_health(&self) -> Fixed { ... }
    pub fn derived_energy(&self) -> Fixed { ... }
    pub fn derived_hunger(&self) -> Fixed { ... }
    pub fn derived_thirst(&self) -> Fixed { ... }
    pub fn derived_fatigue(&self) -> Fixed { ... }
    pub fn derived_sickness(&self) -> Fixed { ... }
    pub fn derived_injury(&self) -> Fixed { ... }
    pub fn derived_fertility(&self) -> Option<Fixed> { ... }
}
```

This allows old systems to keep working while new biology is introduced.

---

# 8. Psychological Systems Upgrade

The psychological layer should become as rich as the biological layer.

The agent should not merely have “traits + needs + emotions.”

It should have a structured mind.

---

## 8.1 Psychological Systems to Implement

Below is an exhaustive brainstorm of psychological systems suitable for human-scale simulation.

---

### 8.1.1 Interoceptive System

#### Purpose

Translate body state into felt experience.

#### Variables

- hunger awareness,
- thirst awareness,
- fatigue awareness,
- pain awareness,
- arousal awareness,
- nausea,
- sexual arousal,
- emotional bodily tone.

#### Function

Interoception connects biology to affect.

```text
felt_state = body_signals
           × attention_to_body
           × emotional_sensitivity
           × cultural_interpretation
```

#### Emergent effects

- anxious agents overinterpret bodily signals,
- stoic agents ignore injury,
- hunger becomes anger,
- fatigue becomes despair,
- arousal becomes attraction or shame.

---

### 8.1.2 Perception and Attention System

Upgrade current attention into a full cognitive gateway.

#### Components

```rust
pub struct PerceptionState {
    pub sensory_input: Vec<Percept>,
    pub salience_map: Vec<SalientItem>,
    pub focus: Option<FocusTarget>,
    pub attention_capacity: Fixed,
    pub habituation: HashMap<PerceptKind, Fixed>,
    pub threat_bias: Fixed,
    pub social_bias: Fixed,
    pub novelty_bias: Fixed,
}
```

#### Mechanics

- salience competition,
- emotional bias,
- expectation tuning,
- inattentional blindness,
- trauma-triggered hypervigilance,
- cultural salience.

#### Emergent effects

- agents miss warnings if habituated,
- fearful agents notice threats,
- lovers notice each other,
- priests notice sacrilege,
- merchants notice scarcity.

---

### 8.1.3 Learning and Memory System

Expand current memory into multiple memory systems.

#### Memory Types

```rust
pub enum MemoryKind {
    Episodic,
    Semantic,
    Procedural,
    Social,
    Emotional,
    Flashbulb,
    Traumatic,
    Cultural,
    Somatic,
}
```

#### Memory Properties

```rust
pub struct MemoryTrace {
    pub id: MemoryId,
    pub kind: MemoryKind,
    pub content: MemoryContent,
    pub strength: Fixed,
    pub emotional_charge: Fixed,
    pub sensory_richness: Fixed,
    pub accuracy: Fixed,
    valence: Fixed,
    pub identity_relevance: Fixed,
    pub social_sharedness: Fixed,
    pub last_rehearsed: u64,
    pub distortion_history: Vec<DistortionEvent>,
}
```

#### Mechanics

- encoding depends on attention and emotion,
- sleep consolidates memory,
- rehearsal strengthens,
- retrieval reconstructs,
- emotion distorts,
- identity protects,
- social retelling mutates,
- trauma memories become intrusive or fragmented.

#### Emergent effects

- rumors alter memories,
- rituals reinforce cultural memory,
- traumatic events flash back,
- elders preserve traditions,
- false memories can spread.

---

### 8.1.4 Appraisal and Emotion System

Keep Lazarus appraisal, but deepen it.

#### Expanded Appraisal Dimensions

Add:

- moral violation,
- sacredness violation,
- identity threat,
- attachment threat,
- status threat,
- purity/disgust,
- agency attribution,
- controllability,
- future implication,
- social audience,
- narrative meaning.

#### Emotion Families

Expand beyond 8 emotions:

- fear,
- anger,
- joy,
- sadness,
- trust,
- shame,
- guilt,
- pride,
- disgust,
- contempt,
- awe,
- gratitude,
- jealousy,
- envy,
- loneliness,
- tenderness,
- humiliation,
- relief,
- hope,
- despair,
- nostalgia,
- moral outrage.

#### Emotion Regulation Strategies

Agents should not just feel emotions; they should regulate them.

```rust
pub enum EmotionRegulationStrategy {
    Reappraisal,
    Suppression,
    Rumination,
    Avoidance,
    SeekingSupport,
    Prayer,
    Ritual,
    Aggression,
    Humor,
    Dissociation,
    SubstanceUse, // if world has alcohol/herbs
    Work,
    Caregiving,
}
```

Regulation success depends on:

- personality,
- stress,
- social support,
- culture,
- skill,
- exhaustion.

---

### 8.1.5 Motivation and Need System

Expand current 8 needs into a layered motivation architecture.

#### Biological Needs

- hunger,
- thirst,
- sleep,
- warmth,
- health,
- safety.

#### Psychological Needs

- attachment,
- belonging,
- esteem,
- autonomy,
- competence,
- meaning,
- certainty,
- novelty,
- play,
- care,
- sexuality/romance abstractly,
- justice,
- recognition.

#### Motivation Dynamics

```text
motivation_pressure =
    need_deficit
  × personality_weight
  × emotional_amplification
  × cultural_legitimacy
  × situational_affordance
```

---

### 8.1.6 Personality and Temperament System

Keep 12 traits, but add temperament and state-trait dynamics.

#### Temperament

More biologically rooted:

- reactivity,
- soothability,
- sociability,
- persistence,
- sensitivity,
- rhythm/regularity,
- approach/withdrawal.

#### Trait Plasticity

Traits should slowly change through:

- repeated behavior,
- trauma,
- success,
- failure,
- roles,
- aging,
- relationships,
- institutions.

Example:

```text
trait_change =
    repeated_state_expression
  × identity_integration
  × social_reinforcement
  × developmental_plasticity
```

---

### 8.1.7 Identity and Self-Model System

This is essential for “mind-like” agents.

#### Core Structure

```rust
pub struct SelfModel {
    pub self_concept: Vec<IdentityClaim>,
    pub roles: Vec<RoleIdentity>,
    pub values: Vec<ValueCommitment>,
    pub sacred_values: Vec<SacredValue>,
    pub self_esteem: Fixed,
    pub self_coherence: Fixed,
    pub identity_security: Fixed,
    pub shame_proneness: Fixed,
    pub guilt_proneness: Fixed,
    pub narrative_identity: NarrativeIdentity,
}
```

#### Identity Claims

Examples:

- “I am a farmer.”
- “I am a faithful person.”
- “I am a protector.”
- “I am respected.”
- “I am betrayed.”
- “I am a mother.”
- “I am a leader.”
- “I am powerless.”

#### Identity Defense

Beliefs linked to identity resist change:

```text
belief_resistance =
    base_resistance
  + identity_linkage
  + social_reinforcement
  + emotional_charge
  + sacredness
```

#### Emergent effects

- cognitive dissonance,
- hypocrisy avoidance,
- moral injury,
- conversion experiences,
- radicalization,
- repentance,
- prideful defiance.

---

### 8.1.8 Belief and Epistemic System

Upgrade current belief system into an epistemic mind.

#### Components

```rust
pub struct EpistemicState {
    pub beliefs: Vec<Belief>,
    pub trust_network: TrustNetwork,
    pub epistemic_style: EpistemicStyle,
    pub curiosity: Fixed,
    pub dogmatism: Fixed,
    pub conspiracy_susceptibility: Fixed,
    pub source_monitoring: Fixed,
    pub need_for_closure: Fixed,
}
```

#### Epistemic Styles

- empirical,
- traditional,
- authoritative,
- conspiratorial,
- pragmatic,
- mystical,
- skeptical,
- gullible.

#### Belief Dynamics

Beliefs update through:

- direct evidence,
- trusted testimony,
- institutional authority,
- emotional fit,
- identity protection,
- narrative coherence,
- repeated exposure,
- ritual reinforcement.

---

### 8.1.9 Theory of Mind System

Agents should model other minds.

#### Structure

```rust
pub struct OtherModel {
    pub target: AgentId,
    pub perceived_traits: Personality,
    pub perceived_goals: Vec<GoalHypothesis>,
    pub perceived_beliefs: Vec<BeliefHypothesis>,
    pub perceived_emotions: DiscreteEmotions,
    pub perceived_intent: IntentHypothesis,
    pub trust_model: TrustModel,
    pub threat_model: ThreatModel,
    pub empathy: Fixed,
    pub projection_bias: Fixed,
}
```

#### Mechanics

Agents infer:

- what others know,
- what others want,
- what others feel,
- whether others are hostile,
- whether others are lying,
- whether others share identity.

Errors are important:

- misattribution,
- paranoia,
- projection,
- idealization,
- dehumanization,
- false consensus.

---

### 8.1.10 Moral Cognition System

#### Components

```rust
pub struct MoralCognition {
    pub moral_foundations: MoralFoundations,
    pub internalized_norms: Vec<InternalizedNorm>,
    pub moral_emotions: MoralEmotions,
    pub moral_identity: Fixed,
    pub hypocrisy_sensitivity: Fixed,
    pub purity_sensitivity: Fixed,
    pub fairness_sensitivity: Fixed,
    pub loyalty_sensitivity: Fixed,
    pub authority_sensitivity: Fixed,
}
```

#### Moral Foundations

- care/harm,
- fairness/cheating,
- loyalty/betrayal,
- authority/subversion,
- purity/degradation,
- liberty/oppression.

#### Emergent effects

- moral outrage,
- scapegoating,
- martyrdom,
- reform movements,
- religious schisms,
- honor feuds.

---

### 8.1.11 Language and Communication System

Language should be action.

#### Speech Acts

- inform,
- request,
- command,
- promise,
- threaten,
- apologize,
- praise,
- insult,
- gossip,
- confess,
- bless,
- curse,
- persuade,
- accuse,
- deny,
- reassure,
- flirt,
- propose,
- vow,
- excommunicate.

#### Communication Model

```rust
pub struct SpeechAct {
    pub speaker: AgentId,
    pub listener: AgentId,
    pub audience: Vec<AgentId>,
    pub act: SpeechActKind,
    pub content: PropositionRef,
    pub emotional_tone: Fixed,
    pub credibility: Fixed,
    pub social_cost: Fixed,
    pub relational_intent: RelationalIntent,
}
```

#### Effects

Speech changes:

- beliefs,
- relationships,
- status,
- emotions,
- obligations,
- reputation,
- group cohesion.

---

### 8.1.12 Executive Function and Metacognition System

#### Components

```rust
pub struct ExecutiveState {
    pub working_memory_capacity: Fixed,
    pub inhibition: Fixed,
    pub cognitive_flexibility: Fixed,
    pub planning_depth: u32,
    pub impulsivity: Fixed,
    pub self_monitoring: Fixed,
    pub error_sensitivity: Fixed,
}
```

#### Mechanics

Stress, fatigue, pain, and trauma reduce executive function.

High executive function enables:

- long-term planning,
- deception,
- diplomacy,
- skill mastery,
- emotional regulation,
- institutional leadership.

---

### 8.1.13 Developmental Psychology System

Agents should develop across life.

#### Life Stages

- infant,
- child,
- adolescent,
- young adult,
- adult,
- mature adult,
- elder.

#### Developmental Processes

- attachment formation,
- temperament expression,
- socialization,
- identity formation,
- moral development,
- skill acquisition,
- puberty,
- pair-bonding readiness,
- generativity,
- aging/decline.

#### Childhood Environment

```rust
pub struct DevelopmentalHistory {
    pub caregiver_security: Fixed,
    pub nutrition_history: Fixed,
    pub trauma_history: Fixed,
    pub socialization_style: SocializationStyle,
    pub education_history: Vec<EducationEvent>,
    pub role_models: Vec<AgentId>,
    pub cultural_imprinting: Vec<CulturalImprint>,
}
```

---

### 8.1.14 Attachment System

This is central to relationships.

#### Attachment Styles

```rust
pub enum AttachmentStyle {
    Secure,
    Anxious,
    Avoidant,
    Disorganized,
}
```

#### Variables

```rust
pub struct AttachmentSystem {
    pub style: AttachmentStyle,
    pub security: Fixed,
    pub anxiety: Fixed,
    pub avoidance: Fixed,
    pub protest_threshold: Fixed,
    pub soothing_receptivity: Fixed,
    pub separation_distress: Fixed,
    pub caregiving_style: CaregivingStyle,
}
```

#### Dynamics

Under threat:

- secure agents seek support and recover,
- anxious agents cling and demand reassurance,
- avoidant agents withdraw and self-regulate,
- disorganized agents oscillate or freeze.

Attachment affects:

- friendship,
- romance,
- marriage,
- parenting,
- faction loyalty,
- religious devotion,
- leader dependence.

---

### 8.1.15 Psychopathology / Mental Health System

Not as labels, but as risk dynamics.

#### Risk States

- depression,
- anxiety,
- PTSD,
- paranoia,
- addiction,
- compulsive behavior,
- dissociation,
- grief pathology,
- resentment syndrome.

#### Causal Inputs

- chronic stress,
- trauma,
- isolation,
- humiliation,
- loss,
- moral injury,
- chronic pain,
- sleep deprivation,
- addiction substances,
- genetic vulnerability.

#### Effects

- reduced agency,
- distorted appraisal,
- relationship damage,
- institutional withdrawal,
- radicalization vulnerability.

---

### 8.1.16 Imagination and Prospection System

Agents should simulate possible futures.

```rust
pub struct ProspectionState {
    pub scenarios: Vec<MentalScenario>,
    pub hope: Fixed,
    pub dread: Fixed,
    pub optimism_bias: Fixed,
    pub catastrophic_bias: Fixed,
    pub planning_confidence: Fixed,
}
```

#### Mental Scenarios

- “If I complain, the council may punish me.”
- “If I marry her, my clan gains land.”
- “If the harvest fails, my child dies.”
- “If I join the protest, I may gain respect.”

Prospection is biased by emotion:

- fear amplifies dread,
- ambition amplifies hope,
- trauma amplifies catastrophe,
- depression reduces hope.

---

### 8.1.17 Narrative and Meaning-Making System

Agents should interpret life as a story.

```rust
pub struct NarrativeIdentity {
    pub life_theme: LifeTheme,
    pub redemption_script: Fixed,
    pub contamination_script: Fixed,
    pub victimhood_script: Fixed,
    pub heroism_script: Fixed,
    pub chosenness_script: Fixed,
    pub shame_script: Fixed,
}
```

#### Meaning-Making

Events are interpreted through narrative frames:

- punishment as justice,
- suffering as test,
- loss as curse,
- success as blessing,
- betrayal as proof of unworthiness,
- survival as destiny.

This feeds religion, ideology, and resilience.

---

### 8.1.18 Cultural Cognition System

Agents should think through cultural categories.

#### Components

- categories,
- prototypes,
- taboos,
- honor codes,
- purity maps,
- ritual scripts,
- mythic templates.

#### Effects

- outgroup disgust,
- sacred boundaries,
- ritual obedience,
- cultural creativity,
- syncretism,
- heresy.

---

### 8.1.19 Skill and Habit System

```rust
pub struct SkillState {
    pub skills: HashMap<SkillId, SkillLevel>,
    pub habits: Vec<Habit>,
    pub automaticity: Fixed,
}
```

Skills:

- farming,
- cooking,
- healing,
- trading,
- fighting,
- speaking,
- leadership,
- ritual,
- crafting,
- parenting,
- deception,
- diplomacy.

Habits form through repetition and stress.

Under stress, agents fall back on habits.

---

### 8.1.20 Decision Policy System

This integrates all psychology into action.

```rust
pub struct DecisionPolicy {
    pub utility_weights: UtilityWeights,
    pub moral_weights: MoralWeights,
    pub risk_policy: RiskPolicy,
    pub social_policy: SocialPolicy,
    pub habit_policy: HabitPolicy,
    pub emotional_policy: EmotionalPolicy,
}
```

The policy itself can learn.

This makes the agent feel like an adaptive intelligence rather than a static utility function.

---

# 9. The Agent as an LLM-Like Mind

The user’s phrase “each human node must itself become LIKE an AI/ML Model, or an LLM” is architecturally powerful if interpreted correctly.

The agent should not necessarily run a literal large language model. That would be expensive, nondeterministic, and brittle.

Instead, each agent should have an **LLM-like cognitive architecture**:

- context assembly,
- representation,
- inference,
- generation,
- evaluation,
- learning,
- self-reflection.

---

## 9.1 Cognitive Runtime Architecture

```text
┌─────────────────────────────────────────────┐
│ Context Assembly                            │
│ percepts, memories, goals, relationships,   │
│ body state, cultural frames                 │
├─────────────────────────────────────────────┤
│ Representation Layer                        │
│ propositions, concept vectors, scripts,     │
│ self-model, other-models                    │
├─────────────────────────────────────────────┤
│ Inference Layer                             │
│ spreading activation, belief updating,      │
│ causal attribution, theory of mind          │
├─────────────────────────────────────────────┤
│ Affective Evaluation                        │
│ appraisal, emotion, moral feeling           │
├─────────────────────────────────────────────┤
│ Prospective Simulation                      │
│ imagine candidate actions and outcomes      │
├─────────────────────────────────────────────┤
│ Policy / Decision Layer                     │
│ utility + morality + identity + habit       │
├─────────────────────────────────────────────┤
│ Action / Speech Generation                  │
│ action grammar, speech acts                 │
├─────────────────────────────────────────────┤
│ Learning Layer                              │
│ update memory, beliefs, skills, traits      │
├─────────────────────────────────────────────┤
│ Self-Reflection Layer                       │
│ narrative update, identity maintenance      │
└─────────────────────────────────────────────┘
```

---

## 9.2 Deterministic “Neural-Like” Implementation

Use lightweight deterministic mechanisms:

### Concept embeddings

Each concept/proposition can have a low-dimensional fixed-point vector:

```text
concept_vector: [
  safety,
  sacredness,
  status,
  kinship,
  scarcity,
  threat,
  purity,
  freedom,
  loyalty,
  pleasure,
  shame,
  hope,
]
```

Similarity:

```text
similarity(a, b) = dot(a, b) / (norm(a) * norm(b))
```

### Spreading activation

When an agent perceives something:

```text
activation(node) += input_strength
for edge in associations:
    activation(target) += activation(source) * edge_weight * decay
```

This produces associative thought.

### Predictive error

Agents maintain expectations:

```text
prediction_error = observed_outcome - expected_outcome
```

Large prediction error:

- increases attention,
- updates beliefs,
- creates emotional intensity,
- may trigger trauma or insight.

### Reinforcement learning values

Actions have learned values:

```text
action_value(action, context) =
    expected_need_relief
  + expected_emotional_relief
  + expected_social_reward
  + expected_identity_congruence
  - expected_cost
  - expected_risk
```

Values update from outcomes.

### Script grammar

Agents generate behavior from scripts:

```text
CourtshipScript:
  notice
  appraise
  signal interest
  seek proximity
  test reciprocity
  gossip validation
  propose bond
  seek social approval
  formalize
```

This gives LLM-like generativity without nondeterminism.

---

## 9.3 Optional External LLM Role

An actual LLM may be used only as an auxiliary:

- generate readable event summaries,
- author scenarios,
- explain agent psychology,
- produce debug natural-language traces,
- assist designers.

It must not determine simulation truth unless:

- weights are fixed,
- inference is deterministic,
- outputs are validated,
- all randomness is seeded,
- all results are replayable.

Recommended rule:

> **LLM is interpreter, not oracle.**

---

# 10. Relational Systems Upgrade

Human life emerges in the space between persons.

Mindstrata should model this space as a first-class simulation domain.

---

## 10.1 Three Relational Fields

When agents enter one another’s fields, three layers activate:

### 10.1.1 Sensory Field

What agents can perceive:

- sight,
- sound,
- proximity,
- expression,
- gesture,
- smell abstractly,
- environmental cues.

### 10.1.2 Social Field

Relationship graph:

- trust,
- status,
- obligation,
- reputation,
- kinship,
- faction,
- role.

### 10.1.3 Noospheric Field

Shared symbolic space:

- rumors,
- memes,
- beliefs,
- narratives,
- sacred symbols,
- legitimacy,
- collective emotions.

---

## 10.2 Relationship Model v2

Replace simple relationship edges with rich relational systems.

```rust
pub struct RelationshipV2 {
    pub from: AgentId,
    pub to: AgentId,

    // Core affect
    pub trust: Fixed,
    pub affection: Fixed,
    pub respect: Fixed,
    pub fear: Fixed,
    pub obligation: Fixed,

    // Deep relational dimensions
    pub intimacy: Fixed,
    pub passion: Fixed,
    pub commitment: Fixed,
    pub attachment_security: Fixed,
    pub dependence: Fixed,
    pub power_balance: Fixed,
    pub solidarity: Fixed,
    pub resentment: Fixed,
    pub admiration: Fixed,
    pub jealousy: Fixed,
    pub gratitude: Fixed,
    pub guilt_toward: Fixed,
    pub moral_debt: Fixed,

    // Identity and meaning
    pub shared_identity: Fixed,
    pub role_expectation: RoleExpectation,
    pub public_label: RelationshipLabel,
    pub private_label: RelationshipLabel,

    // Structure
    pub kinship_coefficient: Fixed,
    pub household_link: Option<HouseholdId>,
    pub faction_link: Option<FactionId>,
    pub institutional_link: Option<InstitutionId>,

    // History
    pub stage: RelationshipStage,
    pub stage_progress: Fixed,
    pub interaction_count: u32,
    pub positive_memory_weight: Fixed,
    pub negative_memory_weight: Fixed,
    pub betrayal_history: Vec<BetrayalEvent>,
    pub reconciliation_history: Vec<ReconciliationEvent>,

    // Dynamics
    pub last_positive_tick: u64,
    pub last_negative_tick: u64,
    pub decay_rate: Fixed,
    pub volatility: Fixed,
}
```

---

## 10.3 Relationship Stages

### General Social Stages

```text
Unnoticed
  ↓
Noticed
  ↓
Acquaintance
  ↓
Familiar
  ↓
Neighbor
  ↓
Friend
  ↓
Close Friend
  ↓
Confidant
  ↓
Ally
```

Negative branch:

```text
Disliked
  ↓
Rival
  ↓
Enemy
  ↓
Nemesis
```

Authority branch:

```text
Patron / Client
Lord / Vassal
Master / Apprentice
Priest / Layperson
Elder / Junior
Guard / Citizen
```

Kin branch:

```text
Kin
Parent / Child
Sibling
Cousin
In-Law
Ancestor / Descendant
```

---

## 10.4 Intersexual and Romantic Relationship System

This should be abstract, socially embedded, and age-gated.

### Romantic Stages

```text
Awareness
  ↓
Attraction
  ↓
Flirtation
  ↓
Courtship
  ↓
Exclusivity
  ↓
Betrothal
  ↓
Marriage
  ↓
Household Formation
  ↓
Parenthood
  ↓
Stabilization / Strain
  ↓
Reconciliation / Separation / Widowhood
```

### Attraction System

```rust
pub struct AttractionModel {
    pub physical_attraction: Fixed,
    pub personality_attraction: Fixed,
    pub status_attraction: Fixed,
    pub moral_attraction: Fixed,
    pub familiarity: Fixed,
    pub reciprocity: Fixed,
    pub social_approval: Fixed,
    pub kinship_penalty: Fixed,
    pub taboo_penalty: Fixed,
    pub attachment_resonance: Fixed,
}
```

### Pair-Bond

```rust
pub struct PairBond {
    pub partner_a: AgentId,
    pub partner_b: AgentId,
    pub bond_strength: Fixed,
    pub exclusivity: Fixed,
    pub public_recognition: Fixed,
    pub sexual_intimacy_abstract: Fixed,
    pub emotional_intimacy: Fixed,
    pub economic_cooperation: Fixed,
    pub shared_children: Vec<AgentId>,
    pub strain: Fixed,
    pub jealousy_load: Fixed,
}
```

### Jealousy

Jealousy emerges from:

- attachment anxiety,
- dependence,
- status threat,
- fear of abandonment,
- rival attraction,
- public humiliation.

Effects:

- surveillance,
- accusation,
- violence,
- gossip,
- reconciliation,
- breakup.

---

## 10.5 Marriage System

Marriage is not just a relationship label. It is an institution.

```rust
pub struct Marriage {
    pub partners: Vec<AgentId>,
    pub marriage_type: MarriageType,
    pub legitimacy: Fixed,
    pub household: Option<HouseholdId>,
    pub kin_alliance: Vec<KinLink>,
    pub property_arrangement: PropertyArrangement,
    pub religious_sanction: Fixed,
    pub community_recognition: Fixed,
    pub children: Vec<AgentId>,
    pub vows: Vec<Vow>,
    pub strain: Fixed,
}
```

Marriage affects:

- inheritance,
- household economics,
- faction alliances,
- legitimacy of children,
- status,
- sexual norms,
- religious standing,
- kin networks.

Marriage dissolution can occur through:

- death,
- abandonment,
- annulment,
- divorce,
- exile,
- religious sanction.

---

## 10.6 Kinship System

Kinship should be a network, not a label.

```rust
pub struct KinshipGraph {
    pub biological_links: Vec<KinLink>,
    pub marriage_links: Vec<KinLink>,
    pub adoption_links: Vec<KinLink>,
    pub ritual_links: Vec<KinLink>, // godparent, oath-sworn
}
```

Kinship affects:

- trust baseline,
- obligation,
- inheritance,
- feud participation,
- marriage restrictions,
- household formation,
- clan identity.

---

## 10.7 Household System

Households become primary social units.

```rust
pub struct Household {
    pub id: HouseholdId,
    pub members: Vec<AgentId>,
    pub head: Option<AgentId>,
    pub residence: SiteId,
    pub pooled_resources: ResourcePool,
    pub roles: Vec<HouseholdRole>,
    pub cohesion: Fixed,
    pub conflict: Fixed,
    pub reputation: Fixed,
    pub traditions: Vec<PracticeId>,
}
```

Household dynamics:

- resource pooling,
- childcare,
- elder care,
- domestic conflict,
- inheritance,
- hospitality,
- shame/pride.

---

## 10.8 Clan System

Clans emerge from kinship plus narrative plus repeated alliance.

```rust
pub struct Clan {
    pub id: ClanId,
    pub core_households: Vec<HouseholdId>,
    pub founder_memory: Option<AgentId>,
    pub shared_ancestor: Option<AgentId>,
    pub prestige: Fixed,
    pub honor_code: Vec<NormId>,
    pub enemies: Vec<ClanId>,
    pub allies: Vec<ClanId>,
    pub myths: Vec<MemeId>,
    pub cohesion: Fixed,
}
```

Clan emergence:

- marriages connect households,
- shared grievances create solidarity,
- feuds create boundaries,
- myths justify identity,
- elders preserve honor codes.

---

# 11. Power, Status, and Hierarchy Systems

Relationships are never free of power.

Mindstrata should model power as multi-dimensional.

---

## 11.1 Status Components

```rust
pub struct StatusState {
    pub dominance: Fixed,
    pub prestige: Fixed,
    pub authority: Fixed,
    pub legitimacy: Fixed,
    pub wealth_rank: Fixed,
    pub moral_reputation: Fixed,
    pub network_centrality: Fixed,
    pub institutional_rank: Fixed,
    pub honor: Fixed,
    pub shame: Fixed,
}
```

### Dominance

Power through fear/coercion.

### Prestige

Power through admiration/competence.

### Authority

Power through institutional role.

### Legitimacy

Power through perceived rightfulness.

---

## 11.2 Relational Power

For each relationship:

```rust
pub struct RelationalPower {
    pub dependence_a_on_b: Fixed,
    pub dependence_b_on_a: Fixed,
    pub resource_control: Fixed,
    pub emotional_leverage: Fixed,
    pub social_leverage: Fixed,
    pub coercive_capacity: Fixed,
    pub moral_obligation: Fixed,
    pub alternative_options: Fixed,
}
```

Power balance:

```text
power_balance =
    (resource_control + social_leverage + coercive_capacity + emotional_leverage)
  - alternatives
```

---

## 11.3 Hierarchy Formation

Hierarchies emerge from:

- competence,
- age,
- wealth,
- violence,
- religious sanction,
- inheritance,
- charisma,
- network centrality,
- crisis response.

Hierarchy stabilization requires:

- legitimacy,
- ritual,
- reciprocity,
- punishment,
- narrative,
- institutional memory.

Hierarchy destabilization emerges from:

- humiliation,
- corruption,
- scarcity,
- betrayal,
- rival prestige,
- moral panic,
- external shock.

---

# 12. Group Coherence, Communion, and Dissociation

Groups should form and dissolve through psychological and social mechanisms.

---

## 12.1 Group Types

| Group | Basis |
|---|---|
| Household | residence, kinship, economy |
| Family | kinship, attachment |
| Clan | kinship, myth, honor |
| Faction | grievance, interest |
| Cult | charisma, sacred narrative, identity fusion |
| Congregation | ritual, belief |
| Guild | skill, economic interest |
| Patronage network | obligation, protection |
| Peer group | age, proximity, identity |
| Warband | violence, loyalty, survival |

---

## 12.2 Group Formation Mechanics

Groups form when agents share:

- proximity,
- repeated interaction,
- emotional intensity,
- common threat,
- common need,
- shared identity,
- shared grievance,
- charismatic focal agent,
- ritual participation,
- kinship,
- economic interdependence.

```text
group_formation_pressure =
    shared_grievance
  + shared_identity
  + emotional_synchrony
  + repeated_interaction
  + leadership_gravity
  + external_threat
  - social_cost
  - institutional_suppression
```

---

## 12.3 Attachment Theory and Group Dynamics

Attachment styles scale upward.

### Secure groups

- trust leadership,
- repair conflict,
- provide support,
- tolerate dissent.

### Anxious groups

- demand reassurance,
- fear abandonment,
- escalate rumors,
- cling to leaders,
- panic under uncertainty.

### Avoidant groups

- suppress emotion,
- fragment under stress,
- distrust dependence,
- isolate members.

### Disorganized groups

- oscillate between devotion and betrayal,
- prone to purges,
- volatile leadership,
- trauma bonding.

---

## 12.4 Cult Formation Model

Cults can emerge as a special high-intensity group type.

```rust
pub struct CultDynamics {
    pub charismatic_leader: AgentId,
    pub sacred_narrative: MemeId,
    pub identity_fusion: Fixed,
    pub isolation: Fixed,
    pub dependence: Fixed,
    pub fear_load: Fixed,
    pub love_bombing: Fixed,
    pub boundary_strictness: Fixed,
    pub thought_terminating_cliches: Fixed,
    pub exit_cost: Fixed,
}
```

Cult formation conditions:

- high meaning need,
- high fear,
- low institutional legitimacy,
- charismatic leader,
- strong boundary markers,
- repeated ritual,
- social isolation,
- identity fusion.

Cult dissolution conditions:

- leader failure,
- prophecy disconfirmation,
- internal betrayal,
- external repression,
- member exhaustion,
- rival narrative,
- economic collapse.

---

## 12.5 Communion and Ritual

Ritual is a core technology of social cohesion.

```rust
pub struct Ritual {
    pub id: RitualId,
    pub name: String,
    pub participants: Vec<AgentId>,
    pub frequency: RitualFrequency,
    pub emotional_intensity: Fixed,
    pub synchrony: Fixed,
    pub sacredness: Fixed,
    pub identity_relevance: Fixed,
    pub cost: Fixed,
}
```

Ritual effects:

- increases bonding,
- reduces anxiety,
- reinforces norms,
- creates collective effervescence,
- strengthens legitimacy,
- marks life transitions,
- encodes cultural memory.

Examples:

- harvest festival,
- funeral,
- naming ceremony,
- marriage rite,
- oath-swearing,
- public confession,
- excommunication,
- seasonal prayer,
- sacrifice abstractly,
- communal meal.

---

# 13. Noospheric and Cultural Systems Upgrade

The noosphere is the symbolic field in which minds interact.

It includes:

- rumors,
- memes,
- ideologies,
- sacred symbols,
- collective memories,
- propaganda,
- moral panics,
- legitimacy narratives.

---

## 13.1 Meme System

```rust
pub struct Meme {
    pub id: MemeId,
    pub content: MemeContent,
    pub lineage: MemeLineage,
    pub emotional_charge: Fixed,
    pub identity_relevance: Fixed,
    pub moral_charge: Fixed,
    pub credibility: Fixed,
    pub novelty: Fixed,
    pub complexity: Fixed,
    pub virality: Fixed,
    pub mutation_rate: Fixed,
    pub host_count: u32,
    pub institutional_backing: Option<InstitutionId>,
    pub suppression_level: Fixed,
    pub sacredness: Fixed,
}
```

### Meme Content Types

- rumor,
- insult,
- praise,
- theological claim,
- political accusation,
- moral norm,
- conspiracy,
- prophecy,
- historical myth,
- practical knowledge,
- taboo,
- joke,
- song,
- slogan.

---

## 13.2 Meme Propagation

Transmission probability:

```text
transmission_chance =
    source_credibility
  × emotional_arousal
  × identity_relevance
  × novelty
  × listener_susceptibility
  × repetition
  × social_conformity_pressure
  × institutional_amplification
  - skepticism
  - belief_contradiction
  - suppression
```

Mutation:

```text
mutation =
    memory_error
  + emotional_exaggeration
  + identity_bias
  + narrative_simplification
  + audience_tailoring
```

---

## 13.3 Rumor System v2

Rumors are memes with uncertainty and social stakes.

```rust
pub struct Rumor {
    pub meme_id: MemeId,
    pub target: Option<EntityId>,
    pub accusation_severity: Fixed,
    pub evidence_quality: Fixed,
    pub source_chain: Vec<AgentId>,
    pub public_belief_prevalence: Fixed,
    pub emotional_contagion: Fixed,
    pub moral_panic_potential: Fixed,
}
```

Rumor effects:

- reputation damage,
- panic,
- scapegoating,
- market distortion,
- faction formation,
- institutional crisis.

---

## 13.4 Propaganda System

Institutions should be able to intentionally shape narratives.

```rust
pub struct PropagandaCampaign {
    pub id: CampaignId,
    pub sponsor: InstitutionId,
    pub target_audience: Audience,
    pub narrative: MemeId,
    pub intensity: Fixed,
    pub channels: Vec<PropagandaChannel>,
    pub credibility: Fixed,
    pub coercion: Fixed,
    pub duration: u64,
    pub resistance: Fixed,
}
```

Channels:

- sermons,
- edicts,
- public rituals,
- market announcements,
- trusted messengers,
- punishment examples,
- songs,
- monuments,
- education,
- gossip networks.

Propaganda effectiveness:

```text
effectiveness =
    institutional_legitimacy
  × messenger_trust
  × repetition
  × emotional_fit
  × identity_congruence
  × audience_fear
  × network_centrality
  - counter_narrative_strength
  - hypocrisy_evidence
```

---

## 13.5 Collective Memory

```rust
pub struct CollectiveMemory {
    pub group_id: GroupId,
    pub events: Vec<SharedMemory>,
    pub founding_myths: Vec<MemeId>,
    pub traumas: Vec<SharedTrauma>,
    pub heroes: Vec<AgentId>,
    pub villains: Vec<AgentId>,
    pub sacred_events: Vec<EventId>,
}
```

Collective memory is maintained by:

- ritual,
- storytelling,
- monuments,
- education,
- anniversary events,
- institutional repetition.

---

## 13.6 Echo Chambers and Polarization

Track network-level belief structure.

```rust
pub struct BeliefEcology {
    pub proposition_clusters: Vec<BeliefCluster>,
    pub polarization_index: Fixed,
    pub echo_chamber_strength: Fixed,
    pub cross_cutting_ties: Fixed,
    pub narrative_dominance: HashMap<MemeId, Fixed>,
}
```

Polarization emerges when:

- agents sort into homophilous networks,
- institutions compete,
- fear rises,
- gossip punishes dissent,
- identity fusion increases,
- trusted bridges collapse.

---

# 14. Integration With Existing Mindstrata Systems

The upgrade must not discard the current architecture. It should extend it.

---

## 14.1 Compatibility Strategy

### Keep

- `AgentId`,
- deterministic clock,
- fixed-point math,
- event journal,
- provenance,
- RON specs,
- test harness,
- utility AI,
- appraisal,
- gossip,
- norms,
- institutions.

### Replace via facade

- `BodyState` becomes derived from `EmbodiedState`.
- `NeedState` becomes derived from biological + psychological needs.
- `Personality` remains but is influenced by temperament and development.
- `Affect` remains but is enriched by interoception and hormones.
- `Relationship` becomes `RelationshipV2` with migration adapter.

---

## 14.2 New Tick Loop

Proposed deterministic order:

```text
1. Scenario shocks
2. Time advance / circadian / season
3. Ecology fast update
4. Sensory perception
5. Autonomic nervous system
6. Respiratory / cardiovascular fast update
7. Digestion / metabolism
8. Endocrine modulation
9. Immune / inflammation
10. Pain update
11. Body-derived needs
12. Interoception
13. Cognitive state update
14. Attention / salience
15. Memory retrieval
16. Appraisal
17. Emotion generation
18. Emotion regulation
19. Belief update
20. Theory-of-mind update
21. Identity / self-model update
22. Motivation / goal generation
23. Prospection / planning
24. Intention formation
25. Action selection
26. Action execution
27. Social interaction
28. Speech / language acts
29. Relationship update
30. Attachment update
31. Status / hierarchy update
32. Household / kinship update
33. Group / faction update
34. Institutional update
35. Meme / rumor propagation
36. Propaganda / ritual update
37. Cultural practice update
38. Memory encoding
39. Memory consolidation if sleep/daily phase
40. Skill / habit learning
41. Health / disease update
42. Reproduction / pregnancy / birth
43. Demography / aging / death
44. Market / logistics
45. Ecology slow update
46. Provenance recording
47. Metrics snapshot
```

Slow-phase systems run conditionally:

```text
if tick % DAILY == 0 { ... }
if tick % WEEKLY == 0 { ... }
if tick % SEASONAL == 0 { ... }
```

---

# 15. Data-Driven Specification Expansion

Add new RON files.

## 15.1 Proposed Spec Files

```text
specs/
  biology/
    genome.ron
    hormones.ron
    organs.ron
    diseases_v2.ron
    life_stages.ron
    reproduction.ron
  psychology/
    cognitive_systems.ron
    emotions_v2.ron
    regulation_strategies.ron
    identity_frames.ron
    moral_foundations.ron
    attachment_styles.ron
  social/
    relationship_stages.ron
    courtship.ron
    marriage.ron
    kinship.ron
    status_roles.ron
    groups.ron
  culture/
    memes.ron
    rituals.ron
    propaganda.ron
    taboos_v2.ron
    sacred_symbols.ron
    education.ron
```

---

## 15.2 Example: Relationship Stage Spec

```ron
(
    stages: [
        (
            id: "acquaintance",
            min_interactions: 1,
            trust_threshold: 0.05,
            allowed_transitions: ["familiar", "disliked"],
        ),
        (
            id: "friend",
            min_interactions: 10,
            trust_threshold: 0.45,
            affection_threshold: 0.4,
            allowed_transitions: ["close_friend", "rival", "estranged"],
        ),
        (
            id: "courtship",
            requires_adult: true,
            requires_mutual_attraction: true,
            social_visibility: 0.6,
            allowed_transitions: ["betrothal", "breakup", "scandal"],
        ),
    ],
)
```

---

## 15.3 Example: Meme Spec

```ron
(
    memes: [
        (
            id: "temple_corrupt",
            kind: PoliticalAccusation,
            emotional_charge: 0.8,
            identity_relevance: 0.7,
            base_credibility: 0.3,
            mutation_rate: 0.2,
            target_institution: "temple",
            moral_foundations: ["fairness", "purity"],
        ),
        (
            id: "elder_is_protector",
            kind: LegitimacyNarrative,
            emotional_charge: 0.4,
            identity_relevance: 0.6,
            base_credibility: 0.6,
            mutation_rate: 0.05,
            target_institution: "council",
            moral_foundations: ["authority", "care"],
        ),
    ],
)
```

---

# 16. Provenance and Explainability Upgrade

Every new system must produce debug traces.

## 16.1 New Provenance Categories

- biological cause,
- hormonal modulation,
- attachment trigger,
- identity threat,
- meme exposure,
- propaganda exposure,
- relationship transition,
- status change,
- group formation,
- ritual effect,
- belief mutation,
- trauma trigger,
- reproductive event.

## 16.2 Example Decision Trace

```text
Agent: Marta
Tick: 8421
Action: Accuse neighbor of theft

Causal chain:
  - Sleep debt: 0.72
  - Cortisol axis: 0.81
  - Hunger: 0.64
  - Recent memory: grain shortage
  - Rumor heard: "neighbor hoards grain"
  - Rumor source trust: 0.68
  - Emotional charge: 0.79
  - Identity threat: "my children may starve"
  - Moral foundation: fairness violation
  - Theory of mind: neighbor perceived as deceptive
  - Status motive: gain moral reputation
  - Institutional trust: low
  - Action selected: public accusation
```

This is how emergence becomes debuggable.

---

# 17. Performance and Scalability Strategy

This upgrade is heavy. It must be managed with level-of-detail simulation.

---

## 17.1 Agent Tiers

### Focal Agents

Full simulation:

- biology,
- psychology,
- relationships,
- memory,
- prospection,
- narrative.

### Secondary Agents

Reduced simulation:

- simplified biology,
- heuristic cognition,
- relationship updates only when relevant.

### Background Agents

Aggregate simulation:

- household-level behavior,
- statistical belief updates,
- meme exposure sampled,
- minimal memory.

Agents can be promoted/demoted dynamically based on:

- player focus,
- narrative importance,
- institutional role,
- faction leadership,
- crisis involvement.

---

## 17.2 Cognitive Budget

Each tick, each agent has limited processing capacity.

```rust
pub struct CognitiveBudget {
    pub max_appraisals: u32,
    pub max_memory_retrievals: u32,
    pub max_prospections: u32,
    pub max_social_inferences: u32,
}
```

Salient events get priority.

---

## 17.3 Relationship Caching

Do not update every relationship every tick.

Update when:

- interaction occurs,
- rumor mentions target,
- emotional event involves target,
- stage transition near threshold,
- daily decay pass.

Dormant relationships decay slowly.

---

## 17.4 Meme Aggregation

For large populations:

- track meme prevalence per group,
- sample individual exposure,
- use network centrality for spreaders,
- aggregate echo chamber metrics.

---

# 18. Testing and Validation Strategy

The upgrade must remain testable.

---

## 18.1 Unit Tests

Examples:

- hormone levels remain bounded,
- genome inheritance produces plausible ranges,
- pregnancy progresses only for adults,
- attachment style affects distress response,
- relationship stage transitions require thresholds,
- meme mutation preserves lineage,
- propaganda increases belief under trusted source,
- sleep debt increases heuristic bias,
- pain narrows attention,
- kinship coefficient prevents incestuous marriage.

---

## 18.2 Property Tests

- determinism across seeds,
- no negative age,
- no impossible fertility,
- no relationship stage regression without event,
- no belief confidence outside 0..1,
- no resource duplication,
- no agent acts without local knowledge.

---

## 18.3 Integration Tests

- two compatible adults can form courtship,
- courtship can become marriage,
- marriage can produce household,
- children inherit genetic predispositions,
- childhood trauma increases attachment insecurity,
- chronic stress increases aggression or depression risk,
- rumor degrades over transmission hops,
- trusted institution propaganda shifts belief,
- ritual increases group cohesion,
- faction forms under shared grievance,
- cult can form around charismatic leader under crisis.

---

## 18.4 Statistical Emergence Tests

Over many seeds:

- friendships correlate with proximity,
- marriages correlate with compatibility and status,
- children resemble parents statistically,
- gossip accuracy declines with hops,
- propaganda effectiveness correlates with legitimacy,
- stress correlates with conflict,
- rituals correlate with group stability,
- inequality correlates with faction formation,
- attachment insecurity correlates with relationship volatility.

---

# 19. Phased Implementation Roadmap

## Phase 0: Architectural Refactor

### Goal

Prepare the codebase without breaking existing tests.

### Tasks

- introduce `EmbodiedState` facade,
- introduce `PsychologyRuntime` trait,
- introduce `RelationshipV2` adapter,
- introduce `Meme` and `NoosphereField` structures,
- add scheduler phases,
- add new RNG streams,
- add provenance categories,
- add spec folders.

### Acceptance

- all 266 tests still pass,
- old `BodyState` API still works,
- deterministic replay preserved.

---

## Phase 1: Biological Substrate v1

### Goal

Make agents embodied.

### Systems

- genome,
- endocrine axes,
- metabolism,
- digestion,
- circadian,
- nervous arousal,
- pain,
- reproductive basics,
- aging/development.

### Acceptance

- hunger affects mood,
- stress hormones reduce planning,
- sleep debt impairs cognition,
- adults can court,
- pregnancy possible,
- children inherit traits,
- elders age and decline.

---

## Phase 2: Psychological Runtime v1

### Goal

Make agents mind-like.

### Systems

- interoception,
- attention v2,
- memory v2,
- appraisal v2,
- emotion regulation,
- identity/self-model,
- theory of mind,
- prospection,
- narrative identity.

### Acceptance

- agents ruminate,
- agents misremember under emotion,
- agents defend identity-linked beliefs,
- agents infer intentions,
- agents imagine future outcomes,
- agents produce explainable decisions.

---

## Phase 3: Relational Depth v1

### Goal

Make relationships developmental.

### Systems

- relationship stages,
- attachment,
- attraction,
- courtship,
- marriage,
- kinship,
- household,
- status/power,
- jealousy,
- betrayal/reconciliation.

### Acceptance

- friendships form gradually,
- courtship emerges,
- marriage creates households,
- kinship networks matter,
- attachment style affects conflict,
- status struggles emerge.

---

## Phase 4: Cultural / Noospheric Depth v1

### Goal

Make culture an emergent field.

### Systems

- memes,
- rumor v2,
- propaganda,
- ritual,
- collective memory,
- echo chambers,
- moral panic v2,
- sacred values.

### Acceptance

- rumors mutate,
- institutions can run campaigns,
- rituals increase cohesion,
- cults/factions can form,
- legitimacy rises/falls narratively,
- polarization measurable.

---

## Phase 5: Integration and Tuning

### Goal

Make the systems cohere.

### Tasks

- balance hormonal effects,
- tune attachment dynamics,
- tune meme virality,
- tune marriage/fertility,
- tune trauma/recovery,
- tune institutional propaganda,
- add inspector views,
- add replay visualizations.

### Acceptance

- 10,000-tick simulations remain stable,
- emergent stories are legible,
- no system dominates unnaturally,
- debug traces explain major events.

---

# 20. Example Emergent Scenario After Upgrade

A drought scenario should now unfold with much richer causality.

```text
Tick 0:
  Scenario shock: rainfall reduced.

Tick 100:
  Ecology: soil moisture declines.
  Market: grain price rises.

Tick 300:
  Agents experience hunger and thirst.
  Digestive state: low nutrients.
  Endocrine: stress axis rises.
  Nervous system: sympathetic arousal increases.
  Psychology: planning horizon shortens.

Tick 500:
  Sleep quality declines.
  Irritability rises.
  Attachment anxiety increases in insecure agents.
  Parents become more protective.
  Status competition intensifies around food access.

Tick 700:
  Rumor emerges: "The temple hoards grain."
  Meme emotional charge high.
  Gossip mutates: hoarding becomes corruption.
  Moral foundation violation: fairness + purity.

Tick 900:
  Temple legitimacy declines.
  Priest attempts ritual reassurance.
  Ritual temporarily increases bonding.
  But hypocrisy evidence accumulates.

Tick 1200:
  Faction forms among hungry, resentful agents.
  Charismatic agent with high dominance and prestige becomes leader.
  Attachment-anxious agents fuse strongly with group.
  Avoidant agents withdraw.

Tick 1500:
  Propaganda campaign by faction: "Council serves temple."
  Counter-propaganda by council: "Order preserves survival."
  Echo chambers form.
  Cross-cutting friendships strain.

Tick 1800:
  Public accusation triggers feud.
  Status challenge occurs.
  Violence erupts.
  Injury activates pain/trauma systems.
  Moral panic spreads.

Tick 2200:
  Either:
    - institution reforms and legitimacy recovers,
    - faction becomes cult,
    - clan feud stabilizes,
    - households migrate,
    - settlement collapses.
```

This is no longer a scripted revolt.

It is an embodied, psychological, relational, cultural cascade.

---

# 21. Architectural Decision Records

## ADR-001: Keep Deterministic Core

**Decision:** All authoritative simulation remains deterministic.

**Reason:** Emergent debugging requires replayability.

---

## ADR-002: Biology Is Abstract but Causal

**Decision:** Simulate organ systems as regulatory axes, not medical detail.

**Reason:** Playability and performance.

---

## ADR-003: Psychology Is Neurosymbolic

**Decision:** Use symbolic beliefs plus associative activation plus utility learning.

**Reason:** LLM-like intelligence without nondeterminism.

---

## ADR-004: Relationships Are Developmental Systems

**Decision:** Relationships have stages, history, power, attachment, and public labels.

**Reason:** Social emergence requires relational depth.

---

## ADR-005: Culture Is a Propagating Field

**Decision:** Memes, rumors, rituals, and propaganda are first-class entities.

**Reason:** Large-scale meaning dynamics require explicit modeling.

---

## ADR-006: Use Level-of-Detail Agents

**Decision:** Not every agent receives full cognitive processing every tick.

**Reason:** Scalability.

---

# 22. Risks and Mitigations

## Risk 1: Scope Explosion

**Mitigation:** Phase delivery, facades, strict MVP per system.

## Risk 2: Performance Collapse

**Mitigation:** cognitive budgets, agent tiers, slow phases, aggregation.

## Risk 3: Biological Reductionism

**Mitigation:** biology biases psychology; appraisal and identity remain central.

## Risk 4: Cultural Determinism

**Mitigation:** agents can resist, reinterpret, innovate, and convert.

## Risk 5: Ethical Sensitivity

**Mitigation:** age-gating, abstract sexuality, configurable norms, avoid essentialist genetics.

## Risk 6: Emergent Incoherence

**Mitigation:** provenance traces, statistical tests, golden replays, tuning passes.

---

# 23. Immediate Next Steps

The best immediate actions are:

1. **Create `EmbodiedState` facade**
   - preserve old `BodyState` getters,
   - add genome/endocrine/nervous stubs.

2. **Create `RelationshipV2`**
   - migrate old fields,
   - add intimacy/commitment/attachment/power.

3. **Create `Meme` and `NoosphereField`**
   - replace ad-hoc rumor logic gradually.

4. **Add scheduler phases**
   - fast/hourly/daily/weekly/seasonal.

5. **Add provenance categories**
   - biological, relational, cultural.

6. **Add first integration test**
   - stress hormones reduce planning horizon,
   - rumor mutates over three hops,
   - courtship can emerge from repeated positive interaction.

---

# 24. Final Architectural Summary

The upgraded Mindstrata agent should be:

```text
A genetically predisposed,
hormonally modulated,
embodied organism,
with a nervous system,
affective mind,
predictive cognition,
identity-protective beliefs,
attachment-shaped relationships,
status-sensitive social self,
culturally imprinted meaning system,
and locally bounded knowledge,
acting inside a deterministic world.
```

This transforms Mindstrata from:

> a strong emergent village simulation

into:

> a deep human-society simulation where history emerges from embodied minds, relational fields, and cultural ecosystems.

The foundation is already excellent. The upgrade should not replace it. It should deepen every layer beneath the agent and every layer between agents.
