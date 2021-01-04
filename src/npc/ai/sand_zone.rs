use ggez::GameResult;
use num_traits::{abs, clamp, clamp_max};

use crate::bullet::BulletManager;
use crate::caret::CaretType;
use crate::common::Direction;
use crate::npc::list::NPCList;
use crate::npc::NPC;
use crate::player::Player;
use crate::rng::RNG;
use crate::shared_game_state::SharedGameState;

impl NPC {
    pub(crate) fn tick_n044_polish(&mut self, state: &mut SharedGameState, npc_list: &NPCList) -> GameResult {
        match self.action_num {
            0 | 1 => {
                self.anim_num = 0;
                self.action_num = match self.direction {
                    Direction::Left => 8,
                    Direction::Right => 2,
                    _ => 8,
                };
            }
            2 => {
                self.vel_y += 0x20;
                if self.vel_y > 0 && self.flags.hit_bottom_wall() {
                    self.vel_y = -0x100;
                    self.vel_x += 0x100;
                }

                if self.flags.hit_right_wall() {
                    self.action_num = 3;
                }
            }
            3 => {
                self.vel_x += 0x20;
                if self.vel_x > 0 && self.flags.hit_right_wall() {
                    self.vel_x = -0x100;
                    self.vel_y -= 0x100;
                }

                if self.flags.hit_top_wall() {
                    self.action_num = 4;
                }
            }
            4 => {
                self.vel_y -= 0x20;
                if self.vel_y < 0 && self.flags.hit_top_wall() {
                    self.vel_y = 0x100;
                    self.vel_x -= 0x100;
                }

                if self.flags.hit_left_wall() {
                    self.action_num = 5;
                }
            }
            5 => {
                self.vel_x -= 0x20;
                if self.vel_x < 0 && self.flags.hit_left_wall() {
                    self.vel_x = 0x100;
                    self.vel_y += 0x100;
                }

                if self.flags.hit_bottom_wall() {
                    self.action_num = 2;
                }
            }
            6 => {
                self.vel_y += 0x20;
                if self.vel_y > 0 && self.flags.hit_bottom_wall() {
                    self.vel_y = -0x100;
                    self.vel_x -= 0x100;
                }

                if self.flags.hit_left_wall() {
                    self.action_num = 7;
                }
            }
            7 => {
                self.vel_x -= 0x20;
                if self.vel_x < 0 && self.flags.hit_left_wall() {
                    self.vel_x = 0x100;
                    self.vel_y -= 0x100;
                }

                if self.flags.hit_top_wall() {
                    self.action_num = 8;
                }
            }
            8 => {
                self.vel_y -= 0x20;
                if self.vel_y < 0 && self.flags.hit_top_wall() {
                    self.vel_y = 0x100;
                    self.vel_x += 0x100;
                }

                if self.flags.hit_right_wall() {
                    self.action_num = 9;
                }
            }
            9 => {
                self.vel_x += 0x20;
                if self.vel_x > 0 && self.flags.hit_right_wall() {
                    self.vel_x = -0x100;
                    self.vel_y += 0x100;
                }

                if self.flags.hit_bottom_wall() {
                    self.action_num = 6;
                }
            }
            _ => {}
        }

        if self.life <= 100 {
            npc_list.create_death_smoke(self.x, self.y, self.display_bounds.right, 8, state, &self.rng);
            state.sound_manager.play_sfx(25);
            self.cond.set_alive(false);

            let mut npc = NPC::create(45, &state.npc_table);
            npc.cond.set_alive(true);
            npc.x = self.x;
            npc.y = self.y;
            for _ in 0..9 {
                let _ = npc_list.spawn(0x100, npc.clone());
            }
        }

        self.vel_x = clamp(self.vel_x, -0x200, 0x200);
        self.vel_y = clamp(self.vel_y, -0x200, 0x200);

        if self.shock > 0 {
            self.x += self.vel_x / 2;
            self.y += self.vel_y / 2;
        } else {
            self.x += self.vel_x;
            self.y += self.vel_y;
        }

        if self.action_num > 1 && self.action_num <= 9 {
            self.anim_num += 1;
            if self.anim_num > 2 {
                self.anim_num = 1;
            }
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };

        self.anim_rect = state.constants.npc.n044_polish[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n045_baby(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 2;
            self.vel_x = if self.rng.next_u16() & 1 != 0 {
                self.rng.range(-0x200..-0x100) as i32
            } else {
                self.rng.range(0x100..0x200) as i32
            };
            self.vel_y = if self.rng.next_u16() & 1 != 0 {
                self.rng.range(-0x200..-0x100) as i32
            } else {
                self.rng.range(0x100..0x200) as i32
            };
            self.vel_x2 = self.vel_x;
            self.vel_y2 = self.vel_y;
        }

        match self.action_num {
            1 | 2 => {
                self.anim_num += 1;
                if self.anim_num > 2 {
                    self.anim_num = 1;
                }
            }
            _ => {}
        }

        if self.vel_x2 < 0 && self.flags.hit_left_wall() {
            self.vel_x2 = -self.vel_x2;
        }

        if self.vel_x2 > 0 && self.flags.hit_right_wall() {
            self.vel_x2 = -self.vel_x2;
        }

        if self.vel_y2 < 0 && self.flags.hit_top_wall() {
            self.vel_y2 = -self.vel_y2;
        }

        if self.vel_y2 > 0 && self.flags.hit_bottom_wall() {
            self.vel_y2 = -self.vel_y2;
        }

        self.vel_x2 = clamp(self.vel_x2, -0x200, 0x200);
        self.vel_y2 = clamp(self.vel_y2, -0x200, 0x200);

        if self.shock > 0 {
            self.x += self.vel_x2 / 2;
            self.y += self.vel_y2 / 2;
        } else {
            self.x += self.vel_x2;
            self.y += self.vel_y2;
        }

        self.anim_rect = state.constants.npc.n045_baby[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n047_sandcroc(&mut self, state: &mut SharedGameState, players: [&mut Player; 2]) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.action_counter = 0;
                    self.anim_num = 0;
                    self.target_y = self.y;
                    self.npc_flags.set_shootable(false);
                    self.npc_flags.set_ignore_solidity(false);
                    self.npc_flags.set_invulnerable(false);
                    self.npc_flags.set_solid_soft(false);
                }

                let player = self.get_closest_player_mut(players);
                if abs(self.x - player.x) < 8 * 0x200 && player.y > self.y && player.y < self.y + 8 * 0x200 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    state.sound_manager.play_sfx(102);
                }

                self.x += (player.x - self.x).signum() * 2 * 0x200;
            }
            2 => {
                self.anim_counter += 1;
                if self.anim_counter > 3 {
                    self.anim_num += 1;
                    self.anim_counter = 0;
                }

                match self.anim_num {
                    3 => self.damage = 10,
                    4 => {
                        self.action_num = 3;
                        self.action_counter = 0;
                        self.npc_flags.set_shootable(true);
                    }
                    _ => {}
                }
            }
            3 => {
                self.damage = 0;
                self.npc_flags.set_solid_soft(true);

                self.action_counter += 1;
                if self.shock > 0 {
                    self.action_num = 4;
                    self.action_counter = 0;
                }
            }
            4 => {
                self.npc_flags.set_ignore_solidity(true);
                self.y += 0x200;
                self.action_counter += 1;
                if self.action_counter == 32 {
                    self.action_num = 5;
                    self.action_counter = 0;
                    self.npc_flags.set_solid_soft(false);
                    self.npc_flags.set_shootable(false);
                }
            }
            5 => {
                if self.action_counter > 99 {
                    self.y = self.target_y;
                    self.action_num = 0;
                    self.anim_num = 0;
                } else {
                    self.action_counter += 1;
                }
            }
            _ => {}
        }

        self.anim_rect = state.constants.npc.n047_sandcroc[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n049_skullhead(&mut self, state: &mut SharedGameState, npc_list: &NPCList) -> GameResult {
        let parent = self.get_parent_ref_mut(npc_list);

        Ok(())
    }

    pub(crate) fn tick_n118_curly_boss(&mut self, state: &mut SharedGameState, players: [&mut Player; 2], npc_list: &NPCList, bullet_manager: &BulletManager) -> GameResult {
        let i = self.get_closest_player_idx_mut(&players);

        let looking_up = (self.direction == Direction::Left && self.x < players[i].x) ||
                         (self.direction == Direction::Right && self.x > players[i].x);

        match self.action_num {
            0 => {
                self.action_num = 1;
                self.anim_num = 0;
                self.anim_counter = 0;
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.action_counter = self.rng.range(50..100) as u16;
                    self.anim_num = 0;

                    self.direction = if self.x > players[i].x {
                        Direction::Left
                    } else {
                        Direction::Right
                    };

                    self.npc_flags.set_shootable(true);
                    self.npc_flags.set_invulnerable(false);
                }

                if self.action_counter != 0 {
                    self.action_counter -= 1;
                } else {
                    self.action_num = 13;
                }
            }
            13 | 14 => {
                if self.action_num == 13 {
                    self.action_num = 14;
                    self.action_counter = self.rng.range(50..100) as u16;
                    self.anim_num = 3;

                    self.direction = if self.x > players[i].x {
                        Direction::Left
                    } else {
                        Direction::Right
                    };
                }

                self.anim_counter += 1;
                if self.anim_counter > 2 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                }

                if self.anim_num > 6 {
                    self.anim_num = 3;
                }

                self.vel_x += if self.direction == Direction::Left {
                    -0x40
                } else {
                    0x40
                };

                if self.action_counter != 0 {
                    self.action_counter -= 1;
                } else {
                    self.npc_flags.set_shootable(true);
                    self.action_num = 20;
                    self.action_counter = 0;
                    state.sound_manager.play_sfx(103);
                }
            }
            20 => {
                self.direction = if self.x > players[i].x {
                    Direction::Left
                } else {
                    Direction::Right
                };

                self.vel_x = (self.vel_x * 8) / 9;

                self.anim_num += 1;
                if self.anim_num > 1 {
                    self.anim_num = 0;
                }

                self.action_counter += 1;
                if self.action_counter > 50 {
                    self.action_num = 21;
                    self.action_counter = 0;
                }
            }
            21 => {
                self.action_counter += 1;
                if self.action_counter % 4 == 1 {
                    let mut npc = NPC::create(123, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.x;
                    npc.y = self.y;
                    if looking_up {
                        self.anim_num = 2;
                        npc.y -= 8 * 0x200;
                        npc.direction = Direction::Up;
                    } else {
                        self.anim_num = 0;
                        npc.x += if self.direction == Direction::Left {
                            -8 * 0x200
                        } else {
                            8 * 0x200
                        };
                        npc.y += 4 * 0x200;
                        npc.direction = self.direction;
                    }
                    let _ = npc_list.spawn(0x100, npc);
                }
                if self.action_counter > 30 {
                    self.action_num = 10;
                }
            }
            30 => {
                self.anim_num += 1;
                if self.anim_num > 8 {
                    self.anim_num = 7;
                }

                self.action_counter += 1;
                if self.action_counter > 30 {
                    self.action_num = 10;
                    self.anim_num = 0;
                }
            }
            _ => {}
        }

        if self.action_num > 10 && self.action_num < 30 && bullet_manager.count_bullets_type_idx_all(6) > 0 {
            self.action_num = 30;
            self.action_counter = 0;
            self.anim_num = 7;
            self.npc_flags.set_shootable(false);
            self.npc_flags.set_invulnerable(true);
            self.vel_x = 0;
        }

        self.vel_x = clamp(self.vel_x, -0x1ff, 0x1ff);
        self.vel_y = clamp_max(self.vel_y + 0x20, 0x5ff);

        self.x += self.vel_x;
        self.y += self.vel_y;

        self.anim_rect = if self.direction == Direction::Left {
            state.constants.npc.n118_curly_boss[self.anim_num as usize]
        } else {
            state.constants.npc.n118_curly_boss[self.anim_num as usize + 9]
        };

        Ok(())
    }

    pub(crate) fn tick_n120_colon_a(&mut self, state: &SharedGameState) -> GameResult {
        self.anim_rect = match self.direction {
            Direction::Left => state.constants.npc.n120_colon_a[0],
            _ => state.constants.npc.n120_colon_a[1],
        };

        Ok(())
    }

    pub(crate) fn tick_n121_colon_b(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.direction == Direction::Left {
            match self.action_num {
                0 | 1 => {
                    if self.action_num == 0 {
                        self.action_num = 1;
                        self.anim_num = 0;
                        self.anim_counter = 0;
                    }

                    if self.rng.range(0..120) == 10 {
                        self.action_num = 2;
                        self.action_counter = 0;
                        self.anim_num = 1;
                    }
                }
                2 => {
                    self.action_counter += 1;
                    if self.action_counter > 8 {
                        self.action_num = 1;
                        self.anim_num = 0;
                    }
                }
                _ => {}
            }

            self.anim_rect = state.constants.npc.n121_colon_b[self.anim_num as usize];
        } else {
            self.anim_rect = state.constants.npc.n121_colon_b[2];

            self.action_counter += 1;
            if self.action_counter > 100 {
                self.action_counter = 0;
                state.create_caret(self.x, self.y, CaretType::Zzz, Direction::Left);
            }
        }

        Ok(())
    }

    pub(crate) fn tick_n122_colon_enraged(&mut self, state: &SharedGameState, players: [&mut Player; 2]) -> GameResult {
        let i = self.get_closest_player_idx_mut(&players);
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                }

                if self.rng.range(0..120) == 10 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }

                if abs(players[i].x - self.x) < 32 * 0x200 && abs(players[i].y - self.y) < 32 * 0x200 {
                    self.direction = if self.x > players[i].x {
                        Direction::Left
                    } else {
                        Direction::Right
                    };
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 8 {
                    self.action_num = 1;
                    self.anim_num = 0;
                }
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.life = 1000;
                    self.action_num = 11;
                    self.action_counter = self.rng.range(0..50) as u16;
                    self.anim_num = 0;
                    self.damage = 0;
                }

                if self.action_counter != 0 {
                    self.action_counter -= 1;
                } else {
                    self.action_num = 13;
                }
            }
            13 | 14 => {
                if self.action_num == 13 {
                    self.action_num = 14;
                    self.action_counter = self.rng.range(0..50) as u16;

                    self.direction = if self.x > players[i].x {
                        Direction::Left
                    } else {
                        Direction::Right
                    };
                }

                self.anim_counter += 1;
                if self.anim_counter > 2 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                }

                if self.anim_num > 5 {
                    self.anim_num = 2;
                }

                self.vel_x += if self.direction == Direction::Left {
                    -0x40
                } else {
                    0x40
                };

                if self.action_counter > 0 {
                    self.action_counter -= 1;
                } else {
                    self.npc_flags.set_shootable(true);
                    self.action_num = 15;
                    self.anim_num = 2;
                    self.vel_y = -0x200;
                    self.damage = 2;
                }
            }
            15 => {
                if self.flags.hit_bottom_wall() {
                    self.npc_flags.set_shootable(true);
                    self.vel_x = 0;
                    self.action_num = 10;
                    self.damage = 0;
                }
            }
            20 => {
                if self.flags.hit_bottom_wall() {
                    self.vel_x = 0;
                    self.action_num = 21;
                    self.damage = 0;

                    if self.anim_num == 6{
                        self.anim_num = 8;
                    } else {
                        self.anim_num = 9;
                    }

                    self.action_counter = self.rng.range(300..400) as u16;
                }
            }
            21 => {
                if self.action_counter > 0 {
                    self.action_counter -= 1;
                } else {
                    self.npc_flags.set_shootable(true);
                    self.life = 1000;
                    self.action_num = 11;
                    self.action_counter = self.rng.range(0..50) as u16;
                    self.anim_num = 0;
                }
            }
            _ => {}
        }

        if self.action_num > 10 && self.action_num < 20 && self.life != 1000 {
            self.action_num = 20;
            self.vel_y = -0x200;
            self.anim_num = self.rng.range(6..7) as u16;
            self.npc_flags.set_shootable(false);
        }

        self.vel_x = clamp(self.vel_x, -0x1ff, 0x1ff);
        self.vel_y = clamp_max(self.vel_y + 0x20, 0x5ff);

        self.x += self.vel_x;
        self.y += self.vel_y;

        self.anim_rect = if self.direction == Direction::Left {
            state.constants.npc.n122_colon_enraged[self.anim_num as usize]
        } else {
            state.constants.npc.n122_colon_enraged[self.anim_num as usize + 10]
        };

        Ok(())
    }

    pub(crate) fn tick_n123_curly_boss_bullet(&mut self, state: &mut SharedGameState) -> GameResult {
        let mut should_break = false;

        match self.action_num {
            0 => {
                self.action_num = 1;
                state.create_caret(self.x, self.y, CaretType::Shoot, Direction::Left);
                state.sound_manager.play_sfx(32);

                let rand = self.rng.range(-0x80..0x80);
                (self.vel_x, self.vel_y) = match self.direction {
                    Direction::Left => (-0x1000, rand),
                    Direction::Up => (rand, -0x1000),
                    Direction::Right => (0x1000, rand),
                    Direction::Bottom => (rand, 0x1000),
                    Direction::FacingPlayer => unreachable!(),
                }
            }
            1 => {
                should_break = match self.direction {
                    Direction::Left => self.flags.hit_left_wall(),
                    Direction::Up => self.flags.hit_top_wall(),
                    Direction::Right => self.flags.hit_right_wall(),
                    Direction::Bottom => self.flags.hit_bottom_wall(),
                    Direction::FacingPlayer => unreachable!(),
                };

                self.x += self.vel_x;
                self.y += self.vel_y;
            }
            _ => {}
        }

        if should_break {
            state.create_caret(self.x, self.y, CaretType::ProjectileDissipation, Direction::Right);
            state.sound_manager.play_sfx(28);
            self.cond.set_alive(false);
        }

        self.anim_rect = state.constants.npc.n123_curly_boss_bullet[self.direction as usize];

        Ok(())
    }

    pub(crate) fn tick_n124_sunstone(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.x += 8 * 0x200;
                    self.y += 8 * 0x200;
                }

                self.npc_flags.set_ignore_solidity(false);
                self.anim_num = 0;
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.action_counter = 0;
                    self.anim_num = 1;

                    self.npc_flags.set_ignore_solidity(true);
                }

                match self.direction {
                    Direction::Left => self.x -= 0x80,
                    Direction::Up => self.y -= 0x80,
                    Direction::Right => self.x += 0x80,
                    Direction::Bottom => self.y += 0x80,
                    Direction::FacingPlayer => {}
                }

                state.quake_counter = 20;
                if self.action_counter % 8 == 0 {
                    state.sound_manager.play_sfx(26);
                }
            }
            _ => {}
        }

        self.anim_rect = state.constants.npc.n124_sunstone[self.anim_num as usize];

        Ok(())
    }
}
