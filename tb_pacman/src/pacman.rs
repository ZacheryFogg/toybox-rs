use super::digit_sprites::{draw_score, DIGIT_HEIGHT};
use crate::types::*;
use access_json::JSONQuery;
use serde_json;
use std::collections::{HashSet, VecDeque};
use toybox_core;
use toybox_core::graphics::{Color, Drawable, FixedSpriteData};
use toybox_core::random;
use toybox_core::{AleAction, Direction, Input, QueryError};
use rand::seq::SliceRandom;

// Module contains basic constants related to GUI
pub mod screen {
    pub const GAME_SIZE: (i32, i32) = (250, 250); // Size of GUI window
    pub const BOARD_OFFSET: (i32, i32) = (15,15); // Game offset from top-left corner of GUI
    pub const PLAYER_SIZE: (i32, i32) = (7, 7); // Size of non-sprite player - GUI size only - not collision boxes
    pub const ENEMY_SIZE: (i32, i32) = (7, 7);
    pub const TILE_SIZE: (i32, i32) = (4, 5); // Size of each tile - GUI and Collision 
    pub const LIVES_Y_POS: i32 = 198; // Position of lives markers
    pub const LIVES_X_POS: i32 = 148;
    pub const LIVES_X_STEP: i32 = 16; 
    pub const SCORE_Y_POS: i32 = 198;
    pub const SCORE_X_POS: i32 = LIVES_X_POS - LIVES_X_STEP * 3 - 8;
}


// Module contains images for game GUI. Relevant if render_images == True
pub mod raw_images {
    pub const PACMAN: &[u8] = include_bytes!("resources/pacman/ .png");
    pub const GHOST_RED: &[u8] = include_bytes!("resources/pacman/ .png");
    pub const GHOST_PINK: &[u8] = include_bytes!("resources/pacman/ .png");
    pub const GHOST_GREEN: &[u8] = include_bytes!("resources/pacman/ .png");
    pub const GHOST_YELLOW: &[u8] = include_bytes!("resources/pacman/ .png");
    pub const GHOST_VULNERABLE: &[u8] = include_bytes!("resources/pacman/ .png");
    pub const GHOST_FLASHING: &[u8] = include_bytes!("resources/pacman/ .png");
}


// Module creates static references to images
pub mod images {
    use super::*;
    lazy_static! { //lazy_static to allow other variables to be loaded
        pub static ref PACMAN: FixedSpriteData = FixedSpriteData::load_png(raw_images::PACMAN);
        pub static ref GHOST_RED: FixedSpriteData = FixedSpriteData::load_png(raw_images::GHOST_RED);
        pub static ref GHOST_PINK: FixedSpriteData = FixedSpriteData::load_png(raw_images::GHOST_PINK);
        pub static ref GHOST_GREEN: FixedSpriteData = FixedSpriteData::load_png(raw_images::GHOST_GREEN);
        pub static ref GHOST_YELLOW: FixedSpriteData = FixedSpriteData::load_png(raw_images::GHOST_YELLOW);
        pub static ref GHOST_VULNERABLE: FixedSpriteData = FixedSpriteData::load_png(raw_images::GHOST_VULNERABLE);
        pub static ref GHOST_FLASHING: FixedSpriteData = FixedSpriteData::load_png(raw_images::GHOST_FLASHING);
    }
}


// Module creates variables related to World (world is internal, while screen in GUI)
mod world {
    use super::screen; // Import variable from screen module
    pub const SCALE: i32 = 16;
    pub const TILE_SIZE: (i32, i32) = (screen::TILE_SIZE.0 * SCALE, screen::TILE_SIZE.1 * SCALE);
}


// Load board from file
pub const PACMAN_BOARD: &str = include_str!("resources/pacman_default_board");


// Module intiates speed of mobs 
mod inits {
    pub const ENEMY_STARTING_SPEED: i32 = 10;
    pub const PLAYER_SPEED: i32 = 8;
}


// Implement Pacman type
impl Pacman {
    // Using internal variables, return a vector color addresses
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
            rand: random::Gen::new_from_seed(13)
            board: PACMAN_BOARD.lines().map(|s| s.to_owned()).collect(), // Vec<String> 
            player_start: TilePoint::new(12, 15), //Start Tile of Player
            bg_color: Color::rgb(31, 41, 148),
            fg_color: Color::rgb(242, 124, 124),
            pellet_color: Color:rgb(252, 186, 3),
            power_pellet_color: Color::rgb(245, 129, 66),
            teleport_color: Color::rgb(200, 66, 245),
            player_color: Color::rgb(245, 233, 66),
            enemy_color: Color::rgb(255,0,0),
            start_lives: 3,
            render_images: false,
            enemy_starting_speed: inits::ENEMY_STARTING_SPEED,
            player_speed: inits::PLAYER_SPEED,
            vulnerable_time: 300
            enemies: vec![MovementAI::EnemyRandomMvmt,  //4 random agents for now
                          MovementAI::EnemyRandomMvmt,
                          MovementAI::EnemyRandomMvmt,
                          MovementAI::EnemyRandomMvmt]
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

// Implement GridBox type: represents boxes/tile on the board
// impl GridBox {
//     /// 
//     fn new(top_left: TilePoint, bottom_right: TilePoint, pellet: bool, power_pellet: bool) -> GridBox {
//         GridBox {
//             top_left,
//             bottom_right,
//             pellet, 
//             power_pellet,
//         }
//     }
//     // Check if TilePoint is in GridBox 
//     fn matches(&self, tile: &TilePoint) -> bool {
//         let x1 = self.top_left.tx;
//         let x2 = self.bottom_right.tx;
//         let y1 = self.top_left.ty;
//         let y2 = self.bottom_right.ty;

//         let xq = tile.tx;
//         let yq = tile.ty;

//         (x1 <= xq) && (xq <= x2) && (y1 <= yq) && (yq <= y2)
//     }
//     // Check whether or not this gridbox should be updated: pellet collected
//     fn should_update_paint(&self, board: &Board) -> bool {
//         if self.pellet == false && self.power_pellet == false{ // If empty then no update required
//             return false;
//         }

//         let x1 = self.top_left.tx;
//         let x2 = self.bottom_right.tx;
//         let y1 = self.top_left.ty;
//         let y2 = self.bottom_right.ty;

//         // Determine using board if pellet was collected
//         false
//     }
// }

// Implement Tile type: 
impl Tile {
    fn new_from_char(c: char) -> Result<Tile, String> {
    // Convert character into Tile of certain type
        match c {
            'e' => Ok(Tile::Empty),
            '=' => Ok(Tile::Pellet), // Empty + pellet
            'p' => Ok(Tile::PowerPellet), // Empty + power pellet
            '#' => Ok(Tile::Wall), // Non movelable area 
            'h' => Ok(Tile:House), // Enemies home
            'l' => Ok(Tile:LeftTeleport),
            'r' => Ok(Tile:RightTeleport),
            _ => Err(format!("Cannot construct AmidarTile from '{}'", c)),
        }
    }
    // Tile can be walked on
    pub fn walkable(self) -> bool {
        match self{
            Tile::House | Tile::Wall => false,
            Tile::Pellet | Tile::PowerPellet | Tile::Empty | Tile::LeftTeleport | Tile::RightTeleport => true,
        }
    }

    // Tile contains a pellet or power pellet 
    pub fn is_stil_collectable(self) -> bool {
        match self {
            Tile::Pellet | Tile::PowerPellet => true,
            Tile::Empty | Tile::Wall | Tile::House | Tile::LeftTeleport | Tile::RightTeleport => false,
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
    // reset agent in world to it's start
    fn reset(&mut self, player_start: &TilePoint, board: &Board) {
        self.step = None;
        self.ai.reset();
        self.position = match self.ai {
            MovementAI::Player => player_start.to_world(),
            MovementAI::EnemyRandomMvmt { ref start, .. } => start.clone().to_world(),
        };
        self.history.clear();
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

        // Manage history:
        if self.is_player() {
            // Check if based on player history any pellets were collected
            board.check_pellets(&mut self.history).into_option()
        } else {
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
            ghost_consumed: false,
        }
    }//maybe indicates if some significant event happened
    fn happened(&self) -> bool {
        self.junctions.is_some()
            || self.pellets_collected != 0
            || self.power_pellets_collected != 0
            || self.ghost_consumed != 0
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
    fn is_pellet(&self, xy: &TilePoint) -> bool {
        self.get_tile(xy) == Tile::Pellet
    }
    fn is_power_pellet(&self, xy: &TilePoint) -> bool {
        self.get_tile(xy) == Tile::PowerPellet
    }
    fn is_left_tp(&self, xy: &TilePoint) -> bool {
        self.get_tile(xy) == Tile::LeftTeleport
    }
    fn is_right_tp(&self, xy: &TilePoint) -> bool {
        self.get_tile(xy) == Tile::RightTeleport
    }
    fn is_home(&self, xy: &TilePoint) -> bool {
        self.get_tile(xy) == Tile::Home
    }
    fn is_wall(&self, xy: &TilePoint) -> bool {
        self.get_tile(xy) == Tile::Wall
    }

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

    // This function takes in the players history to determine a board update
    fn check_pellets(&mut self, player_history: &mut VecDeque<u32>) -> BoardUpdate {
        // Init a BoardUpdate to represent the score_change
        let mut score_change = BoardUpdate::new();
        
        // Iterate through player history and collect pellets
        if let Some(end) = player_history.front() {
            if let Some(start) = player_history.iter().find(|j| *j != end) {
                // iterate from start..end and paint()

                let t1 = self.lookup_position(*start);//create tilepoints
                let t2 = self.lookup_position(*end);
                let dx = (t2.tx - t1.tx).signum();
                let dy = (t2.ty - t1.ty).signum();
                debug_assert!(dx.abs() + dy.abs() == 1);

                // determine if any pellets were collected
                let mut newly_emptied = false;
                newly_pellet_emptied |= self.collect_pellet(&t1)
                newly_power_pellet_emptied |= self.collect_power_pellet(&t1);
                let mut t = t1.clone();
                // move from beginning of history to end 
                while t != t2 {
                    t = t.translate(dx, dy);
                    newly_pellet_emptied |= self.collect_pellet(&t)
                    newly_power_pellet_emptied |= self.collect_power_pellet(&t);
                }
                if newly_pellet_emptied {
                    if dy.abs() > 0 {
                        score_change.pellets_collected += (t2.ty - t1.ty).abs();
                    } else {
                        score_change.pellets_collected += (t2.tx - t1.tx).abs();
                    }
                    score_change.junctions = Some((*start, *end));
                }
                if newly_power_pellet_emptied {
                    if dy.abs() > 0 {
                        score_change.power_pellets_collected += (t2.ty - t1.ty).abs();
                    } else {
                        score_change.power_pellets_collected += (t2.tx - t1.tx).abs();
                    }
                    score_change.junctions = Some((*start, *end));
                }
            }
        }

        if score_change.happened() {
            // Don't forget this location should still be in history:
            let current = *player_history.front().unwrap();
            player_history.clear();
            player_history.push_front(current);
        }

        score_change
    }
    // Change value of Tile to Empty if it was a Pellet
    pub fn collect_pellet(&mut self, tile: &TilePoint) -> bool {
        let val = &mut self.tiles[tile.ty as usize][tile.tx as usize];
        // Tile is not a pellet and no change will occur in terms of collecting pellets
        if *val != Tile::Pellet {
            false
        } else {
            *val = Tile::Empty
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
            *val = Tile::Empty
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
                if tile.is_stil_collectable() {
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
            enemies_vulenerable: false,
            level: 1,
            player,
            enemies,
            board,
        };
        
        let mut state = State {
            config,
            state: core,
        };
        state.reset();
        Ok(state)
    }
    // reset the state
    pub fn reset(&mut self) {
        self.state
            .player
            .reset(&self.config.player_start, &self.state.board);
        // On the default board, we imagine starting from below the initial place.
        // This way going up paints the first segment.
        if self.config.default_board_bugs {
            self.state.player.history.push_front(
                self.state
                    .board
                    .get_junction_id(&TilePoint::new(31, 18))
                    .unwrap(),
            );
        }
        for enemy in &mut self.state.enemies {
            enemy.reset(&self.config.player_start, &self.state.board);
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
            // // If ghosts are vulnerable, then player is not killed and ghost are 
            // if self.state.chase_timer > 0 {
            //     if !enemy.caught {
            //         EnemyPlayerState::EnemyCatch(enemy_id)
            //     } else {
            //         EnemyPlayerState::Miss
            //     }
            // } else if self.state.jump_timer > 0 {
            //     EnemyPlayerState::Miss
            // } else {
            //     EnemyPlayerState::PlayerDeath
            // }
            EnemyPlayerState::PlayerDeath
        } else {
            // No overlap.
            EnemyPlayerState::Miss
        }
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
            AleAction::FIRE,
            AleAction::UP,
            AleAction::RIGHT,
            AleAction::LEFT,
            AleAction::DOWN,
            AleAction::UPFIRE,
            AleAction::RIGHTFIRE,
            AleAction::LEFTFIRE,
            AleAction::DOWNFIRE,
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

        // Move the player and determine whether the board changes.
        if let Some(score_change) = self.state.player.update(
            buttons,
            &mut self.state.board,
            None,
            history_limit,
            &mut self.state.rand,
        ) {
           
            let mut allow_score_change = true;
            if allow_score_change {
                self.state.score += score_change.pellets_collected;
                // max 1 point for vertical, for some reason.
                self.state.score += score_change.power_pellets_collected;
            }

            if score_change.power_pellets_collected > 0 {
                self.state.vulnerability_timer = self.config.vulnerable_time;
            }
        }

        if self.state.vulnerability_timer > 0 {
            self.state.vulnerability_timer -= 1;

        let mut dead = false;
        let mut changes: Vec<EnemyPlayerState> = Vec::new();

        // check-collisions after player move:
        for (i, e) in self.state.enemies.iter().enumerate() {
            let state = self.check_enemy_player_collision(e, i);
            if state != EnemyPlayerState::Miss {
                changes.push(state);
            }
        }

        // move enemies:
        for e in self.state.enemies.iter_mut() {
            e.update(
                Input::default(),
                &mut self.state.board,
                Some(self.state.player.clone()),
                history_limit,
                &mut self.state.rand,
            );
        }

        // check-collisions again (so we can't run through enemies):
        for (i, e) in self.state.enemies.iter().enumerate() {
            let state = self.check_enemy_player_collision(e, i);
            if state != EnemyPlayerState::Miss {
                changes.push(state);
            }
        }

        // Process EnemyPlayerState that were interesting!
        for change in changes {
            match change {
                EnemyPlayerState::Miss => {
                    // This was filtered out.
                }
                EnemyPlayerState::PlayerDeath => {
                    dead = true;
                    break;
                }
                // EnemyPlayerState::EnemyCatch(eid) => {
                //     if !self.state.enemies[eid].caught {
                //         self.state.score += self.config.chase_score_bonus;
                //         self.state.enemies[eid].caught = true;
                //     }
                // }
            }
        }
        // If score >= 10,000 increase life by 1 

        // If dead, reset. If alive, check to see if we have advanced to the next level.
        if dead {
            self.state.lives -= 1;
            self.state.score = pre_update_score;
            self.reset();
        } else {
            if self.state.board.board_complete() {
                self.reset();
                // Increment the level
                self.state.level += 1;
                // If we triggered the chase counter immediately before
                // advancing, it will still be on and will mess up the sprites. Reset to 0.
                self.state.vulnerability_timer = 0;
                // Time to paint again!
                self.state.board = Board::fast_new();
                // If you successfully complete a level, you can get a life back (up the maximum)
                // if self.lives() < self.config.start_lives {
                //     self.state.lives += 1;
                // }
                // if self.state.level > 2 {
                //     // Starting at level 3, there are six enemies.
                //     // We haven't observed an agent that can get to level 3 and can't find any description
                //     // of what level 3 looks like, so we are leaving this blank for now.
                // }
                // Increase enemy speed.
                // Make pretty later
                // let new_speed = {
                //     if self.state.level < 3 {
                //         self.config.enemy_starting_speed
                //     } else if self.state.level < 5 {
                //         self.config.enemy_starting_speed + 2
                //     } else {
                //         self.config.enemy_starting_speed + 4
                //     }
                // };
                // for e in &mut self.state.enemies {
                //     e.change_speed(new_speed);
                // }
            }
        }
    }
    // This is where we draw out the board
    fn draw(&self) -> Vec<Drawable> {
        let mut output = Vec::new();
        output.push(Drawable::Clear(self.config.bg_color));
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
                let ghosts = self.state.level % 2 == 1 && self.state.level < 6;

                if self.config.render_images {
                    // let tile_sprite: &FixedSpriteData = match tile {
                    //     &Tile::Painted => {
                    //         if ghosts {
                    //             &images::BLOCK_TILE_PAINTED_L1
                    //         } else {
                    //             &images::BLOCK_TILE_PAINTED_L2
                    //         }
                    //     }
                    //     &Tile::Unpainted | &Tile::ChaseMarker => {
                    //         if ghosts {
                    //             &images::BLOCK_TILE_UNPAINTED_L1
                    //         } else {
                    //             &images::BLOCK_TILE_UNPAINTED_L2
                    //         }
                    //     }
                    //     &Tile::Empty => continue,
                    // };
                    // output.push(Drawable::sprite(
                    //     offset_x + tx * tile_w,
                    //     offset_y + ty * tile_h,
                    //     tile_sprite.clone(),
                    // ));
                } else {
                    let tile_color = match tile {
                        &Tile::Pellet=> self.config.pellet_color,
                        &Tile::PowerPellet => self.config.power_pellet_color,
                        &Tile::Wall => self.config.fg_color
                        &Tile::RightTeleport | &Tile::LeftTeleport => self.config.teleport_color,
                        &Tile::Empty | &Tile::Home => self.config.bg_color,
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
        let player_sprite = images::PACMAN.clone()

        if self.config.render_images {
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

        for enemy in &self.state.enemies {
            let (x, y) = enemy.position.to_screen().pixels();
            let (w, h) = screen::ENEMY_SIZE;
            output.push(Drawable::rect(
                        self.config.enemy_color,
                        offset_x + x - 1,
                        offset_y + y - 1,
                        w,
                        h,
                    ));
            // if self.config.render_images {
            //     output.push(Drawable::sprite(
            //         offset_x + x - 1,
            //         offset_y + y - 1,
            //         if self.state.chase_timer > 0 {
            //             if enemy.caught {
            //                 match self.state.level % 2 {
            //                     1 => images::ENEMY_CAUGHT_L1.clone(),
            //                     0 => images::ENEMY_CAUGHT_L2.clone(),
            //                     _ => unreachable!(),
            //                 }
            //             } else {
            //                 match self.state.level % 2 {
            //                     1 => images::ENEMY_CHASE_L1.clone(),
            //                     0 => images::ENEMY_CHASE_L2.clone(),
            //                     _ => unreachable!(),
            //                 }
            //             }
            //         } else if self.state.jump_timer > 0 {
            //             match self.state.level % 2 {
            //                 1 => images::ENEMY_JUMP_L1.clone(),
            //                 0 => images::ENEMY_JUMP_L2.clone(),
            //                 _ => unreachable!(),
            //             }
            //         } else {
            //             match self.state.level % 2 {
            //                 1 => images::ENEMY_L1.clone(),
            //                 0 => images::ENEMY_L2.clone(),
            //                 _ => unreachable!(),
            //             }
            //         },
            //     ))
            // } else {
            //     output.push(Drawable::rect(
            //         self.config.enemy_color,
            //         offset_x + x - 1,
            //         offset_y + y - 1,
            //         w,
            //         h,
            //     ));
            // }
        }

        output.extend(draw_score(
            self.state.score,
            screen::SCORE_X_POS,
            screen::SCORE_Y_POS + 1,
        ));
        for i in 0..self.state.lives {
            output.push(Drawable::rect(
                self.config.player_color,
                screen::LIVES_X_POS - i * screen::LIVES_X_STEP,
                screen::LIVES_Y_POS,
                1,
                DIGIT_HEIGHT + 1,
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
            "num_tiles_unpainted" => {
                let mut sum = 0;
                for row in state.board.tiles.iter() {
                    sum += row
                        .iter()
                        .filter(|t| t.walkable() && t.needs_paint())
                        .count();
                }
                serde_json::to_string(&sum)?
            }
            "regular_mode" => {
                serde_json::to_string(&(state.chase_timer == 0 && state.jump_timer == 0))?
            }
            "jump_mode" => serde_json::to_string(&(state.jump_timer > 0))?,
            "chase_mode" => serde_json::to_string(&(state.chase_timer > 0))?,
            "jumps_remaining" => serde_json::to_string(&(state.jumps > 0))?,
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