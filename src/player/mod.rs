use std::clone::Clone;

use ggez::{Context, GameResult};
use num_derive::FromPrimitive;
use num_traits::clamp;

use crate::caret::CaretType;
use crate::common::{Condition, Direction, Equipment, Flag, interpolate_fix9_scale, Rect};
use crate::entity::GameEntity;
use crate::frame::Frame;
use crate::input::dummy_player_controller::DummyPlayerController;
use crate::input::player_controller::PlayerController;
use crate::npc::NPC;
use crate::rng::RNG;
use crate::shared_game_state::SharedGameState;
use crate::npc::list::NPCList;

mod player_hit;

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
#[repr(u8)]
pub enum ControlMode {
    Normal = 0,
    IronHead,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PlayerAppearance {
    Quote = 0,
    /// Cave Story+ player skins
    YellowQuote,
    HumanQuote,
    HalloweenQuote,
    ReindeerQuote,
    Curly,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum TargetPlayer {
    Player1,
    Player2,
}

impl TargetPlayer {
    #[inline]
    pub fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone)]
pub struct Player {
    pub x: i32,
    pub y: i32,
    pub vel_x: i32,
    pub vel_y: i32,
    pub target_x: i32,
    pub target_y: i32,
    pub prev_x: i32,
    pub prev_y: i32,
    pub life: u16,
    pub max_life: u16,
    pub cond: Condition,
    pub flags: Flag,
    pub equip: Equipment,
    pub direction: Direction,
    pub display_bounds: Rect<usize>,
    pub hit_bounds: Rect<usize>,
    pub control_mode: ControlMode,
    pub question: bool,
    pub booster_fuel: u32,
    pub up: bool,
    pub down: bool,
    pub shock_counter: u8,
    pub current_weapon: u8,
    pub stars: u8,
    pub damage: u16,
    pub air_counter: u16,
    pub air: u16,
    pub appearance: PlayerAppearance,
    pub controller: Box<dyn PlayerController>,
    weapon_offset_y: i8,
    index_x: i32,
    index_y: i32,
    splash: bool,
    booster_switch: u8,
    bubble: u8,
    damage_counter: u16,
    damage_taken: i16,
    anim_num: u16,
    anim_counter: u16,
    anim_rect: Rect<u16>,
    weapon_rect: Rect<u16>,
}

impl Player {
    pub fn new(state: &mut SharedGameState) -> Player {
        let constants = &state.constants;

        Player {
            x: 0,
            y: 0,
            vel_x: 0,
            vel_y: 0,
            target_x: 0,
            target_y: 0,
            prev_x: 0,
            prev_y: 0,
            life: constants.my_char.life,
            max_life: constants.my_char.max_life,
            cond: Condition(0),
            flags: Flag(0),
            equip: Equipment(0),
            direction: Direction::Right,
            display_bounds: constants.my_char.display_bounds,
            hit_bounds: constants.my_char.hit_bounds,
            control_mode: constants.my_char.control_mode,
            question: false,
            booster_fuel: 0,
            index_x: 0,
            index_y: 0,
            splash: false,
            up: false,
            down: false,
            current_weapon: 0,
            weapon_offset_y: 0,
            shock_counter: 0,
            booster_switch: 0,
            stars: 0,
            damage: 0,
            air_counter: 0,
            air: 0,
            appearance: PlayerAppearance::Quote,
            controller: Box::new(DummyPlayerController::new()),
            bubble: 0,
            damage_counter: 0,
            damage_taken: 0,
            anim_num: 0,
            anim_counter: 0,
            anim_rect: constants.my_char.animations_right[0],
            weapon_rect: Rect::new(0, 0, 0, 0),
        }
    }

    pub fn get_texture_offset(&self) -> u16 {
        (self.appearance as u16 % 6) * 64 + if self.equip.has_mimiga_mask() { 32 } else { 0 }
    }

    fn tick_normal(&mut self, state: &mut SharedGameState, npc_list: &NPCList) -> GameResult {
        if !state.control_flags.interactions_disabled() && state.control_flags.control_enabled() {
            if self.equip.has_air_tank() {
                self.air = 1000;
                self.air_counter = 0;
            } else if !state.settings.god_mode && self.flags.in_water() {
                self.air_counter = 60;
                if self.air > 0 {
                    self.air -= 1;
                } else if let Some(true) = state.game_flags.get(4000) {
                    state.textscript_vm.start_script(1100);
                } else {
                    self.cond.set_hidden(true);
                    state.create_caret(self.x, self.y, CaretType::DrownedQuote, self.direction);
                    state.textscript_vm.start_script(41);
                }
            } else {
                self.air = 1000;

                if self.air_counter > 0 {
                    self.air_counter -= 1;
                }
            }
        }

        if self.cond.hidden() {
            return Ok(());
        }

        let physics = if self.flags.in_water() {
            state.constants.my_char.water_physics
        } else {
            state.constants.my_char.air_physics
        };

        self.question = false;

        if !state.control_flags.control_enabled() {
            self.booster_switch = 0;
        }

        // ground movement
        if self.flags.hit_bottom_wall() || self.flags.hit_right_slope() || self.flags.hit_left_slope() {
            self.booster_switch = 0;

            if state.settings.infinite_booster {
                self.booster_fuel = u32::MAX;
            } else if self.equip.has_booster_0_8() || self.equip.has_booster_2_0() {
                self.booster_fuel = state.constants.booster.fuel;
            } else {
                self.booster_fuel = 0;
            }

            if state.control_flags.control_enabled() {
                let trigger_only_down = self.controller.trigger_down()
                    && !self.controller.trigger_up()
                    && !self.controller.trigger_left()
                    && !self.controller.trigger_right();

                let only_down = self.controller.move_down()
                    && !self.controller.move_up()
                    && !self.controller.move_left()
                    && !self.controller.move_right();

                if trigger_only_down && only_down && !self.cond.interacted() && !state.control_flags.interactions_disabled() {
                    self.cond.set_interacted(true);
                    self.question = true;
                } else {
                    if self.controller.move_left() && self.vel_x > -physics.max_dash {
                        self.vel_x -= physics.dash_ground;
                    }

                    if self.controller.move_right() && self.vel_x < physics.max_dash {
                        self.vel_x += physics.dash_ground;
                    }

                    if self.controller.move_left() {
                        self.direction = Direction::Left;
                    }

                    if self.controller.move_right() {
                        self.direction = Direction::Right;
                    }
                }
            }

            if !self.cond.increase_acceleration() {
                if self.vel_x < 0 {
                    if self.vel_x > -physics.resist {
                        self.vel_x = 0;
                    } else {
                        self.vel_x += physics.resist;
                    }
                }
                if self.vel_x > 0 {
                    if self.vel_x < physics.resist {
                        self.vel_x = 0;
                    } else {
                        self.vel_x -= physics.resist;
                    }
                }
            }
        } else { // air movement
            if state.control_flags.control_enabled() {
                if self.controller.trigger_jump() && self.booster_fuel != 0 {
                    if self.equip.has_booster_0_8() {
                        self.booster_switch = 1;

                        if self.vel_y > 0x100 { // 0.5fix9
                            self.vel_y /= 2;
                        }
                    } else if state.settings.infinite_booster || self.equip.has_booster_2_0() {
                        if self.controller.move_up() {
                            self.booster_switch = 2;
                            self.vel_x = 0;
                            self.vel_y = state.constants.booster.b2_0_up;
                        } else if self.controller.move_left() {
                            self.booster_switch = 1;
                            self.vel_x = state.constants.booster.b2_0_left;
                            self.vel_y = 0;
                        } else if self.controller.move_right() {
                            self.booster_switch = 1;
                            self.vel_x = state.constants.booster.b2_0_right;
                            self.vel_y = 0;
                        } else if self.controller.move_down() {
                            self.booster_switch = 3;
                            self.vel_x = 0;
                            self.vel_y = state.constants.booster.b2_0_down;
                        } else {
                            self.booster_switch = 2;
                            self.vel_x = 0;
                            self.vel_y = state.constants.booster.b2_0_up_nokey;
                        }
                    }
                }

                if self.controller.move_left() && self.vel_x > -physics.max_dash {
                    self.vel_x -= physics.dash_air;
                }

                if self.controller.move_right() && self.vel_x < physics.max_dash {
                    self.vel_x += physics.dash_air;
                }

                if self.controller.look_left() {
                    self.direction = Direction::Left;
                }

                if self.controller.look_right() {
                    self.direction = Direction::Right;
                }
            }

            if (state.settings.infinite_booster || self.equip.has_booster_2_0()) && self.booster_switch != 0
                && (!self.controller.jump() || self.booster_fuel == 0) {
                match self.booster_switch {
                    1 => { self.vel_x /= 2 }
                    2 => { self.vel_y /= 2 }
                    _ => {}
                }
            }

            if self.booster_fuel == 0 || !self.controller.jump() {
                self.booster_switch = 0;
            }
        }

        // jumping
        if state.control_flags.control_enabled() {
            self.up = self.controller.move_up();
            self.down = self.controller.move_down() && !self.flags.hit_bottom_wall();

            if self.controller.trigger_jump() && (self.flags.hit_bottom_wall()
                || self.flags.hit_right_slope()
                || self.flags.hit_left_slope())
                && !self.flags.force_up() {
                self.vel_y = -physics.jump;
                state.sound_manager.play_sfx(15);
            }
        }

        // stop interacting when moved
        if state.control_flags.control_enabled() && (self.controller.move_left()
            || self.controller.move_right()
            || self.controller.move_up()
            || self.controller.jump()
            || self.controller.shoot()) {
            self.cond.set_interacted(false);
        }

        // booster losing fuel
        if self.booster_switch != 0 && self.booster_fuel != 0 {
            self.booster_fuel -= 1;
        }

        // wind / current forces

        if self.flags.force_left() {
            self.vel_x -= 0x88;
        }
        if self.flags.force_up() {
            self.vel_y -= 0x80;
        }
        if self.flags.force_right() {
            self.vel_x += 0x88;
        }
        if self.flags.force_down() {
            self.vel_y += 0x55;
        }

        if (state.settings.infinite_booster || self.equip.has_booster_2_0()) && self.booster_switch != 0 {
            match self.booster_switch {
                1 => {
                    if self.flags.hit_left_wall() || self.flags.hit_right_wall() {
                        self.vel_y = -0x100; // -0.5fix9
                    }

                    if self.direction == Direction::Left {
                        self.vel_x -= 0x20; // 0.1fix9
                    }
                    if self.direction == Direction::Right {
                        self.vel_x += 0x20; // 0.1fix9
                    }

                    if self.controller.trigger_jump() || self.booster_fuel % 3 == 1 {
                        if self.direction == Direction::Left || self.direction == Direction::Right {
                            state.create_caret(self.x + 0x400, self.y + 0x400, CaretType::Exhaust, self.direction.opposite());
                        }
                        state.sound_manager.play_sfx(113);
                    }
                }
                2 => {
                    self.vel_y -= 0x20;

                    if self.controller.trigger_jump() || self.booster_fuel % 3 == 1 {
                        state.create_caret(self.x, self.y + 6 * 0x200, CaretType::Exhaust, Direction::Bottom);
                        state.sound_manager.play_sfx(113);
                    }
                }
                3 if self.controller.trigger_jump() || self.booster_fuel % 3 == 1 => {
                    state.create_caret(self.x, self.y + 6 * 0x200, CaretType::Exhaust, Direction::Up);
                    state.sound_manager.play_sfx(113);
                }
                _ => {}
            }
        } else if self.flags.force_up() {
            self.vel_y += physics.gravity_air;
        } else if self.equip.has_booster_0_8() && self.booster_switch != 0 && self.vel_y > -0x400 {
            self.vel_y -= 0x20;

            if self.booster_fuel % 3 == 0 {
                state.create_caret(self.x, self.y + self.hit_bounds.bottom as i32 / 2, CaretType::Exhaust, Direction::Bottom);
                state.sound_manager.play_sfx(113);
            }

            // bounce off of ceiling
            if self.flags.hit_top_wall() {
                self.vel_y = 0x200; // 1.0fix9
            }
        } else if self.vel_y < 0 && state.control_flags.control_enabled() && self.controller.jump() {
            self.vel_y += physics.gravity_air;
        } else {
            self.vel_y += physics.gravity_ground;
        }

        if !state.control_flags.control_enabled() || !self.controller.trigger_jump() {
            if self.flags.hit_right_slope() && self.vel_x < 0 {
                self.vel_y = -self.vel_x;
            }

            if self.flags.hit_left_slope() && self.vel_x > 0 {
                self.vel_y = self.vel_x;
            }

            if (self.flags.hit_bottom_wall() && self.flags.hit_right_bigger_half() && self.vel_x < 0)
                || (self.flags.hit_bottom_wall() && self.flags.hit_left_bigger_half() && self.vel_x > 0)
                || (self.flags.hit_bottom_wall() && self.flags.hit_left_smaller_half() && self.flags.hit_right_smaller_half()) {
                self.vel_y = 0x400; // 2.0fix9
            }
        }

        let max_move = if self.flags.in_water() && !(self.flags.force_left() || self.flags.force_up() || self.flags.force_right() || self.flags.force_down()) {
            state.constants.my_char.water_physics.max_move
        } else {
            state.constants.my_char.air_physics.max_move
        };

        self.vel_x = clamp(self.vel_x, -max_move, max_move);
        self.vel_y = clamp(self.vel_y, -max_move, max_move);

        if !self.splash && self.flags.in_water() {
            let vertical_splash = !self.flags.hit_bottom_wall() && self.vel_y > 0x200;
            let horizontal_splash = self.vel_x > 0x200 || self.vel_x < -0x200;

            if vertical_splash || horizontal_splash {
                let mut droplet = NPC::create(73, &state.npc_table);
                droplet.cond.set_alive(true);
                droplet.y = self.y;
                droplet.direction = if self.flags.water_splash_facing_right() { Direction::Right } else { Direction::Left };

                for _ in 0..7 {
                    droplet.x = self.x + (state.game_rng.range(-8..8) * 0x200) as i32;
                    droplet.vel_x = if vertical_splash {
                        (self.vel_x + state.game_rng.range(-0x200..0x200) as i32) - (self.vel_x / 2)
                    } else if horizontal_splash {
                        self.vel_x + state.game_rng.range(-0x200..0x200) as i32
                    } else {
                        0 as i32
                    };
                    droplet.vel_y = state.game_rng.range(-0x200..0x80) as i32;

                    let _ = npc_list.spawn(0x100, droplet.clone());
                }

                state.sound_manager.play_sfx(56);
            }

            self.splash = true;
        }

        if !self.flags.in_water() {
            self.splash = false;
        }

        // spike damage
        if self.flags.hit_by_spike() {
            self.damage(10, state, npc_list);
        }

        // camera
        self.index_x = clamp(self.index_x + self.direction.vector_x() * 0x200, -0x8000, 0x8000);

        if state.control_flags.control_enabled() && self.controller.look_up() {
            self.index_y -= 0x200; // 1.0fix9
            if self.index_y < -0x8000 { // -64.0fix9
                self.index_y = -0x8000;
            }
        } else if state.control_flags.control_enabled() && self.controller.look_down() {
            self.index_y += 0x200; // 1.0fix9
            if self.index_y > 0x8000 { // -64.0fix9
                self.index_y = 0x8000;
            }
        } else {
            if self.index_y > 0x200 { // 1.0fix9
                self.index_y -= 0x200;
            }

            if self.index_y < -0x200 { // 1.0fix9
                self.index_y += 0x200;
            }
        }

        self.target_x = self.x + self.index_x;
        self.target_y = self.y + self.index_y;

        if self.vel_x > physics.resist || self.vel_x < -physics.resist {
            self.x += self.vel_x;
        }

        self.y += self.vel_y;

        Ok(())
    }

    fn tick_ironhead(&mut self, _state: &mut SharedGameState) -> GameResult {
        // todo ironhead boss controls
        Ok(())
    }

    fn tick_noclip(&mut self) -> GameResult {
        let speed = if self.controller.jump() {
            4000
        } else {
            2000
        };

        if self.controller.move_down() {
            self.y += speed;
        }
        if self.controller.move_left() {
            self.x -= speed;
        }
        if self.controller.move_right() {
            self.x += speed;
        }
        if self.controller.move_up() {
            self.y -= speed;
        }

        if self.controller.look_left() {
            self.direction = Direction::Left;
            self.anim_num = 0;
        }
        if self.controller.look_right() {
            self.direction = Direction::Right;
            self.anim_num = 0;
        }

        self.target_x = self.x;
        self.target_y = self.y;
        self.vel_x = 0;
        self.vel_y = 0;
        Ok(())
    }

    fn tick_animation(&mut self, state: &mut SharedGameState) {
        if self.cond.hidden() {
            return;
        }

        if self.flags.hit_bottom_wall() {
            if self.cond.interacted() {
                self.anim_num = 11;
            } else if state.control_flags.control_enabled() && self.controller.move_up() && (self.controller.move_left() || self.controller.move_right()) {
                self.cond.set_fallen(true);

                self.anim_counter += 1;
                if self.anim_counter > 4 {
                    self.anim_counter = 0;

                    self.anim_num += 1;
                    if self.anim_num == 7 || self.anim_num == 9 {
                        state.sound_manager.play_sfx(24);
                    }
                }

                if self.anim_num > 9 || self.anim_num < 6 {
                    self.anim_num = 6;
                }
            } else if state.control_flags.control_enabled() && (self.controller.move_left() || self.controller.move_right()) {
                self.cond.set_fallen(true);

                self.anim_counter += 1;
                if self.anim_counter > 4 {
                    self.anim_counter = 0;

                    self.anim_num += 1;
                    if self.anim_num == 2 || self.anim_num == 4 {
                        state.sound_manager.play_sfx(24);
                    }
                }

                if self.anim_num > 4 || self.anim_num < 1 {
                    self.anim_num = 1;
                }
            } else if state.control_flags.control_enabled() && self.controller.move_up() {
                if self.cond.fallen() {
                    state.sound_manager.play_sfx(24);
                }

                self.cond.set_fallen(false);
                self.anim_num = 5;
            } else {
                if self.cond.fallen() {
                    state.sound_manager.play_sfx(24);
                }

                self.cond.set_fallen(false);
                self.anim_num = 0;
            }
        } else if self.controller.look_up() {
            self.anim_num = 6;
        } else if self.controller.look_down() {
            self.anim_num = 10;
        } else {
            self.anim_num = if self.vel_y > 0 { 1 } else { 3 };
        }

        self.weapon_offset_y = 0;
        self.weapon_rect.left = (self.current_weapon as u16 % 13) * 24;
        self.weapon_rect.top = (self.current_weapon as u16 / 13) * 96;
        self.weapon_rect.right = self.weapon_rect.left + 24;
        self.weapon_rect.bottom = self.weapon_rect.top + 16;

        match self.direction {
            Direction::Left => {
                self.anim_rect = state.constants.my_char.animations_left[self.anim_num as usize];
            }
            Direction::Right => {
                self.weapon_rect.top += 16;
                self.weapon_rect.bottom += 16;
                self.anim_rect = state.constants.my_char.animations_right[self.anim_num as usize];
            }
            _ => {}
        }

        if self.up {
            self.weapon_offset_y = -4;
            self.weapon_rect.top += 32;
            self.weapon_rect.bottom += 32;
        } else if self.down {
            self.weapon_offset_y = 4;
            self.weapon_rect.top += 64;
            self.weapon_rect.bottom += 64;
        }

        if self.anim_num == 1 || self.anim_num == 3 || self.anim_num == 6 || self.anim_num == 8 {
            self.weapon_rect.top += 1;
        }

        let offset = self.get_texture_offset();
        self.anim_rect.top += offset;
        self.anim_rect.bottom += offset;
    }

    pub fn damage(&mut self, hp: i32, state: &mut SharedGameState, npc_list: &NPCList) {
        if state.settings.god_mode || self.shock_counter > 0 {
            return;
        }

        state.sound_manager.play_sfx(16);
        self.shock_counter = 128;
        self.cond.set_interacted(false);

        if self.control_mode == ControlMode::Normal {
            self.vel_y = -0x400; // -2.0fix9
        }

        self.life = self.life.saturating_sub(hp as u16);

        if self.equip.has_whimsical_star() && self.stars > 0 {
            self.stars -= 1;
        }

        self.damage = self.damage.saturating_add(hp as u16);

        if self.life == 0 {
            state.sound_manager.play_sfx(17);
            self.cond.0 = 0;
            state.control_flags.set_tick_world(true);
            state.control_flags.set_interactions_disabled(true);
            state.textscript_vm.start_script(40);

            state.create_caret(self.x, self.y, CaretType::Explosion, Direction::Left);
            let mut npc = NPC::create(4, &state.npc_table);
            npc.cond.set_alive(true);
            for _ in 0..0x40 {
                npc.x = self.x + state.game_rng.range(-10..10) as i32 * 0x200;
                npc.y = self.y + state.game_rng.range(-10..10) as i32 * 0x200;

                let _ = npc_list.spawn(0x100, npc.clone());
            }
        }
    }
}

impl GameEntity<&NPCList> for Player {
    fn tick(&mut self, state: &mut SharedGameState, npc_list: &NPCList) -> GameResult {
        if !self.cond.alive() {
            return Ok(());
        }

        if self.damage_counter != 0 {
            self.damage_counter -= 1;
        }

        if self.shock_counter != 0 {
            self.shock_counter -= 1;
        } else if self.damage_taken != 0 {
            // todo: damage popup
            self.damage_taken = 0;
        }

        if state.settings.noclip {
            self.tick_noclip()?;
        } else {
            match self.control_mode {
                ControlMode::Normal => self.tick_normal(state, npc_list)?,
                ControlMode::IronHead => self.tick_ironhead(state)?,
            }
        }

        self.cond.set_increase_acceleration(false);
        self.tick_animation(state);

        Ok(())
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context, frame: &Frame) -> GameResult<> {
        if !self.cond.alive() || self.cond.hidden() || (self.shock_counter / 2 % 2 != 0) {
            return Ok(());
        }

        {
            let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "MyChar")?;
            batch.add_rect(
                interpolate_fix9_scale(self.prev_x - self.display_bounds.left as i32 - frame.prev_x,
                                       self.x - self.display_bounds.left as i32 - frame.x,
                                       state.frame_time),
                interpolate_fix9_scale(self.prev_y - self.display_bounds.left as i32 - frame.prev_y,
                                       self.y - self.display_bounds.left as i32 - frame.y,
                                       state.frame_time),
                &self.anim_rect,
            );
            batch.draw(ctx)?;
        }

        if self.current_weapon != 0 {
            let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "Arms")?;
            match self.direction {
                Direction::Left => {
                    batch.add_rect(
                        interpolate_fix9_scale(self.prev_x - self.display_bounds.left as i32 - frame.prev_x,
                                               self.x - self.display_bounds.left as i32 - frame.x,
                                               state.frame_time) - 8.0,
                        interpolate_fix9_scale(self.prev_y - self.display_bounds.left as i32 - frame.prev_y,
                                               self.y - self.display_bounds.left as i32 - frame.y,
                                               state.frame_time) + self.weapon_offset_y as f32,
                        &self.weapon_rect,
                    );
                }
                Direction::Right => {
                    batch.add_rect(
                        interpolate_fix9_scale(self.prev_x - self.display_bounds.left as i32 - frame.prev_x,
                                               self.x - self.display_bounds.left as i32 - frame.x,
                                               state.frame_time),
                        interpolate_fix9_scale(self.prev_y - self.display_bounds.left as i32 - frame.prev_y,
                                               self.y - self.display_bounds.left as i32 - frame.y,
                                               state.frame_time) + self.weapon_offset_y as f32,
                        &self.weapon_rect,
                    );
                }
                _ => {}
            }

            batch.draw(ctx)?;
        }

        Ok(())
    }
}
