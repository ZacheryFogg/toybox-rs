use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use toybox_core::graphics::Color;
use toybox_core::random;
use toybox_core::Direction;

use std::collections::{HashSet, VecDeque};

/// This struct represents the configuration of an Pacman game, and affects any new games generated from it.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]

pub struct Pacman {
     /// The random number generator that seeds new games.
     pub rand: random::Gen,
     /// A representation of the board as a list of strings.
     pub board: Vec<String>,
     /// Where does the player start on a new life?
     pub player_start: TilePoint,
     /// What is the background color?
     pub bg_color: Color,
     /// What is the backround of the game window
     pub window_color: Color,
     /// What color is the foreground: the maze walls and pellets
     pub fg_color: Color,
     /// What is the player rectangle color?
     pub player_color: Color,
     /// What is the pellet rectange color
     pub pellet_color: Color,
     /// What is the color of a vulnrable ghost
     pub vulnerable_color: Color,
     /// What is the color of a power pellet
     pub power_pellet_color: Color,
     /// What is the color of a teleport tile
     pub teleport_color: Color,
     /// What color are enemies?
     pub enemy_color: Color,
     /// How many lives do new games start with?
     pub start_lives: i32,
     /// How long does a power pellet make the ghosts vulnerable for
     pub vulnerable_time: i32,
     /// How long should a mob be immobilized when player catches it 
     pub immobilized_time: i32,
     /// Should we show images/sprites (true) or just colored rectangles (false).
     pub render_images: bool,
     /// What AIs should we use to spawn enemies on a new game?
     pub enemies: Vec<MovementAI>,
     /// How many previous junctions should the player and enemies remember?
     pub history_limit: u32,
     /// How fast do enemies move?
     pub enemy_starting_speed: i32,
     /// How fast does the player move?
     pub player_speed: i32,
     /// How much score is required to gain an additional life?
     pub life_gain_threshold: i32,
     /// How much score is gained per pellet collection?
     pub score_increase_per_pellet: i32,
     /// How much score is gained per power pellet collection?
     pub score_increase_per_power_pellet: i32,
     /// What is the base score for catching a ghost?
     pub score_increase_base_per_ghost_catch: i32, 
     /// Multiplier for how long an enemy is immobilized at the start of a level 
     pub start_immobilized_base: i32,
}

/// When things are drawn, they are drawn in screen coordinates, i.e., pixels.
#[derive(Debug, Clone)]
pub struct ScreenPoint {
    pub sx: i32,
    pub sy: i32,
}

/// Strongly-typed vector for "world" positioning in Pacman. World points are larger than screen points because players/enemies often move fractions of a pixel per frame.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorldPoint {
    pub x: i32,
    pub y: i32,
}

/// Strongly-typed vector for "tile" positioning in Amidar. These coordinates are related to world and screen points, but are more useful for addressing specific painted/unpainted tiles.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TilePoint {
    pub tx: i32,
    pub ty: i32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
pub enum Tile {
    /// Tiles that are maze walls are treated like walls for collision.
    Wall,
    /// Walkable tiles that contain a pellet
    Pellet,
    /// Walkable tiles that contain a power pellet.
    PowerPellet,
    /// Tile that is devoid of any pellet; it may have always been empty or its pellet may have been collected
    Empty,
    /// Tile will teleport Mob to a corresponding postion on the opposite side of the board
    Teleport,
    /// House blocks look empty, but are not walkable
    House
}

/// MovementAI represents Mob (enemy/player) logic for movement.
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, JsonSchema)]
pub enum MovementAI {
    /// Movement is based upon input commands.
    Player,  
    /// At every junction, an enemy chooses a random legal direction and proceeds in that direction until hitting the next junction.
    EnemyRandomMvmt {
        /// Where do I start?
        start: TilePoint,
        /// Which direction to move first?
        start_dir: Direction,
        /// Which direction am I currently moving?
        dir: Direction,
    },
}

/// Mob is a videogame slang for "mobile" unit. Players and Enemies are the same struct.
#[derive(Clone, Serialize, Deserialize, JsonSchema)]
pub struct Mob {
    /// How is this unit controlled?
    pub ai: MovementAI,
    /// Where is this unit placed (WorldPoint represents sub-pixels!)
    pub position: WorldPoint,
    /// Am I vulnerable: ghosts are blue / flash if vulnerbale, players vulnerability is irrelevant
    pub vulnerable: bool,
    /// How fast do I get to move?
    pub speed: i32,
    /// Am I currently moving toward a point?
    pub step: Option<TilePoint>,
    /// Which junctions have I visited most recently?
    pub history: VecDeque<u32>,
    /// How much longer is the Mob immobilized: Used when player catches a ghost, it should spawn and not move for a period of time
    pub immobilized_timer: i32, // not relevant for player
}

/// Board represents the Pacman level/board and all associated information.
#[derive(Clone, Serialize, Deserialize, JsonSchema)]
pub struct Board {
    /// What are the state of the tiles on the board: rows first, then columns.
    pub tiles: Vec<Vec<Tile>>,
    /// How wide is the board?
    pub width: u32,
    /// How tall is the board?
    pub height: u32,
    /// Which positions (y*width + x) are junctions? Helps MovementAI and painting game logic!
    pub junctions: HashSet<u32>,
}

// This struct is temporarily used inside of the game logic, to ensure purely-functional behavior in certain points. Encodes any changes to the board that could happen in a single update.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug, JsonSchema)]
pub struct BoardUpdate {
    /// Number of pellets collected
    pub pellets_collected: i32,
    /// If a power pellet was collected, meaning that ghosts on the board will become vulnerable
    pub power_pellets_collected: i32,
    /// If we just collected something, the start junction and the end junction as a tuple!
    pub junctions: Option<(u32, u32)>,

}

#[derive(Clone, Serialize, Deserialize, JsonSchema)]
// Per frame state, duplicate fields to Pacman 
pub struct StateCore {
    /// Where are random numbers drawn from?
    pub rand: random::Gen,
    /// How many points have the player earned?
    pub score: i32,
    /// How many lives does the player posess?
    pub lives: i32,
    /// What is the current level? 1-based. The levels will be no different
    pub level: i32,
    /// The position and state of the player.
    pub player: Mob,
    /// If player catches more than one enemy during a powerup(collecting one power pellet), they will earn more score. 
    pub enemies_caught_multiplier: i32,
    /// The position and other state for the enemies.
    pub enemies: Vec<Mob>,
    /// A representation of the current game board.
    pub board: Board,
    /// Timer representing how much longer the ghosts are vulnerable for 
    pub vulnerability_timer: i32,
    /// How many lives Pacman has gained through score increased: relevant for increasing lives every x score
    pub lives_gained: i32,
}

/// Wrapping the current game config into one struct with the current frame state.
#[derive(Clone, Serialize, Deserialize, JsonSchema)]
pub struct State {
    /// The config that generated the original state for this game.
    pub config: Pacman,
    /// The state that represents the immediately current frame.
    pub state: StateCore,
}

/// When we compared the player position to all the enemies, what happened?
#[derive(PartialEq, Eq, Clone, Copy, JsonSchema)]
pub enum EnemyPlayerState {
    /// Most of the time: nobody's colliding.
    Miss,
    /// The player just died.
    PlayerDeath,
    /// In chase mode, the player just caught the given enemy (id by index in state.enemies list!)
    EnemyCatch(usize),
}

