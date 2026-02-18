# Bevy 0.18 — ECS Core

Covers the Entity-Component-System architecture and the core Bevy APIs that every system and feature builds on. Read this first if you are new to Bevy or need a refresher on how the engine is structured.

---

## ECS Architecture

ECS (Entity-Component-System) is the foundational design pattern in Bevy. Understanding it is essential before working with any Bevy code.

### The Core Idea

Traditional OOP games use class hierarchies: a `Player` class inherits from `Character`, which inherits from `GameObject`. ECS inverts this. Instead of objects that contain behaviour, you have:

- **Data** stored in components attached to entities
- **Logic** living in systems that operate on that data
- No inheritance; composition only

This makes game objects flexible (mix-and-match components), performant (data laid out contiguously in memory for cache efficiency), and highly parallelisable (systems can run in parallel when they don't access the same components).

### Entities

An entity is just a unique identifier — a thin handle like an integer. It has no data or behaviour on its own. Entities exist only as a way to group components together.

```rust
// Commands gives you back the entity ID when spawning:
let entity: Entity = commands.spawn(Player).id();
```

### Components

Components are plain data structs marked with `#[derive(Component)]`. They hold state but contain no logic.

```rust
#[derive(Component)]
pub struct Player;                     // Marker component (zero-size, for tagging)

#[derive(Component)]
pub struct Health(pub f32);            // Data component

#[derive(Component)]
pub struct JumpState {
    pub is_grounded: bool,
    pub jumps_remaining: u8,
}
```

**Design rules:**
- Components are data only — no methods with game logic
- Keep components small and focused; one concern per component
- Prefer many small components over one large struct
- Use marker components (zero-size) to tag entities for query filtering

### Systems

Systems are plain Rust functions. Bevy calls them every frame (or on a schedule). They declare what data they need via their parameters, and Bevy injects that data automatically.

```rust
fn apply_gravity(
    time: Res<Time>,
    mut query: Query<&mut LinearVelocity, With<Player>>,
) {
    for mut velocity in &mut query {
        velocity.y -= 980.0 * time.delta_secs();
    }
}
```

Bevy can run systems in parallel when they don't conflict (i.e., they don't both mutably access the same component type). This happens automatically — you don't write threading code.

### Resources

Resources are global singleton data — not attached to any entity. Use them for things shared across the whole game: settings, game state, asset handles, scores.

**Resource vs Component:** If the data belongs to one entity, it's a Component. If it's global and shared, it's a Resource.

### Events

Events are one-shot messages. A system writes an event; another reads it in the same or next frame. Use events for things that happen occasionally (collision detected, enemy died, level loaded) rather than polling state every frame.

### Plugins

Plugins are the unit of modularity. A plugin groups related systems, resources, and components and registers them with the app in one place.

### How It All Connects

```
┌─────────────────────────────────────────────────────────┐
│                        App                              │
│                                                         │
│  Plugins ──► register Systems, Resources, Events        │
│                                                         │
│  World ─────┬─► Entities (IDs)                         │
│             ├─► Components (data on entities)           │
│             └─► Resources (global singletons)           │
│                                                         │
│  Scheduler ─► calls Systems each frame                  │
│               Systems read/write World via:             │
│               - Query (component access)                │
│               - Res / ResMut (resource access)          │
│               - Commands (deferred entity mutations)    │
│               - EventReader / EventWriter               │
└─────────────────────────────────────────────────────────┘
```

---

## App Structure

The `App` is the entry point. It owns the ECS world and the scheduler.

```rust
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "WoK".into(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(TiledPlugin::default())
        .add_plugins(TiledPhysicsPlugin::<TiledPhysicsAvianBackend>::default())
        .add_systems(Startup, (setup_level, spawn_player))
        .add_systems(Update, (player_movement, camera_follow, exit_on_esc))
        .run();
}
```

---

## Systems and Scheduling

### Schedules

| Schedule | When it runs | Typical use |
|---|---|---|
| `Startup` | Once, before the first frame | Spawn initial entities, load assets |
| `Update` | Every frame | Game logic, input, animation |
| `FixedUpdate` | At a fixed timestep (default 64Hz) | Physics, deterministic simulation |
| `PostUpdate` | After `Update`, before rendering | Transform propagation, camera |

Physics systems from Avian2D run in `FixedUpdate`. Game logic that reads physics state should also run in `FixedUpdate` (or `PostUpdate`) to avoid one-frame lag.

### System Parameters

Systems declare dependencies through their parameters. Bevy resolves these automatically.

```rust
fn my_system(
    time: Res<Time>,                           // Read a resource
    mut gravity: ResMut<Gravity>,              // Write a resource
    mut commands: Commands,                     // Deferred entity operations
    query: Query<(&Transform, &mut Velocity)>,  // Access components
    mut events: EventWriter<PlayerDied>,        // Send events
    asset_server: Res<AssetServer>,             // Load assets
) { }
```

### System Ordering

By default, systems in the same schedule run in parallel when Bevy determines they don't conflict. When order matters:

```rust
app.add_systems(Update, (
    apply_input.before(apply_physics),
    apply_physics.before(sync_camera),
    sync_camera,
));

// Or with system sets:
app.configure_sets(Update, PhysicsSet::Movement.before(PhysicsSet::Collision));
```

---

## Queries

Queries are the primary way systems read and write component data.

### Basic Query

```rust
// Read-only
fn system(query: Query<&Transform>) {
    for transform in &query { }
}

// Mutable
fn system(mut query: Query<&mut Transform>) {
    for mut transform in &mut query {
        transform.translation.y += 1.0;
    }
}

// Multiple components
fn system(query: Query<(&Transform, &mut Velocity)>) {
    for (transform, mut velocity) in &query { }
}
```

### Filters

Filters narrow results without giving access to the filtered component's data.

```rust
Query<&Transform, With<Player>>                         // Only entities with Player
Query<&mut Health, Without<Dead>>                       // Only entities without Dead
Query<&Health, Changed<Health>>                         // Only entities where Health changed this frame
Query<&Transform, (With<Player>, Without<Frozen>)>      // Combine filters
```

### Single Entity

When a query should return exactly one result (the player, the camera):

```rust
let player_transform = player_query.single();      // panics if 0 or >1 results
let mut cam = camera_query.single_mut();
```

Use `get_single()` / `get_single_mut()` for the non-panicking versions.

---

## Commands and Spawning

`Commands` queues operations to be applied at the end of the current stage, avoiding borrow conflicts during system execution.

### Spawning

```rust
commands.spawn((
    Player,
    RigidBody::Dynamic,
    Collider::rectangle(16.0, 16.0),
    LinearVelocity::ZERO,
    Sprite::from_image(asset_server.load("player.png")),
    Transform::from_xyz(0.0, 0.0, 10.0),
));
```

### Modifying Entities

```rust
commands.entity(entity)
    .insert(Stunned { duration: 0.5 })
    .remove::<Shield>();
```

### Despawning

```rust
commands.entity(entity).despawn();             // Entity only
commands.entity(entity).despawn_recursive();   // Entity + all children
```

---

## Resources

```rust
#[derive(Resource, Default)]
pub struct Score(pub u32);

app.init_resource::<Score>();
app.insert_resource(GameSettings { gravity: 980.0 });

fn read(score: Res<Score>) { let s = score.0; }
fn write(mut score: ResMut<Score>) { score.0 += 100; }
```

---

## Events

```rust
#[derive(Event)]
pub struct Landed;

app.add_event::<Landed>();

// Send
fn check_landing(mut writer: EventWriter<Landed>) {
    writer.send(Landed);
}

// Receive — events persist for 2 frames then are cleared
fn on_landed(mut reader: EventReader<Landed>) {
    for _event in reader.read() { }
}
```

For immediate, targeted reactions, Bevy 0.18 supports **observers**:

```rust
commands.entity(player).observe(|trigger: Trigger<Landed>| {
    // Runs immediately when Landed is triggered on this entity
});
```

---

## Assets

Assets load asynchronously. `AssetServer` returns a `Handle<T>` immediately; the data arrives later.

```rust
let texture: Handle<Image> = asset_server.load("player.png");
let map: Handle<TiledMapAsset> = asset_server.load("map.tmx");

// Check if loaded before using:
if asset_server.is_loaded(&handle) { }
```

---

## Input

```rust
fn handle_input(keyboard: Res<ButtonInput<KeyCode>>) {
    if keyboard.pressed(KeyCode::ArrowRight) { }       // Held
    if keyboard.just_pressed(KeyCode::KeyZ) { }        // First frame only
    if keyboard.just_released(KeyCode::ArrowLeft) { }  // Release frame only
}
```

Key codes used in this project: `ArrowLeft`, `ArrowRight`, `ArrowUp`, `ArrowDown`, `KeyCode::KeyZ` (jump), `KeyCode::Escape`.
