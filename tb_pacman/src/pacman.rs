use super::digit_sprites::{draw_score, DIGIT_HEIGHT};
use crate::typespacman::*;
use access_json::JSONQuery;
use serde_json;
use std::collections::{HashSet, VecDeque};
use toybox_core;
use toybox_core::graphics::{Color, Drawable, FixedSpriteData};
use toybox_core::random;
use toybox_core::{AleAction, Direction, Input, QueryError};
use rand::seq::SliceRandom;
use std::time::{Duration, SystemTime};
// use std::thread::sleep;

// Module contains basic constants related to GUI
pub mod screen {
    pub const GAME_SIZE: (i32, i32) = (147, 160); // Size of GUI window
    pub const BOARD_OFFSET: (i32, i32) = (0,0); // Game offset from top-left corner of GUI
    pub const PLAYER_SIZE: (i32, i32) = (8, 8); // Size of non-sprite player - GUI size only - not collision boxes
    pub const ENEMY_SIZE: (i32, i32) = (8, 8);
    pub const TILE_SIZE: (i32, i32) = (7, 7); // Size of each tile - GUI and Collision 
    pub const LIVES_Y_POS: i32 = 130; // Position of lives markers
    pub const LIVES_X_POS: i32 = 100;
    pub const LIVES_X_STEP: i32 = 16; 
    pub const SCORE_Y_POS: i32 = 130;
    pub const SCORE_X_POS: i32 = LIVES_X_POS - LIVES_X_STEP * 3 - 14;
}


// Module contains images for game GUI. Relevant if render_images == True
pub mod raw_images {
    pub const PACMAN_MID: &[u8] = include_bytes!("resources/pacman/pacmanMid.png");
    pub const PACMAN_CLOSED: &[u8] = include_bytes!("resources/pacman/pacmanClosed.png");
    pub const PACMAN_OPEN: &[u8] = include_bytes!("resources/pacman/pacmanOpen.png");
    pub const PACMAN_LARGE: &[u8] = include_bytes!("resources/pacman/pacmanLarge.png");
    pub const GHOST_RED1: &[u8] = include_bytes!("resources/pacman/ghostRed1.png");
    pub const GHOST_RED2: &[u8] = include_bytes!("resources/pacman/ghostRed2.png");
    pub const GHOST_PINK1: &[u8] = include_bytes!("resources/pacman/ghostPink1.png");
    pub const GHOST_PINK2: &[u8] = include_bytes!("resources/pacman/ghostPink2.png");
    pub const GHOST_GREEN1: &[u8] = include_bytes!("resources/pacman/ghostGreen1.png");
    pub const GHOST_GREEN2: &[u8] = include_bytes!("resources/pacman/ghostGreen2.png");
    pub const GHOST_YELLOW1: &[u8] = include_bytes!("resources/pacman/ghostYellow1.png");
    pub const GHOST_YELLOW2: &[u8] = include_bytes!("resources/pacman/ghostYellow2.png");
    pub const GHOST_VULNERABLE_WHITE: &[u8] = include_bytes!("resources/pacman/ghostVulnerableWhite.png");
    pub const GHOST_VULNERABLE_BLUE: &[u8] = include_bytes!("resources/pacman/ghostVulnerableBlue.png");
    pub const TILE_WITH_PELLET: &[u8] = include_bytes!("resources/pacman/tileWithPellet.png");
    pub const TILE_WITH_POWER_PELLET: &[u8] = include_bytes!("resources/pacman/tileWithPowerPellet.png");
    pub const TILE_EMPTY: &[u8] = include_bytes!("resources/pacman/tileEmpty.png");
    pub const TILE_WALL: &[u8] = include_bytes!("resources/pacman/tileWall.png");
}


// Module creates static references to images
pub mod images {
    use super::*;
    lazy_static! { //lazy_static to allow other variables to be loaded
        pub static ref PACMAN_MID: FixedSpriteData = FixedSpriteData::load_png(raw_images::PACMAN_MID);
        pub static ref PACMAN_CLOSED: FixedSpriteData = FixedSpriteData::load_png(raw_images::PACMAN_CLOSED);
        pub static ref PACMAN_OPEN: FixedSpriteData = FixedSpriteData::load_png(raw_images::PACMAN_OPEN);
        pub static ref PACMAN_LARGE: FixedSpriteData = FixedSpriteData::load_png(raw_images::PACMAN_LARGE);
        pub static ref GHOST_RED1: FixedSpriteData = FixedSpriteData::load_png(raw_images::GHOST_RED1);
        pub static ref GHOST_RED2: FixedSpriteData = FixedSpriteData::load_png(raw_images::GHOST_RED2);
        pub static ref GHOST_PINK1: FixedSpriteData = FixedSpriteData::load_png(raw_images::GHOST_PINK1);
        pub static ref GHOST_PINK2: FixedSpriteData = FixedSpriteData::load_png(raw_images::GHOST_PINK2);
        pub static ref GHOST_GREEN1: FixedSpriteData = FixedSpriteData::load_png(raw_images::GHOST_GREEN1);
        pub static ref GHOST_GREEN2: FixedSpriteData = FixedSpriteData::load_png(raw_images::GHOST_GREEN2);
        pub static ref GHOST_YELLOW1: FixedSpriteData = FixedSpriteData::load_png(raw_images::GHOST_YELLOW1);
        pub static ref GHOST_YELLOW2: FixedSpriteData = FixedSpriteData::load_png(raw_images::GHOST_YELLOW2);
        pub static ref GHOST_VULNERABLE_WHITE: FixedSpriteData = FixedSpriteData::load_png(raw_images::GHOST_VULNERABLE_WHITE);
        pub static ref GHOST_VULNERABLE_BLUE: FixedSpriteData = FixedSpriteData::load_png(raw_images::GHOST_VULNERABLE_BLUE);
        pub static ref TILE_WITH_PELLET: FixedSpriteData = FixedSpriteData::load_png(raw_images::TILE_WITH_PELLET);
        pub static ref TILE_WITH_POWER_PELLET: FixedSpriteData = FixedSpriteData::load_png(raw_images::TILE_WITH_POWER_PELLET);
        pub static ref TILE_EMPTY: FixedSpriteData = FixedSpriteData::load_png(raw_images::TILE_EMPTY);
        pub static ref TILE_WALL: FixedSpriteData = FixedSpriteData::load_png(raw_images::TILE_WALL);
    }
}


// Module creates variables related to World (world is internal, while screen in GUI)
mod world {
    use super::screen; 
    pub const SCALE: i32 = 32;
    pub const TILE_SIZE: (i32, i32) = (screen::TILE_SIZE.0 * SCALE, screen::TILE_SIZE.1 * SCALE);
}


// Load board from file
pub const PACMAN_BOARD: &str = include_str!("resources/pacman_default_board");


// Module intiates speed of mobs 
mod inits {
    pub const ENEMY_STARTING_SPEED: i32 = 18;
    pub const PLAYER_SPEED: i32 = 30;
}


// Implement Pacman type
impl Pacman {
    pub fn colors(&self) -> Vec<&Color> {
        vec![
            &self.bg_color,
            &self.fg_color,
            &self.enemy_color, // These two are only relevant if render_images == false
            &self.player_color,
        ]
    }
}

// Implement Pacman default values 
impl Default for Pacman {
    fn default() -> Self {
        Pacman {
            rand: random::Gen::new_from_seed(13),
            board: PACMAN_BOARD.lines().map(|s| s.to_owned()).collect(),
            player_start: TilePoint::new(10, 11), 
            bg_color: Color::rgb(6,21,136),
            window_color: Color::rgb(0,0,0),
            fg_color: Color::rgb(232, 107, 115),
            pellet_color: Color::rgb(252, 186, 3),
            power_pellet_color: Color::rgb(245, 129, 66),
            teleport_color: Color::rgb(200, 66, 245),
            player_color: Color::rgb(197,187,88),
            enemy_color: Color::rgb(255,0,0),
            vulnerable_color: Color::rgb(76, 108, 206),
            start_lives: 3,
            history_limit: 12,
            render_images: true,
            enemy_starting_speed: inits::ENEMY_STARTING_SPEED,
            life_gain_threshold: 10000,
            score_increase_per_pellet: 10,
            score_increase_per_power_pellet: 50,
            score_increase_base_per_ghost_catch: 200,
            player_speed: inits::PLAYER_SPEED,
            vulnerable_time: 500, // 10 seconds
            // 4 random agents for now
            enemies: vec![MovementAI::EnemyRandomMvmt {start: TilePoint::new(10, 6), start_dir: Direction::Up, dir: Direction::Up,},
                          MovementAI::EnemyRandomMvmt {start: TilePoint::new(10, 6), start_dir: Direction::Up, dir: Direction::Up,},
                          MovementAI::EnemyRandomMvmt {start: TilePoint::new(10, 6), start_dir: Direction::Up, dir: Direction::Up,},
                          MovementAI::EnemyRandomMvmt {start: TilePoint::new(10, 6), start_dir: Direction::Up, dir: Direction::Up,}]   
        }
    }
}

// Impl ScreenPoint is used to convert a WorldPoint to the GUI position (pixel)
impl ScreenPoint {
    // Return a new ScreenPoint
    fn new(sx: i32, sy: i32) -> ScreenPoint {
        ScreenPoint { sx, sy }
    }
    // Return tuple that is screen point (pixel pos)
    pub fn pixels(&self) -> (i32, i32) {
        (self.sx, self.sy)
    }
}


// Impl WorldPoint: which is the underlying positioning system
impl WorldPoint {
    // Returns WorldPoint 
    fn new(x: i32, y: i32) -> WorldPoint {
        WorldPoint { x, y }
    }
    // Convert current WorldPoint to ScreenPoint. WorldPoints >= ScreenPoints
    pub fn to_screen(&self) -> ScreenPoint {
        ScreenPoint::new(self.x / world::SCALE, self.y / world::SCALE)
    }
    // Convert WorldPoint to TilePoint
    pub fn to_tile(&self) -> TilePoint {
        let mut tx = self.x / world::TILE_SIZE.0;
        let mut ty = self.y / world::TILE_SIZE.1;
        if self.x < 0 {
            tx -= 1;
        }
        if self.y < 0 {
            ty -= 1;
        }
        TilePoint::new(tx, ty)
    }
    // Translate WorldPoint by given deltas 
    pub fn translate(&self, dx: i32, dy: i32) -> WorldPoint {
        WorldPoint::new(self.x + dx, self.y + dy)
    }
}

// TilePoint type: used for tile positioning in Pacman. Coords relates to World and Screen Points
impl TilePoint {
    pub fn new(tx: i32, ty: i32) -> TilePoint {
        TilePoint { tx, ty }
    }
    // Return Manhattan Distance between this tile and another TilePoint
    pub fn manhattan_dist(&self, other: &TilePoint) -> i32 {
        (self.tx - other.tx).abs() + (self.ty - other.ty).abs()
    }
    // Create WorldPoint from TilePoint using TILE_SIZE as a scaling factor
    pub fn to_world(&self) -> WorldPoint {
        WorldPoint::new(self.tx * world::TILE_SIZE.0, self.ty * world::TILE_SIZE.1)
    }
    // Return a new TilePoint which is self but translated by certain delta
    pub fn translate(&self, dx: i32, dy: i32) -> TilePoint {
        TilePoint::new(self.tx + dx, self.ty + dy)
    }
    // Return new translated TilePoint in direction which is inputed 
    pub fn step(&self, dir: Direction) -> TilePoint {
        let (dx, dy) = dir.delta();
        self.translate(dx, dy)
    }
}

// Implement Tile type: 
impl Tile {
    fn new_from_char(c: char) -> Result<Tile, String> {
    // Convert character into Tile of certain type
        match c {
            'e' => Ok(Tile::Empty),
            '=' => Ok(Tile::Pellet), // Empty + pellet
            'p' => Ok(Tile::PowerPellet), // Empty + power pellet
            '#' => Ok(Tile::Wall), // Non movelable area 
            'h' => Ok(Tile::House), // Enemies home
            't' => Ok(Tile::Teleport),
            _ => Err(format!("Cannot construct AmidarTile from '{}'", c)),
        }
    }
    // Tile can be walked on
    pub fn walkable(self) -> bool {
        match self{
            Tile::House | Tile::Wall => false,
            Tile::Pellet | Tile::PowerPellet | Tile::Empty | Tile::Teleport => true,
        }
    }

    // Tile contains a pellet or power pellet 
    pub fn is_still_collectable(self) -> bool {
        match self {
            Tile::Pellet | Tile::PowerPellet => true,
            Tile::Empty | Tile::Wall | Tile::House | Tile::Teleport => false,
        }
    }
}

// Implement MovementAI types 
impl MovementAI {
    /// Resetting the mob AI state after player death.
    fn reset(&mut self) {
        // match which type of MovementAI this 
        match self {
            MovementAI::Player => {} 
            MovementAI::EnemyRandomMvmt {
                ref mut dir,
                start_dir,
                ..
            } => {
                *dir = *start_dir;
            }
        }
    }
    //depending on 
    fn choose_next_tile(
        &mut self,
        position: &TilePoint, 
        buttons: Input, // defined in toybox core 
        board: &Board, // board define below
        player: Option<Mob>, // could pass in a mob or not, define below
        rng: &mut random::Gen, //random number gen 
    ) -> Option<TilePoint> { // return an optional TilePoint
        match self {
            // match what type of player this is
            &mut MovementAI::Player => { //if player, then take their action and return it
                let mut input: Option<Direction> = None;
                if buttons.left {
                    input = Some(Direction::Left); //direction is from toybox core 
                } else if buttons.right {
                    input = Some(Direction::Right);
                } else if buttons.up {
                    input = Some(Direction::Up);
                } else if buttons.down {
                    input = Some(Direction::Down);
                }

                input.and_then(|dir| { // will return none if input is none 
                    let target_tile = position.step(dir); // get the target tile that we will end up at
                    if board.get_tile(&target_tile).walkable() { 
                        Some(target_tile)
                    } else {
                        None
                    }
                })
            }
            &mut MovementAI::EnemyRandomMvmt { ref mut dir, .. } => {
                let directions = &[
                    Direction::Up,
                    Direction::Down,
                    Direction::Left,
                    Direction::Right,
                ];
                let tp_default = board.can_move(position, *dir);
                if board.is_junction(position) || tp_default.is_none() {
                    let eligible: Vec<(&Direction, Option<TilePoint>)> = directions
                        .iter()
                        .map(|d| (d, board.can_move(position, *d)))
                        .filter(|(_, tp)| tp.is_some())
                        .collect();
                    let (d, tp) = eligible.choose(rng).cloned().unwrap();
                    // Move to the randomly selected tile point, in its dir.
                    *dir = *d;
                    return tp;
                }
                tp_default
            }
        }
    }
}

// Mob type 
impl Mob {
    // create a new mob that is comprised of MovementAI tpye, WorldPoint and speed 
    fn new(ai: MovementAI, position: WorldPoint, speed: i32) -> Mob {
        Mob {
            ai,//movementAI type 
            position,//position in world
            step: None,
            caught: false,
            vulnerable: false,
            speed,
            history: VecDeque::new(),
        }
    }
    // create newplayer which is of movement type Player 
    pub fn new_player(position: WorldPoint, speed: i32) -> Mob {
        Mob {
            ai: MovementAI::Player,
            position,
            step: None,
            caught: false,
            vulnerable: false,
            speed,
            history: VecDeque::new(),//history of agents positions 
        }
    }
    fn is_player(&self) -> bool {
        self.ai == MovementAI::Player
    }
    fn change_speed(&mut self, new_speed: i32) {
        self.speed = new_speed;
    }
    fn teleport(&mut self, board: &Board) {
        self.step = None; 
        let mut new_x = 0;
        if self.position.to_tile().tx == 0 {
            new_x = 19;
        } else {
            new_x = 1;
        }
        self.position = TilePoint::new(new_x, self.position.to_tile().ty).to_world();
        // self.position = match self.ai {
        //     MovementAI::Player => new_position,
        //     MovementAI::EnemyRandomMvmt { ref start, .. } => start.clone().to_world(),
        // }

    }
    // reset agent in world to it's start
    fn reset(&mut self, player_start: &TilePoint, board: &Board) {
        self.step = None;
        self.ai.reset();
        self.position = match self.ai {
            MovementAI::Player => player_start.to_world(),
            MovementAI::EnemyRandomMvmt { ref start, .. } => start.clone().to_world(),
        };
        self.history.clear();
        self.caught = false;
        self.vulnerable = false;
    }
    // return an optional board update 
    pub fn update(
        &mut self,
        buttons: Input,
        board: &mut Board,
        player: Option<Mob>,
        history_limit: u32,
        rng: &mut random::Gen,
    ) -> Option<BoardUpdate> {
        if self.history.is_empty() {
            if let Some(pt) = board.get_junction_id(&self.position.to_tile()) {
                self.history.push_front(pt);
            }
        }

        // Animate/step player movement.
        let next_target = if let Some(ref target) = self.step {
            // Move player 1 step toward its target:
            let world_target = target.to_world();
            let dx = world_target.x - self.position.x;
            let dy = world_target.y - self.position.y;

            if dx == 0 && dy == 0 {
                // We have reached this target tile:
                if let Some(pt) = board.get_junction_id(target) {
                    self.history.push_front(pt);
                }
                None
            } else if dx.abs() < self.speed && dy.abs() < self.speed {
                self.position.x += dx;
                self.position.y += dy;
                if let Some(pt) = board.get_junction_id(target) {
                    self.history.push_front(pt);
                }
                None
            } else {
                self.position.x += self.speed * dx.signum();
                self.position.y += self.speed * dy.signum();
                Some(target.clone())
            }
        } else {
            None
        };

        // Rust prevents us from modifying step back to None when we reach it in the above block, since we have bound a reference to the inside of the if-let-Some.
        // We therefore unconditionally return the target from that expression and write it mutably here, where it is obvious it is safe to the rust compiler.
        self.step = next_target;

        // Not an else if -- if a player or enemy reaches a tile they can immediately choose a new target.
        if self.step.is_none() {
            self.step =
                self.ai
                    .choose_next_tile(&self.position.to_tile(), buttons, board, player, rng)
        }
        if self.is_player(){

            board.check_pellets_every_tile(&mut self.position,&mut self.history).into_option()
        }
        // Manage history:
        // if self.is_player() {
        //     // Check if based on player history any pellets were collected
        //     board.check_pellets(&mut self.history).into_option()
            
        // } 
        else {
            // Each moving object in Amidar keeps track of which junctions it has visited. Here, we
            // make sure that datastructure does not grow unbounded with time; limiting it to
            // what is defined in the config.

            if self.history.len() > (history_limit as usize) {
                let _ = self.history.pop_back();
            }
            None
        }
    }
}

lazy_static! {
    //map the input board to a board object
    static ref DEFAULT_BOARD: Board = Board::try_new(
        &PACMAN_BOARD
            .lines()
            .map(|s| s.to_owned())
            .collect::<Vec<_>>()
    )
    .unwrap();
}

impl BoardUpdate {
    fn new() -> BoardUpdate {
        BoardUpdate {
            junctions: None,
            pellets_collected: 0,
            power_pellets_collected:0,
            teleport: 0,
            ghosts_consumed: 0,
        }
    }//maybe indicates if some significant event happened
    fn happened(&self) -> bool {
        self.junctions.is_some()
            || self.pellets_collected != 0
            || self.power_pellets_collected != 0
            || self.ghosts_consumed != 0
            || self.teleport != 0
    }
    // 
    fn into_option(self) -> Option<Self> {
        if self.happened() {
            Some(self)
        } else {
            None
        }
    }
}

impl Board {
    // Return a clone of the default board
    pub fn fast_new() -> Board {
        DEFAULT_BOARD.clone()
    }
    // Make a new Board object and return the board and the results
    fn try_new(lines: &[String]) -> Result<Board, String> {
        let mut tiles = Vec::new();
        for line in lines {
            // Take each row in line list and use Tile to map the character to a Tile of matching type
            let row: Result<Vec<_>, _> = line.chars().map(Tile::new_from_char).collect();
            // Exit function if row is errorful.
            tiles.push(row?);
        }
        let width = tiles[0].len() as u32;
        let height = tiles.len() as u32;
        // Create board
        let mut board = Board {
            tiles,
            width,
            height,
            junctions: HashSet::new(),
            // boxes: Vec::new(),
        };
        board.init_junctions();
        // debug_assert!(board.boxes.is_empty());
        // board.boxes = board
        //     .junctions
        //     .iter()
        //     .flat_map(|pt| board.junction_corners(*pt))
        //     .collect();
        Ok(board)
    }
    // is a corner of the board 
    fn is_corner(&self, tx: i32, ty: i32) -> bool {
        let last_y = (self.height as i32) - 1;
        let last_x = (self.width as i32) - 1;
        (tx == 0 || tx == last_x) && (ty == 0 || ty == last_y)
    }
    // can move in a given direction from a tilepoint
    fn can_move(&self, position: &TilePoint, dir: Direction) -> Option<TilePoint> {
        let tx = position.tx;
        let ty = position.ty;
        let (dx, dy) = dir.delta();
        let tp = TilePoint::new(tx + dx, ty + dy);
        let tile = self.get_tile(&tp);
        if tile.walkable() {
            Some(tp)
        } else {
            None
        }
    }

    fn is_junction(&self, tile: &TilePoint) -> bool {
        if let Some(num) = self.tile_id(tile) {
            self.junctions.contains(&num)
        } else {
            false
        }
    }

    fn init_junctions(&mut self) {
        // Only run this function once.
        debug_assert!(self.junctions.is_empty());

        for (y, row) in self.tiles.iter().enumerate() {
            let y = y as i32;
            for (x, cell) in row.iter().enumerate() {
                let x = x as i32;
                if cell.walkable() {
                    let neighbors = [(x + 1, y), (x, y + 1), (x - 1, y), (x, y - 1)];
                    let walkable_neighbors = neighbors
                        .iter()
                        .filter(|&&(nx, ny)| self.get_tile(&TilePoint::new(nx, ny)).walkable())
                        .count();
                    if walkable_neighbors > 2 || self.is_corner(x, y) {
                        let y = y as u32;
                        let x = x as u32;
                        let _ = self.junctions.insert(y * self.width + x);
                        // if cell == &Tile::ChaseMarker {
                        //     self.chase_junctions.insert(y * self.width + x);
                        // }
                    }
                }
            }
        }
    }

    // fn is_painted(&self, xy: &TilePoint) -> bool {
    //     self.get_tile(xy) == Tile::Painted
    // }
    // fn is_pellet(&self, xy: &TilePoint) -> bool {
    //     self.get_tile(xy) == Tile::Pellet
    // }
    // fn is_power_pellet(&self, xy: &TilePoint) -> bool {
    //     self.get_tile(xy) == Tile::PowerPellet
    // }
    fn is_tp(&self, xy: &TilePoint) -> bool {
        self.get_tile(xy) == Tile::Teleport
    }
    // fn is_house(&self, xy: &TilePoint) -> bool {
    //     self.get_tile(xy) == Tile::House
    // }
    // fn is_wall(&self, xy: &TilePoint) -> bool {
    //     self.get_tile(xy) == Tile::Wall
    // }

    // give the tile an id
    fn tile_id(&self, tile: &TilePoint) -> Option<u32> {
        if tile.ty < 0
            || tile.tx < 0
            || tile.ty >= self.height as i32
            || tile.tx >= self.width as i32
        {
            return None;
        }
        let y = tile.ty as u32;
        let x = tile.tx as u32;
        Some(y * self.width + x)
    }
    
    fn get_junction_id(&self, tile: &TilePoint) -> Option<u32> {
        if let Some(num) = self.tile_id(tile) {
            if self.junctions.contains(&num) {
                Some(num)
            } else {
                None
            }
        } else {
            None
        }
    }
    fn check_pellets_every_tile(&mut self, player_position: &WorldPoint, player_history: &mut VecDeque<u32>) -> BoardUpdate {
        let mut score_change = BoardUpdate::new();
        let current_tile = player_position.to_tile();
        // println!("X: {} Y: {}", current_tile.tx ,current_tile.ty);
        let mut newly_pellet_emptied = false;
        let mut newly_power_pellet_emptied = false;
        newly_pellet_emptied |= self.collect_pellet(&current_tile);
        newly_power_pellet_emptied |= self.collect_power_pellet(&current_tile);
        if newly_pellet_emptied {
            score_change.pellets_collected += 1;
        } else if newly_power_pellet_emptied {
            score_change.power_pellets_collected +=1;
        }
        if score_change.happened() {
            // Don't forget this location should still be in history:
            let current = *player_history.front().unwrap();
            player_history.clear();
            player_history.push_front(current);
        }
        score_change
    }
    // This function takes in the players history to determine a board update
    // fn check_pellets(&mut self, player_history: &mut VecDeque<u32>) -> BoardUpdate {
        // Init a BoardUpdate to represent the score_change
    //     let mut score_change = BoardUpdate::new();
    //     // Iterate through player history and collect pellets
    //     if let Some(end) = player_history.front() {
    //         // // Is still updating only every junctions
    //         // let current_tile = self.lookup_position(*end);
    //         // // println!("X: {} Y: {}",current_tile.tx, current_tile.ty);
    //         // let mut newly_pellet_emptied = false;
    //         // let mut newly_power_pellet_emptied = false;
    //         // newly_pellet_emptied |= self.collect_pellet(&current_tile);
    //         // newly_power_pellet_emptied |= self.collect_power_pellet(&current_tile);
    //         // if newly_pellet_emptied {
    //         //     score_change.pellets_collected += 1;
    //         // } else if newly_power_pellet_emptied {
    //         //     score_change.power_pellets_collected +=1;
    //         // }

    //         if let Some(start) = player_history.iter().find(|j| *j != end) {
    //             // iterate from start..end and paint()
    //             let t1 = self.lookup_position(*start);//create tilepoints
    //             let t2 = self.lookup_position(*end);
    //             let dx = (t2.tx - t1.tx).signum();
    //             let dy = (t2.ty - t1.ty).signum();
    //             // debug_assert!(dx.abs() + dy.abs() == 1);

    //             // determine if any pellets were collected
    //             let mut newly_pellet_emptied = false;
    //             let mut newly_power_pellet_emptied = false;
    //             newly_pellet_emptied |= self.collect_pellet(&t1);
    //             newly_power_pellet_emptied |= self.collect_power_pellet(&t1);
    //             let mut t = t1.clone();
    //             // move from beginning of history to end 
    //             while t != t2 {
    //                 t = t.translate(dx, dy);
    //                 newly_pellet_emptied |= self.collect_pellet(&t);
    //                 newly_power_pellet_emptied |= self.collect_power_pellet(&t);
    //             }
    //             if newly_pellet_emptied {
    //                 // if dy.abs() > 0 {
    //                 //     score_change.pellets_collected += (t2.ty - t1.ty).abs();
    //                 // } else {
    //                 //     score_change.pellets_collected += (t2.tx - t1.tx).abs();
    //                 // }
    //                 score_change.junctions = Some((*start, *end));
    //             }
    //             if newly_power_pellet_emptied {
    //                 // if dy.abs() > 0 {
    //                 //     score_change.power_pellets_collected += (t2.ty - t1.ty).abs();
    //                 // } else {
    //                 //     score_change.power_pellets_collected += (t2.tx - t1.tx).abs();
    //                 // }
    //                 score_change.junctions = Some((*start, *end));
    //             }
    //         }
    //     }

    //     if score_change.happened() {
    //         // Don't forget this location should still be in history:
    //         let current = *player_history.front().unwrap();
    //         player_history.clear();
    //         player_history.push_front(current);
    //     }

    //     score_change
    // }
    // Change value of Tile to Empty if it was a Pellet
    pub fn collect_pellet(&mut self, tile: &TilePoint) -> bool {
        let val = &mut self.tiles[tile.ty as usize][tile.tx as usize];
        // Tile is not a pellet and no change will occur in terms of collecting pellets
        if *val != Tile::Pellet {
            false
        } else {
            *val = Tile::Empty;
            true 
        }

    }
    // Change value of Tile to Empty if it was a Power Pellet 
    pub fn collect_power_pellet(&mut self, tile: &TilePoint) -> bool {
        let val = &mut self.tiles[tile.ty as usize][tile.tx as usize];
        // Tile is not a power pellet and no change will occur in terms of collecting power pellets
        if *val != Tile::PowerPellet {
            false
        } else {
            *val = Tile::Empty;
            true 
        }
    }
    
    pub fn make_enemy(&self, ai: MovementAI, speed: i32) -> Mob {
        let fake = TilePoint::new(0, 0);
        let mut m = Mob::new(ai, fake.to_world(), speed);
        m.reset(&fake, self);
        m
    }
    pub fn lookup_position(&self, position: u32) -> TilePoint {
        let x = position % self.width;
        let y = position / self.width;
        TilePoint::new(x as i32, y as i32)
    }

    fn get_tile(&self, tile: &TilePoint) -> Tile {
        if let Some(row) = self.tiles.get(tile.ty as usize) {
            if let Some(t) = row.get(tile.tx as usize) {
                return *t;
            }
        }
        Tile::Empty
    }
    // Check if any pellets remain on the board
    pub fn board_complete(&self) -> bool {
        for row in &self.tiles {
            for tile in row {
                if tile.is_still_collectable() {
                    return false;
                }
            }
        }
        true
    }
}

// implementation of the state type 
impl State {
    // return a state or an error 
    pub fn try_new(config: &Pacman) -> Result<State, String> {
        let board = Board::try_new(&config.board)?;
        let mut config = config.clone();

        let enemies = config
            .enemies
            .iter()
            .map(|ai| board.make_enemy(ai.clone(), config.enemy_starting_speed))
            .collect();
        let player = Mob::new_player(config.player_start.to_world(), config.player_speed);
        // init the core of the state
        let core = StateCore {
            rand: random::Gen::new_child(&mut config.rand),
            lives: config.start_lives,
            score: 0,
            vulnerability_timer: 0,
            enemies_caught_multiplier: 1,
            lives_gained: 0,
            level: 1,
            player,
            enemies,
            board,
        };
        
        let mut state = State {
            config,
            state: core,
        };
        state.reset(true);
        Ok(state)
    }
    // reset the state
    pub fn reset(&mut self, player_death: bool) {
        self.state
            .player
            .reset(&self.config.player_start, &self.state.board);
        // On the default board, we imagine starting from below the initial place.
        // This way going up paints the first segment.
        // if self.config.default_board_bugs {
        //     self.state.player.history.push_front(
        //         self.state
        //             .board
        //             .get_junction_id(&TilePoint::new(31, 18))
        //             .unwrap(),
        //     );
        // }

        // commenting this out will make ghosts not reset upon catching pacman
        if !player_death{
            for enemy in &mut self.state.enemies {
                enemy.reset(&self.config.player_start, &self.state.board);
            }
        }
    }
    pub fn board_size(&self) -> WorldPoint {
        let th = self.state.board.height as i32;
        let tw = self.state.board.width as i32;
        TilePoint::new(tw + 1, th + 1).to_world()
    }
    /// Determine whether an enemy and a player are colliding and what to do about it.
    /// returns: (player_dead, enemy_caught)
    fn check_enemy_player_collision(&self, enemy: &Mob, enemy_id: usize) -> EnemyPlayerState {
        if self.state.player.position.to_tile() == enemy.position.to_tile() {
            // If ghosts are vulnerable, then player is not killed and ghost are 
            // if self.state.vulnerability_timer > 0 {
            //     if !enemy.caught {
            //         EnemyPlayerState::EnemyCatch(enemy_id)
            //     } else {
            //         EnemyPlayerState::Miss
            //     }
            //     // EnemyPlayerState::Miss
            // } else {
            //     EnemyPlayerState::PlayerDeath
            // }
            if enemy.vulnerable {
                EnemyPlayerState::EnemyCatch(enemy_id)
            } else {
                EnemyPlayerState::PlayerDeath
            // }
            }
        } else {
            // No overlap.
            EnemyPlayerState::Miss
        }
    }
    // If player/enemy is on a teleport block, teleport player to corresponding position 
    // by updating position
    fn check_teleport(&self, mob: &Mob) -> bool {
        // Get player position
        let current_tile = mob.position.to_tile();
        if self.state.board.is_tp(&current_tile) {
            true
        } else {
            false
        }
        // Check if player position 
        // board.is_tp() accepts a tilepoint and returns bool
    }
}

impl toybox_core::Simulation for Pacman {
    fn reset_seed(&mut self, seed: u32) {
        self.rand.reset_seed(seed)
    }
    fn game_size(&self) -> (i32, i32) {
        screen::GAME_SIZE
    }
    fn new_game(&mut self) -> Box<dyn toybox_core::State> {
        Box::new(State::try_new(self).expect("new_game should succeed."))
    }
    fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Pacman should be JSON serializable!")
    }
    /// Sync with [ALE impl](https://github.com/mgbellemare/Arcade-Learning-Environment/blob/master/src/games/supported/Amidar.cpp#L80)
    /// Note, leaving a call to sort in this impl to remind users that these vecs are ordered!
    fn legal_action_set(&self) -> Vec<AleAction> {
        let mut actions = vec![
            AleAction::NOOP,
            AleAction::UP,
            AleAction::RIGHT,
            AleAction::LEFT,
            AleAction::DOWN,
            AleAction::UPRIGHT,
            AleAction::UPLEFT,
            AleAction::DOWNRIGHT,
            AleAction::DOWNLEFT,
        ];
        actions.sort();
        actions
    }

    fn new_state_from_json(
        &self,
        json_str: &str,
    ) -> Result<Box<dyn toybox_core::State>, serde_json::Error> {
        let state: StateCore = serde_json::from_str(json_str)?;
        Ok(Box::new(State {
            config: self.clone(),
            state,
        }))
    }

    fn from_json(
        &self,
        json_config: &str,
    ) -> Result<Box<dyn toybox_core::Simulation>, serde_json::Error> {
        let config: Pacman = serde_json::from_str(json_config)?;
        Ok(Box::new(config))
    }

    fn schema_for_config(&self) -> String {
        let schema = schema_for!(Pacman);
        serde_json::to_string(&schema).expect("JSONSchema should be flawless.")
    }
    fn schema_for_state(&self) -> String {
        let schema = schema_for!(StateCore);
        serde_json::to_string(&schema).expect("JSONSchema should be flawless.")
    }
}
// this seems to be the where everything is ties together 
impl toybox_core::State for State
where
    State: Clone,
{
    fn lives(&self) -> i32 {
        self.state.lives
    }
    fn score(&self) -> i32 {
        self.state.score
    }
    fn level(&self) -> i32 {
        self.state.level
    }
    fn update_mut(&mut self, buttons: Input) {
        let pre_update_score: i32 = self.score();
        let history_limit = self.config.history_limit;

        if let Some(score_change) = self.state.player.update(
            buttons,
            &mut self.state.board,
            None,
            history_limit,
            &mut self.state.rand,
        ) {
            // Update score for pellets collected
            let mut allow_score_change = true;
            if allow_score_change {
                self.state.score += score_change.pellets_collected * self.config.score_increase_per_pellet;
                // Power pellets offer 
                self.state.score += score_change.power_pellets_collected * self.config.score_increase_per_power_pellet;
            }
            // If a power pellet is collected then ghosts will become vulnerable and timer will start
            if score_change.power_pellets_collected > 0 {
                self.state.vulnerability_timer = self.config.vulnerable_time;
                // Collecting a power pellet will reset enemies caught multipler to 0
                self.state.enemies_caught_multiplier = 1;
                for e in self.state.enemies.iter_mut(){
                    e.vulnerable = true;
                }
            }
        }
        // If the timer is 0, then reset vulnerability of enemies back to false
        if self.state.vulnerability_timer == 0 {
            for e in self.state.enemies.iter_mut(){
                e.vulnerable = false;
            }
        }

        // Check if the player should be teleported
        let result = self.check_teleport(&self.state.player);
        if result {
            self.state.player.teleport(&self.state.board)
        }
        // Decrement timer if active 
        if self.state.vulnerability_timer > 0 {
            self.state.vulnerability_timer -= 1;
        }

        let mut dead = false;
        let mut changes: Vec<EnemyPlayerState> = Vec::new();

        // Check-collisions after player move:
        for (i, e) in self.state.enemies.iter().enumerate() {
            let state = self.check_enemy_player_collision(e, i);
            if state != EnemyPlayerState::Miss {
                changes.push(state);
            }
            
        }
        // Check if enemies should move slower: Will move slower than Pamcan if vulnerable
        for e in self.state.enemies.iter_mut(){
            if e.vulnerable{
                e.change_speed(self.config.enemy_starting_speed-4);
            } else {
                e.change_speed(self.config.enemy_starting_speed);
            }
        }
      
     
        // Move enemies:
        for e in self.state.enemies.iter_mut() {
            e.update(
                Input::default(),
                &mut self.state.board,
                Some(self.state.player.clone()),
                history_limit,
                &mut self.state.rand,
            );        
        }
        // Check if enemies should be teleported
        let mut teleport_vec: Vec<usize> =  Vec::new();
        // Check-collisions again (so we can't run through enemies):
        for (i, e) in self.state.enemies.iter().enumerate() {
            let state = self.check_enemy_player_collision(e, i);
            if state != EnemyPlayerState::Miss {
                changes.push(state);
            }
            let result = self.check_teleport(e);
            if result{
                teleport_vec.push(i);
            }
        }
        // Teleport enemies if any should be
        let mut eid = 0; // index of enemy 
        for e in self.state.enemies.iter_mut(){
            if teleport_vec.iter().any(|&i| i == eid ) {
                e.teleport(&mut self.state.board);
            }
            eid +=1;
        }

        // Process EnemyPlayerState that were interesting!

        // Vector to make sure if an enemy is being evaluated twice
        // let mut caught_vev: Vec<usize> = Vec::new(); 
        let mut recently_caught = 5; 
        for change in changes {
            match change {
                EnemyPlayerState::Miss => {
                    // This was filtered out.
                }
                EnemyPlayerState::PlayerDeath => {
                    dead = true;
                    break;
                }
                EnemyPlayerState::EnemyCatch(eid) => {
                    // if caught_vev.iter().any(|&i| i != eid){
                    if recently_caught != eid {
                        recently_caught = eid;
                        // caught_vev.push(eid);
                        println!("Ran. ID: {}", eid);
                        // Increase score 
                        self.state.score += (self.config.score_increase_base_per_ghost_catch * self.state.enemies_caught_multiplier);
                        // Double catch multiplier
                        self.state.enemies_caught_multiplier *=2;
                        // Reset enemy, which will reset vulnerbability and position
                        self.state.enemies[eid].reset(&self.config.player_start, &self.state.board);
                    }
                    // If enemy was just caught
                    // if !self.state.enemies[eid].caught {
                    //     // If player collects multiple ghosts per power pellet, score is multiplied
                    //     // println!("Multiplier: {}", self.state.enemies_caught_multiplier );
                    //     // self.state.score += (self.config.score_increase_base_per_ghost_catch * self.state.enemies_caught_multiplier);
                    //     // self.state.enemies_caught_multiplier *=2; // increase multiplier with every ghost caught (reset when vlnerability reset)
                    //     println!("Ran");
                    //     self.state.enemies[eid].caught = true;
                    //     self.state.enemies[eid].vulnerable = false;
                    // }
                    // if self.state.enemies[eid].caught{
                    //     self.state.enemies[eid].reset(&self.config.player_start, &self.state.board);
                    //     self.state.enemies[eid].caught = false; 
                        
                    // }
                }
            }
        }
        // If Pacman has achieved a multiple of 10k score then increase lived
        if self.state.score > ((self.state.lives_gained + 1) * self.config.life_gain_threshold) {
            self.state.lives +=1; 
            self.state.lives_gained +=1;
        }

        // If dead, reset. If alive, check to see if we have advanced to the next level.
        if dead {
            self.state.lives -= 1;
            self.state.score = pre_update_score;
            self.reset(true);
        } else {
            if self.state.board.board_complete() {
                self.reset(false);
                // Increment the level
                self.state.level += 1;
                // Reset vulnerability to 0 
                self.state.vulnerability_timer = 0;
                // Reset pellets 
                self.state.board = Board::fast_new();
            }
        }
    }
    // This is where we draw out the board
    fn draw(&self) -> Vec<Drawable> {
        // Current time to be used in flashing animations 
        let mut current_time = 0; 
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => current_time = n.as_millis(),
            Err(_) => panic!("SystemTime Error"),
        }
        let mut output = Vec::new();
        output.push(Drawable::Clear(self.config.window_color));
        if self.state.lives < 0 {
            return output;
        }

        let (tile_w, tile_h) = screen::TILE_SIZE;
        let (offset_x, offset_y) = screen::BOARD_OFFSET;
        // draw the board 
        for (ty, row) in self.state.board.tiles.iter().enumerate() {
            let ty = ty as i32;
            for (tx, tile) in row.iter().enumerate() {
                let tx = tx as i32;

                // Use the level-1 sprites for odd levels less than the sixth level.
                // Use the level-2 sprites for even levels and those greater than the sixth level.
                // We will probably want to put some of this in the config later.
                // let ghosts = self.state.level % 2 == 1 && self.state.level < 6;

                if self.config.render_images {
                    let mut tile_sprite: &FixedSpriteData = match tile {
                        &Tile::Pellet=> &images::TILE_WITH_PELLET,
                        &Tile::PowerPellet=> &images::TILE_WITH_POWER_PELLET,
                        &Tile::Teleport=> &images::TILE_EMPTY,
                        &Tile::Wall=> &images::TILE_WALL,
                        &Tile::House=> &images::TILE_EMPTY,
                        &Tile::Empty => &images::TILE_EMPTY,
                    };
                    if tile == &Tile::PowerPellet {
                        if current_time % 500 > 250 {
                            tile_sprite = &images::TILE_EMPTY;
                        }
                    }
                    output.push(Drawable::sprite(
                        offset_x + tx * tile_w,
                        offset_y + ty * tile_h,
                        tile_sprite.clone(),
                    ));
                } else {
                    let tile_color = match tile {
                        &Tile::Pellet=> self.config.pellet_color,
                        &Tile::PowerPellet => self.config.power_pellet_color,
                        &Tile::Wall => self.config.fg_color,
                        &Tile::Teleport => self.config.teleport_color,
                        &Tile::Empty | &Tile::House => self.config.bg_color,
                    };
                    output.push(Drawable::rect(
                        tile_color,
                        offset_x + tx * tile_w,
                        offset_y + ty * tile_h,
                        tile_w,
                        tile_h,
                    ));
                }
            }
        }

        let (player_x, player_y) = self.state.player.position.to_screen().pixels();
        let (player_w, player_h) = screen::PLAYER_SIZE;
        let mut player_sprite = images::PACMAN_CLOSED.clone();
        if current_time % 300 > 200 {
            player_sprite = images::PACMAN_MID.clone();
        } else if current_time % 300 > 100 {
            player_sprite = images::PACMAN_OPEN.clone();
        }


        let x = true;
        if self.config.render_images | x  {
            output.push(Drawable::sprite(
                offset_x + player_x - 1,
                offset_y + player_y - 1,
                player_sprite,
            ))
        } else {
            output.push(Drawable::rect(
                self.config.player_color,
                offset_x + player_x - 1,
                offset_y + player_y - 1,
                player_w,
                player_h,
            ));
        }
        let mut eid = 0; 
        for enemy in &self.state.enemies {
            let (x, y) = enemy.position.to_screen().pixels();
            let (w, h) = screen::ENEMY_SIZE;

            if self.config.render_images {
                // output.push(Drawable::sprite(
                //     offset_x + x - 1,
                //     offset_y + y - 1,
                //     images::GHOST_RED.clone(),
                // ))
                output.push(Drawable::sprite(
                    offset_x + x - 1,
                    offset_y + y - 1,
                    if enemy.vulnerable{
                        if self.state.vulnerability_timer > 200 || self.state.vulnerability_timer % 15 >=7 {
                            images::GHOST_VULNERABLE_BLUE.clone()
                        } else {
                            images::GHOST_VULNERABLE_WHITE.clone()
                        }
                    } else {
                        if eid == 0 { 
                            if current_time % 250 > 125 {
                                images::GHOST_RED1.clone()
                            } else {
                                images::GHOST_RED2.clone()
                            }
                        } else if eid == 1 {
                            if current_time % 250 > 125 {
                                images::GHOST_GREEN1.clone()
                            } else {
                                images::GHOST_GREEN2.clone()
                            }
                        } else if eid == 2 {
                            if current_time % 250 > 125 {
                                images::GHOST_PINK1.clone()
                            } else {
                                images::GHOST_PINK2.clone()
                            }
                        } else {
                            if current_time % 250 > 125 {
                                images::GHOST_YELLOW1.clone()
                            } else {
                                images::GHOST_YELLOW2.clone()
                            }
                        }
                    }
                ));
            } else {
                output.push(Drawable::rect(
                    self.config.enemy_color,
                    offset_x + x - 1,
                    offset_y + y - 1,
                    w,
                    h,
                ));
            }
            eid +=1;
        }

        output.extend(draw_score(
            self.state.score,
            screen::SCORE_X_POS,
            screen::SCORE_Y_POS + 1,
        ));
        for i in 0..self.state.lives {
            // output.push(Drawable::rect(
            //     self.config.player_color,
            //     screen::LIVES_X_POS - i * screen::LIVES_X_STEP,
            //     screen::LIVES_Y_POS,
            //     1,
            //     DIGIT_HEIGHT + 1,
            // ))
            output.push(Drawable::sprite(
                screen::LIVES_X_POS - i * screen::LIVES_X_STEP,
                screen::LIVES_Y_POS,
                images::PACMAN_LARGE.clone(),
            ))
        }

        output
    }

    fn to_json(&self) -> String {
        serde_json::to_string(&self.state).expect("Should be no JSON Serialization Errors.")
    }

    fn query_json(&self, query: &str, args: &serde_json::Value) -> Result<String, QueryError> {
        if let Ok(parsed) = JSONQuery::parse(query) {
            if let Ok(Some(found)) = parsed.execute(&self) {
                return Ok(serde_json::to_string(&found)?);
            }
        }

        let state = &self.state;
        Ok(match query {
            "world_to_tile" => {
                let world_pt: WorldPoint = serde_json::from_value(args.clone())?;
                let tile = world_pt.to_tile();
                serde_json::to_string(&(tile.tx, tile.ty))?
            }
            "tile_to_world" => {
                let tile_pt: TilePoint = serde_json::from_value(args.clone())?;
                let world = tile_pt.to_world();
                serde_json::to_string(&(world.x, world.y))?
            }
            "num_pellets_and_power_pellets_uncollected" => {
                let mut sum = 0;
                for row in state.board.tiles.iter() {
                    sum += row
                        .iter()
                        .filter(|t| t.walkable() && t.is_still_collectable())
                        .count();
                }
                serde_json::to_string(&sum)?
            }
            // "regular_mode" => {
            //     serde_json::to_string(&(state.chase_timer == 0 && state.jump_timer == 0))?
            // }
            // "jump_mode" => serde_json::to_string(&(state.jump_timer > 0))?,
            // "chase_mode" => serde_json::to_string(&(state.chase_timer > 0))?,
            // "jumps_remaining" => serde_json::to_string(&(state.jumps > 0))?,
            "num_enemies" => serde_json::to_string(&state.enemies.len())?,
            "enemy_tiles" => {
                let positions: Vec<(i32, i32)> = state
                    .enemies
                    .iter()
                    .map(|e| {
                        let tile = e.position.to_tile();
                        (tile.tx, tile.ty)
                    })
                    .collect();
                serde_json::to_string(&positions)?
            }
            "enemy_tile" => {
                if let Some(index) = args.as_u64() {
                    let tile = state.enemies[index as usize].position.to_tile();
                    serde_json::to_string(&(tile.tx, tile.ty))?
                } else {
                    Err(QueryError::BadInputArg)?
                }
            }
            "enemy_caught" => {
                if let Some(index) = args.as_u64() {
                    let status = state.enemies[index as usize].caught;
                    serde_json::to_string(&status)?
                } else {
                    Err(QueryError::BadInputArg)?
                }
            }
            "player_tile" => {
                let tile = state.player.position.to_tile();
                serde_json::to_string(&(tile.tx, tile.ty))?
            }
            _ => Err(QueryError::NoSuchQuery)?,
        })
    }
    fn copy(&self) -> Box<dyn toybox_core::State> {
        Box::new(self.clone())
    }
}