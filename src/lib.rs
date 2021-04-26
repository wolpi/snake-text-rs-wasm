mod utils;

mod objects;
use crate::objects::Direction;
use crate::objects::Command;
use crate::objects::Point;
use crate::objects::Snake;

use std::time::{Duration};

use wasm_bindgen::prelude::*;

extern crate js_sys;
extern crate web_sys;


// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;


// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}


const MAX_INTERVAL: u16 = 700;
const MIN_INTERVAL: u16 = 200;
const MAX_SPEED: u16 = 20;


#[wasm_bindgen]
pub struct Game {
    width: u16,
    height: u16,
    food: Option<objects::Point>,
    snake: Snake,
    speed: u16,
    score: u16,
    screen_buffer: Vec<u16>,
    tmp_utf16_buffer: [u16;2],
}

#[wasm_bindgen]
impl Game {
    pub fn default() -> Self {
        Self::new(30, 10, 2)
    }
    pub fn new(width: u16, height: u16, speed: u16) -> Self {
        utils::set_panic_hook();
        let rand_direction = random_int(0,4);
        Self {
            width,
            height,
            speed,
            food: None,
            snake: Snake::new(
                Point::new(width / 2, height / 2),
                3,
                match rand_direction {
                    0 => Direction::Up,
                    1 => Direction::Right,
                    2 => Direction::Down,
                    _ => Direction::Left
                },
            ),
            score: 0,
            // 32 -> space
            // 95 -> _
            screen_buffer: vec![32; (width * height + height + width) as usize],
            tmp_utf16_buffer: [0 as u16; 2]
        }
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn speed(&self) -> u16 {
        self.speed
    }

    pub fn score(&self) -> u16 {
        self.score
    }

    pub fn screen_buffer(&self) -> *const u16 {
        self.screen_buffer.as_ptr()
    }

    pub fn screen_buffer_len(&self) -> usize {
        self.screen_buffer.len()
    }

    pub fn tick(&mut self, key_code: &str) -> bool {
        log!("wasm tick(key_code: {})", key_code);
        self.place_food();

        let mut done = false;

        let interval = self.calculate_interval();
        let direction = self.snake.get_direction();

        let interval_millis_u128 = interval.as_millis();
        let interval_millis :u32;
        if interval_millis_u128 > u32::MAX as u128 {
            panic!("too much millis :(");
        } else {
            interval_millis = interval_millis_u128 as u32;
        }

        let start = js_sys::Date::new_0().get_milliseconds();
        let mut elapsed = 0;
        while elapsed < interval_millis {
            let millis = js_sys::Date::new_0().get_milliseconds();
            elapsed = millis - start;
        }

        if let Some(command) = self.get_command(key_code) {
            match command {
                Command::Turn(towards) => {
                    if direction != towards && direction.opposite() != towards {
                        self.snake.set_direction(towards);
                    }
                }
            }
        }

        if self.has_collided_with_wall() || self.has_bitten_itself() {
            done = true;
        } else {
            self.snake.slither();

            if let Some(food_point) = self.food {
                if self.snake.get_head_point() == food_point {
                    self.snake.grow();
                    self.food = None;
                    self.place_food();
                    self.score += 1;

                    if self.speed < MAX_SPEED && self.score % 2 == 0 {
                        self.speed += 1;
                    }
                }
            }
        }
        done
    }


    fn calculate_interval(&self) -> Duration {
        let speed = MAX_SPEED - self.speed;
        Duration::from_millis(
            (MIN_INTERVAL + (((MAX_INTERVAL - MIN_INTERVAL) / MAX_SPEED) * speed)) as u64
        )
    }

    fn get_command(&self, code: &str) -> Option<Command> {
        match code {
            "KeyW" => Some(Command::Turn(Direction::Up)),
            "KeyD" => Some(Command::Turn(Direction::Right)),
            "KeyS" => Some(Command::Turn(Direction::Down)),
            "KeyA" => Some(Command::Turn(Direction::Left)),
            _ => None
        }
    }

    fn has_collided_with_wall(&self) -> bool {
        let head_point = self.snake.get_head_point();

        match self.snake.get_direction() {
            Direction::Up => head_point.y == 1,
            Direction::Right => head_point.x == self.width - 2,
            Direction::Down => head_point.y == self.height - 1,
            Direction::Left => head_point.x == 1,
        }
    }

    fn has_bitten_itself(&self) -> bool {
        let next_head_point = self.snake.get_head_point().transform(self.snake.get_direction(), 1);
        let mut next_body_points = self.snake.get_body_points().clone();
        next_body_points.remove(next_body_points.len() - 1);
        next_body_points.remove(0);

        next_body_points.contains(&next_head_point)
    }

    fn place_food(&mut self) {
        if self.food == None {
            loop {
                let random_x = random_int(1,self.width - 1);
                let random_y = random_int(1,self.height - 1);
                let point = Point::new(random_x, random_y);


                if !self.snake.contains_point(&point) {
                    self.food = Some(point);
                    break;
                }
            }
        }
    }


    pub fn draw(&mut self) {
        self.draw_background();
        self.draw_snake();
        self.draw_food();
        self.draw_borders();
    }

    fn draw_snake(&mut self) {
        let body_points = self.snake.get_body_points();
        for (i, body) in body_points.iter().enumerate() {
            let previous = if i == 0 { None } else { body_points.get(i - 1) };
            let next = body_points.get(i + 1);
            let symbol = if let Some(&next) = next {
                if let Some(&previous) = previous {
                    if previous.x == next.x {
                        '║'
                    } else if previous.y == next.y {
                        '═'
                    } else {
                        let d = body.transform(Direction::Down, 1);
                        let r = body.transform(Direction::Right, 1);
                        let u = if body.y == 0 { body.clone() } else { body.transform(Direction::Up, 1) };
                        let l = if body.x == 0 { body.clone() } else { body.transform(Direction::Left, 1) };
                        if (next == d && previous == r) || (previous == d && next == r) {
                            '╔'
                        } else if (next == d && previous == l) || (previous == d && next == l) {
                            '╗'
                        } else if (next == u && previous == r) || (previous == u && next == r) {
                            '╚'
                        } else {
                            '╝'
                        }
                    }
                } else {
                    'O'
                }
            } else if let Some(&previous) = previous {
                if body.y == previous.y {
                    '═'
                } else {
                    '║'
                }
            } else {
                panic!("Invalid snake body point.");
            };

            self.write_at_coords(&symbol, &body.x, &body.y);
        }
    }

    fn draw_food(&mut self) {
        let food = self.food.unwrap_or(Point::new(0, 0));
        if food.x > 0 && food.y > 0 {
            self.write_at_coords(&'•', &food.x, &food.y);
        }
    }

    fn draw_background(&mut self) {
        for y in 1..self.height + 1 {
            for x in 1..self.width + 1 {
                self.write_at_coords(&' ', &x, &y);
            }
        }
    }

    fn draw_borders(&mut self) {
        let width = self.width;
        let height = self.height;
        for y in 0..self.height {
            self.write_at_coords(&'#', &(0 as u16), &y);
            self.write_at_coords(&'#', &width, &y);
            self.write_at_coords(&'\n', &(width + 1), &y);
        }

        for x in 0..self.width {
            self.write_at_coords(&'#', &x, &(0 as u16));
            self.write_at_coords(&'#', &x, &height);
        }

    }

    fn write_at_coords(&mut self, symbol: &char, x: &u16, y: &u16) {
        let mut index = x + y * self.width;
        // add linebreak offset
        index += y;

        // outer border and line break
        if *x >= self.width {
            index -= 1;
        }

        self.write_in_screen_buffer(symbol, &index);
    }

    fn write_in_screen_buffer(&mut self, symbol: &char, index: &u16) {
        symbol.encode_utf16(&mut self.tmp_utf16_buffer);
        self.screen_buffer[*index as usize] = self.tmp_utf16_buffer[0];
        //log!("    wasm screen buffer, symbol: {}, u16 0: {}, u16 1: {}", symbol, ptr[0], ptr[1]);
    }
}

fn random_int(min :u16, max :u16) -> u16 {
    js_sys::Math::floor(js_sys::Math::random() * max as f64 - min as f64) as u16 + min
}
