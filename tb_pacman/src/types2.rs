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
    /// What color is the foreground(The maze, and pellets)
    pub fg_color: Color,
    /// What is the player rectangle color?
    pub player_color: Color,
    /// What color are enemies?
    pub enemy_color: Color,
    /// How many lives do new games start with?
    pub start_lives: i32,
    /// Should we show images/sprites (true) or just colored rectangles (false).
    pub render_images: bool,
    /// This should be false if you ever use a non-default board.
    //  pub default_board_bugs: bool,
    /// What AIs should we use to spawn enemies on a new game?
    pub enemies: Vec<MovementAI>,
    /// How many previous junctions should the player and enemies remember?
    // pub history_limit: u32,
    /// How fast do enemies move?
    pub enemy_starting_speed: i32,
    /// How fast does the player move?
    pub player_speed: i32,
    /// How long are the ghosts vulnerable for 
    pub vulnerable_time: i32
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

/// This represents the boxes on the board, whether or not contains pellets
// #[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
// pub struct GridBox {
//     /// Dimension of the GridBox: which tile is its top-left? Specifies location.
//     pub top_left: TilePoint,
//     /// Dimension of the GridBox: which tile is its bottom-right? Specifies size implicitly.
//     pub bottom_right: TilePoint,
//     /// Does this gridbox contain a pellet 
//     pub pellet: bool,
//     /// Does this gridbox contain a power pellet 
//     pub power_pellet: bool,
    
// }

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
    /// Tile will teleport Mob to the right corridor
    LeftTeleport,
    /// Tile will teleport Mob to the left corridor
    RightTeleport,
    /// Tile is walkable by Ghosts but not by player
    Home,
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
    /// Move randomly unless the player is within some fixed Manhattan distance of this enemy -- in that case, move toward the player.
    EnemyTargetPlayer {
        /// Where do I start?
        start: TilePoint,
        /// Which direction do I explore first?
        start_dir: Direction,
        /// How far (Manhattan distance) can I see?
        vision_distance: i32,
        /// Which direction am I currently moving?
        dir: Direction,
        /// We lock onto a player's position when we see it, so that we can actually be evaded.
        player_seen: Option<TilePoint>,
    },
}

/// Mob is a videogame slang for "mobile" unit. Players and Enemies are the same struct.
#[derive(Clone, Serialize, Deserialize, JsonSchema)]
pub struct Mob {
    /// How is this unit controlled?
    pub ai: MovementAI,
    /// Where is this unit placed (WorldPoint represents sub-pixels!)
    pub position: WorldPoint,
    /// Have I been caught/eaten?
    pub caught: bool,
    /// Am I vulnerable: Ghosts flash if vulnerbale, Pacman is always vulnerbale unless colliding with a vulnerbale enemy
    pub vulnerable: bool,
    /// How fast do I get to move?
    pub speed: i32,
    /// Am I currently moving toward a point?
    pub step: Option<TilePoint>,
    /// Which junctions have I visited most recently?
    pub history: VecDeque<u32>,
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
    /// The list of boxes (inside-portions) of the board.
    // pub boxes: Vec<GridBox>,
}

/// This struct is temporarily used inside of the game logic, to ensure purely-functional behavior in certain points. Encodes any changes to the board that could happen in a single update.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug, JsonSchema)]
pub struct BoardUpdate {
    /// Number of pellets collected
    pub pellets_collected: i32,
    /// If a power pellet was collected, meaning that ghosts on the board will become vulnerable
    pub power_pellets_collected: i32,
    /// If we just collected something, the start junction and the end junction as a tuple!
    pub junctions: Option<(u32, u32)>,
    /// If a vulnerable ghost was consumed,
    pub ghost_consumed: bool,
    /// If a telleport occured: 0 = false, 1 = left->right, 2 = right->left
    pub teleport: i32
}

#[derive(Clone, Serialize, Deserialize, JsonSchema)]
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
    /// Are the enemies vulnerable 
    pub enemies_vulenerable: bool,
    //  How much longer are the ghosts vulnerable for 
    pub vulnerability_timer: i32,
    /// The position and other state for the enemies.
    pub enemies: Vec<Mob>, // vector or Mobs
    /// A representation of the current game board.
    pub board: Board,
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
