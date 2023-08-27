use bracket_lib::prelude::*;

enum GameMode {
    Menu,
    Playing,
    End,
}

#[derive(Debug)]
enum PowerType {
    Coin {value:i32},
    Slow,
    Gap,
    Gravity,
}

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const FRAME_DURATION: f32 = 75.0;
const PLAYER_OFFSET: i32 = 10;

struct Player {
    x: i32,
    y: i32,
    velocity: f32,
}
impl Player {
    fn new(x:i32, y:i32) -> Self {
        Self { x, y, velocity: 0.0, }
    }

    fn render(&self, ctx: &mut BTerm) {
        ctx.set( PLAYER_OFFSET, self.y, GREEN, BLACK, to_cp437('@'));
    }

    fn gravity_and_movement(&mut self) {
        if self.velocity < 2.0 {
            self.velocity += 0.2;
        }

        self.y += self.velocity as i32;
        self.x += 1;
        if self.y < 0 {
            self.y = 0;
        }
    }

    fn flap(&mut self) {
        self.velocity = -2.0;
    }
}

struct Obstacle {
    x: i32,
    gap_y: i32,
    size: i32,
}
impl Obstacle {
    fn new(x:i32, score:i32) -> Self {
        let mut random = RandomNumberGenerator::new();
        Self {
            x,
            gap_y: random.range(10, 40),
            size: i32::max(2, 20-score),
        }
    }

    fn render(&self, ctx: &mut BTerm, player_x : i32) {
        let screen_x = self.x - player_x + PLAYER_OFFSET;
        let half_size = self.size / 2;

        // Draw the top half of the obstacle
        for y in 0..self.gap_y - half_size {
            ctx.set(screen_x, y, RED, BLACK, to_cp437('|'),);
        }
        // Draw the bottom half of the obstacle
        for y in self.gap_y + half_size .. SCREEN_HEIGHT {
            ctx.set(screen_x, y, RED, BLACK, to_cp437('|'),);
        }
    }

    fn hit_obstacle(&self, player: &Player) -> bool {
        let half_size = self.size / 2;
        let does_x_match = player.x == self.x;
        let player_above_gap = player.y < self.gap_y - half_size;
        let player_below_gap = player.y > self.gap_y + half_size;
        does_x_match && (player_above_gap || player_below_gap)
    }
}

#[derive(Debug)]
struct Powerup {
    x: i32,
    y: i32,
    power: PowerType,
}
impl Powerup {
    fn new(x:i32) -> Self {
        let mut random: RandomNumberGenerator = RandomNumberGenerator::new();
        Self { x, 
            y: random.range(10, 60), 
            power: {let power_selector = random.range(0,4);
                match power_selector {
                    0 => PowerType::Coin{value: random.range(2,7)},
                    1 => PowerType::Slow,
                    2 => PowerType::Gap,
                    _ => PowerType::Gravity,
                } 
                
            }
        }
    }

    fn render(&self, ctx: &mut BTerm, player_x: i32) {
        let screen_x = self.x - player_x + PLAYER_OFFSET;
        match self.power {
            PowerType::Coin{value} => {
                    ctx.set(screen_x, self.y, YELLOW, BLACK, to_cp437( 
                        match char::from_digit(value as u32, 10) {
                            Some(c) => c,
                        _ => '#'}),);
                },
            PowerType::Gravity => {ctx.set(screen_x, self.y, YELLOW, BLACK, to_cp437('*'))},
            PowerType::Gap => {ctx.set(screen_x, self.y, YELLOW, BLACK, 23 as u16)},
            _ => {ctx.set(screen_x, self.y, YELLOW, BLACK, to_cp437('>'))} //TODO implement other power ups
        }
    }

    fn activate(&self, player: &Player) -> bool {
        let x_does_match = player.x == self.x;
        let y_does_match = player.y <= self.y+2 && player.y >= self.y-2;

        x_does_match && y_does_match
    }
}

struct State {
    player: Player,
    frame_time: f32,
    mode: GameMode,
    obstacle: Obstacle,
    score: i32,
    powerups: Vec<Powerup>,
}
impl State {
    fn new() -> Self {
        Self {
            player: Player::new(5, 25),
            frame_time: 0.0,
            mode: GameMode::Menu,
            obstacle: Obstacle::new(SCREEN_WIDTH, 0),
            score: 0,
            powerups: Vec::new(),
        }
    }

    fn restart(&mut self) {
        self.player = Player::new(5, 25);
        self.frame_time = 0.0;
        self.mode = GameMode::Playing;
        self.obstacle = Obstacle::new(SCREEN_WIDTH, 0);
        self.score = 0;
    }

    fn play(&mut self, ctx: &mut BTerm) {
        ctx.cls_bg(NAVY);
        // general
        ctx.print(0, 0, "Press SPACE to flap!");
        ctx.print(0, 1, &format!("Score: {}", self.score));
        self.frame_time += ctx.frame_time_ms;
        if self.frame_time > FRAME_DURATION {
            self.frame_time  = 0.0;
            self.player.gravity_and_movement();
        }

        // input
        if let Some(VirtualKeyCode::Space) = ctx.key {
            self.player.flap();
        }

        // draw
        self.render_all(ctx);

        // events
        // if an obstacle is succesfully passed
        self.pass_obstacle();

        // game over
        self.end_game(ctx);

        // if powerup is activated
        self.activate_powerup();
        
        // clear out of scope powerups
        self.powerups.retain(|current| current.x > self.player.x - PLAYER_OFFSET);

        // create new powerups
        let mut random = RandomNumberGenerator::new();
        let create_powerup = random.range(0,10) == 0;
        if create_powerup {self.powerups.push(Powerup::new(self.player.x + SCREEN_WIDTH));}
        
    }

    fn activate_powerup(&mut self) {
        // #![feature(extract_if)]
        // let current_powerup = self.extract_if(|item| item.activate(&self.player)).collect::<Vec<Powerup>>();
        
        if !self.powerups.is_empty(){
            let mut index: Option<usize> = None;
            for (i, item) in self.powerups.iter().enumerate() {
                if item.activate(&self.player) {
                    match item.power {
                        PowerType::Coin { value } => {self.score = self.score + value;println!("HIT")},
                        _ => {}
                    }
                    index = Some(i);
                }
            }
            if let Some(value) = index {self.powerups.remove(value);
            } else {}
        }
    }

    fn pass_obstacle(&mut self) {
        if self.player.x > self.obstacle.x {
            self.score += 1;
            self.obstacle = Obstacle::new(self.player.x + SCREEN_WIDTH, self.score);
        }
    }

    fn end_game(&mut self, ctx: &BTerm) {
        let off_screen = self.player.y > SCREEN_HEIGHT;
        let hit_obstacle = self.obstacle.hit_obstacle(&self.player);
        let mut press_escape = false;

        if let Some(VirtualKeyCode::Escape) = ctx.key {
            press_escape = true;
        }

        if  off_screen || hit_obstacle || press_escape{
            self.mode = GameMode::End;
        }
    }

    fn render_all(&self, ctx: &mut BTerm) {
        self.player.render(ctx);
        self.obstacle.render(ctx, self.player.x);
        if !self.powerups.is_empty(){
            for item in &self.powerups {
                item.render(ctx, self.player.x);
            }
        }
    }

    fn main_menu(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "Welcome to Pias Flappy Dragon");
        ctx.print_centered(8, "(P)lay Game");
        ctx.print_centered(9, "(Q)uit the Game");

        if let Some(key) = ctx.key { 
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }

    fn dead(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "You are dead");
        ctx.print_centered(6, format!("You earned {} points!", self.score));
        ctx.print_centered(8, "(M)ain Menu");
        ctx.print_centered(9, "(P)lay Again");
        ctx.print_centered(10, "(Q)uit Game");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::M => self.mode = GameMode::Menu,
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx:&mut BTerm) {
        match self.mode {
            GameMode::Menu => self.main_menu(ctx),
            GameMode::End => self.dead(ctx),
            GameMode::Playing => self.play(ctx)
        }
    }
}

fn main() -> BError {
    let context = BTermBuilder::simple80x50()
        .with_title("Pias Flappy Dragon")
        .build()?;
    
    main_loop(context, State::new())
}
