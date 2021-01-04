use ggez::GameResult;
use num_traits::{abs, clamp, clamp_max};

use crate::common::Direction;
use crate::npc::NPC;
use crate::player::Player;
use crate::rng::RNG;
use crate::shared_game_state::SharedGameState;

impl NPC {
    pub(crate) fn tick_n029_cthulhu(&mut self, state: &mut SharedGameState, players: [&mut Player; 2]) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.anim_num = 0;
            self.anim_counter = 0;
        }

        let player = self.get_closest_player_mut(players);

        if abs(self.x - player.x) < 48 * 0x200 && self.y - 48 * 0x200 < player.y && self.y + 16 * 0x200 > player.y {
            self.anim_num = 1;
        } else {
            self.anim_num = 0;
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 2 };
        self.anim_rect = state.constants.npc.n029_cthulhu[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n052_sitting_blue_robot(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.anim_rect = state.constants.npc.n052_sitting_blue_robot;
        }

        Ok(())
    }

    pub(crate) fn tick_n055_kazuma(&mut self, state: &mut SharedGameState) -> GameResult {
        let off = if self.direction == Direction::Left { 0 } else { 6 };

        match self.action_num {
            0 => {
                self.action_num = 1;
                self.anim_num = 0;
                self.anim_counter = 0;
                self.anim_rect = state.constants.npc.n055_kazuma[off];
            }
            3 | 4 => {
                if self.action_num == 3 {
                    self.action_num = 4;
                    self.anim_num = 1;
                    self.anim_counter = 0;
                }

                self.anim_counter += 1;
                if self.anim_counter > 4 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                }

                if self.anim_num > 4 {
                    self.anim_num = 1;
                }

                match self.direction {
                    Direction::Left => {
                        self.x -= 0x200;
                    }
                    Direction::Right => {
                        self.x += 0x200;
                    }
                    _ => {}
                }

                self.anim_rect = state.constants.npc.n055_kazuma[self.anim_num as usize + off];
            }
            5 => {
                self.anim_num = 5;
                self.anim_rect = state.constants.npc.n055_kazuma[self.anim_num as usize + off];
            }
            _ => {}
        }

        self.vel_y += 0x20;

        if self.vel_y > 0x5ff {
            self.vel_y = 0x5ff;
        }

        Ok(())
    }

    pub(crate) fn tick_n061_king(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                    self.vel_x = 0;
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
            5 => {
                self.anim_num = 3;
                self.vel_x = 0;
            }
            6 | 7 => {
                if self.action_num == 6 {
                    self.action_num = 7;
                    self.action_counter = 0;
                    self.anim_counter = 0;
                    self.vel_y = -0x400;
                }

                self.anim_num = 2;

                if self.direction == Direction::Left {
                    self.vel_x = -0x200;
                } else {
                    self.vel_x = 0x200;
                }


                if self.action_counter != 0 && self.flags.hit_bottom_wall() {
                    self.action_num = 5;
                }

                self.action_counter += 1;
            }
            8 | 9 => {
                if self.action_num == 8 {
                    self.action_num = 9;
                    self.anim_num = 4;
                    self.anim_counter = 0;
                }

                self.anim_counter += 1;
                if self.anim_counter > 4 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                }

                if self.anim_num > 7 {
                    self.anim_num = 4;
                }

                if self.direction == Direction::Left {
                    self.vel_x = -0x200;
                } else {
                    self.vel_x = 0x200;
                }
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.anim_num = 4;
                    self.anim_counter = 0;
                }

                self.anim_counter += 1;
                if self.anim_counter > 2 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                }

                if self.anim_num > 7 {
                    self.anim_num = 4;
                }

                if self.direction == Direction::Left {
                    self.vel_x = -0x400;
                } else {
                    self.vel_x = 0x400;
                }
            }
            // todo: 20 - king's sword
            // todo: 30,31 - pre misery attack
            // todo: 40,42 - dying
            // todo: 60,61 - leap
            _ => {}
        }


        if self.action_num < 30 || self.action_num >= 40 {
            self.vel_y += 0x40;
            self.vel_x = clamp(self.vel_x, -0x400, 0x400);

            if self.vel_y > 0x5ff {
                self.vel_y = 0x5ff;
            }
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        if self.direction == Direction::Left {
            self.anim_rect = state.constants.npc.n061_king[self.anim_num as usize];
        } else {
            self.anim_rect = state.constants.npc.n061_king[self.anim_num as usize + 10];
        }

        Ok(())
    }

    pub(crate) fn tick_n062_kazuma_computer(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.x -= 4 * 0x200;
                    self.y += 16 * 0x200;
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                    self.anim_rect = state.constants.npc.n062_kazuma_computer[self.anim_num as usize];
                }

                self.anim_counter += 1;
                if self.anim_counter > 2 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                    self.anim_rect = state.constants.npc.n062_kazuma_computer[self.anim_num as usize];
                }

                if self.anim_num > 1 {
                    self.anim_num = 0;
                    self.anim_rect = state.constants.npc.n062_kazuma_computer[self.anim_num as usize];
                }

                if self.rng.range(0..80) == 1 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;
                    self.anim_rect = state.constants.npc.n062_kazuma_computer[self.anim_num as usize];
                }

                if self.rng.range(0..120) == 10 {
                    self.action_num = 3;
                    self.action_counter = 0;
                    self.anim_num = 2;
                    self.anim_rect = state.constants.npc.n062_kazuma_computer[self.anim_num as usize];
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 40 {
                    self.action_num = 3;
                    self.anim_num = 2;
                    self.action_counter = 0;
                    self.anim_rect = state.constants.npc.n062_kazuma_computer[self.anim_num as usize];
                }
            }
            3 => {
                self.action_counter += 1;
                if self.action_counter > 80 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_rect = state.constants.npc.n062_kazuma_computer[self.anim_num as usize];
                }
            }
            _ => {}
        }

        Ok(())
    }

    pub(crate) fn tick_n074_jack(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                    self.vel_x = 0;
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
            8 | 9 => {
                if self.anim_num == 8 {
                    self.action_num = 9;
                    self.anim_num = 2;
                    self.anim_counter = 0;
                }

                self.anim_counter += 1;
                if self.anim_counter > 4 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                }

                if self.anim_num > 5 {
                    self.anim_num = 2;
                }

                if self.direction == Direction::Left {
                    self.vel_x = -0x200;
                } else {
                    self.vel_x = 0x200;
                }
            }
            _ => {}
        }

        self.vel_y += 0x40;
        self.vel_x = clamp(self.vel_x, -0x400, 0x400);

        if self.vel_y > 0x5ff {
            self.vel_y = 0x5ff;
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 6 };

        self.anim_rect = state.constants.npc.n074_jack[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n151_blue_robot_standing(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                }

                if self.rng.range(0..100) == 0 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 16 {
                    self.action_num = 1;
                    self.anim_num = 0;
                }
            }
            _ => {}
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 2 };

        self.anim_rect = state.constants.npc.n151_blue_robot_standing[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n117_curly(&mut self, state: &mut SharedGameState, players: [&mut Player; 2]) -> GameResult {
        let i = self.get_closest_player_idx_mut(&players);
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    if self.direction == Direction::FacingPlayer {
                        self.direction = if self.x > players[i].x {
                            Direction::Left
                        } else {
                            Direction::Right
                        }
                    }

                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                }

                self.vel_x = 0;
                self.vel_y += 0x40;
            }
            3 | 4 => {
                if self.action_num == 3 {
                    self.action_num = 4;
                    self.anim_num = 1;
                    self.anim_counter = 0;
                }

                self.anim_counter += 1;
                if self.anim_counter > 4 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                }

                if self.anim_num > 4 {
                    self.anim_num = 1;
                }

                self.vel_y += 0x40;
                self.vel_x = if self.direction == Direction::Left {
                    -0x200
                } else {
                    0x200
                };
            }
            5 => {
                self.action_num = 6;
                self.anim_num = 5;
                self.cond.set_alive(false);
            }
            6 => {
                self.anim_num = 5;
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.anim_num = 1;
                    self.anim_counter = 0;

                    if self.direction == Direction::FacingPlayer {
                        self.direction = if self.x > players[i].x {
                            Direction::Left
                        } else {
                            Direction::Right
                        }
                    }
                }

                self.anim_counter += 1;
                if self.anim_counter > 4 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                }

                if self.anim_num > 4 {
                    self.anim_num = 1;
                }

                self.vel_x += if self.direction == Direction::Left {
                    -0x200
                } else {
                    0x200
                };

                if abs(players[i].x - self.x) < 20 * 0x200 {
                    self.action_num = 0;
                }
            }
            20 | 21 => {
                self.vel_x = 0;
                self.anim_num = if self.action_num == 20 {
                    6
                } else {
                    9
                };
            }
            30 | 31 => {
                if self.action_num == 30 {
                    self.action_num = 31;
                    self.action_counter = 0;
                    self.vel_y = -0x400;
                }

                self.anim_num = 7;

                self.vel_x = if self.direction == Direction::Left {
                    -0x200
                } else {
                    0x200
                };
                self.vel_y += 0x40;

                if self.action_counter != 0 && self.flags.hit_bottom_wall() {
                    self.action_num = 32;
                }
                self.action_counter += 1;
            }
            32 => {
                self.vel_x = 0;
                self.vel_y += 0x40;
                self.anim_num = 8;
            }
            70 | 71 => {
                if self.action_num == 70 {
                    self.action_num = 71;
                    self.action_counter = 0;
                    self.anim_num = 1;
                    self.anim_counter = 0;
                }

                self.vel_x += if self.direction == Direction::Left {
                    0x100
                } else {
                    -0x100
                };

                self.anim_counter += 1;
                if self.anim_counter > 8 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                }

                if self.anim_num > 4 {
                    self.anim_num = 1;
                }
            }
            _ => {}
        }

        if self.vel_y > 0x5ff {
            self.vel_y = 0x5ff;
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        self.anim_rect = if self.direction == Direction::Left {
            state.constants.npc.n117_curly[self.anim_num as usize]
        } else {
            state.constants.npc.n117_curly[self.anim_num as usize + 10]
        };

        Ok(())
    }
}
