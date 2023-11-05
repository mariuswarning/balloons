use crate::ball::BallState::{Expanding, Normal, Shrinking, Vanish};
use crate::ball::{balls_distance_squared, Ball, BallPair, BallPairIds, BallType, ACTIVE_BALL};
use crate::geometry::{Point, Rect, RenderingRect};
use crate::quadtree::QuadTreeNode;
use std::borrow::BorrowMut;
use std::collections::{HashMap, HashSet};
use web_sys::console;

#[derive(Debug, Clone)]
pub struct Level {
    pub max_shots: usize,
    pub num_of_balls: usize,
    pub num_captured: usize,
}

impl Level {
    fn new(max_shots: usize, num_of_balls: usize, num_captured: usize) -> Self {
        Self {
            max_shots,
            num_of_balls,
            num_captured,
        }
    }
}

#[derive(Debug)]
pub struct GameState {
    pub all_levels: Vec<Level>,
    pub captured: usize,
    pub captured_required: usize,
    pub is_paused: bool,
    pub is_render_debug: bool,
    pub level_id: usize,
    pub next_id: usize,
    pub objects: HashMap<usize, Ball>,
    pub rect: Rect,
    pub result: GameResult,
    pub shots: usize,
    pub tree: Option<Box<QuadTreeNode>>,
}

impl GameState {
    pub fn gen_next_id(&mut self) -> usize {
        self.next_id += 1;
        self.next_id
    }

    pub fn insert_object(&mut self, ball: &mut Ball) {
        let new_id = self.gen_next_id();
        ball.id = new_id;
        self.objects.insert(new_id, *ball);
    }

    pub fn new(width: usize, height: usize) -> Self {
        let all_levels: Vec<Level> = vec![
            Level::new(1, 5, 1),
            Level::new(1, 5, 2),
            Level::new(1, 8, 3),
            Level::new(1, 10, 5),
            Level::new(1, 15, 7),
            Level::new(1, 20, 9),
            Level::new(1, 20, 12),
            Level::new(1, 25, 15),
            Level::new(1, 30, 20),
            Level::new(1, 30, 22),
            Level::new(1, 35, 27),
            Level::new(1, 40, 31),
            Level::new(1, 45, 40),
            Level::new(1, 55, 47),
            Level::new(1, 60, 50),
            Level::new(1, 65, 57),
            Level::new(1, 65, 60),
            Level::new(1, 90, 80),
            Level::new(1, 99, 97),
        ];

        let mut new_state = Self {
            rect: Rect {
                x: 0.0,
                y: 0.0,
                w: width as f64,
                h: height as f64,
            },
            objects: HashMap::new(),
            next_id: 1,
            captured: 0,
            captured_required: 0,
            shots: 3,
            result: GameResult::Playing,
            tree: None,
            is_paused: false,
            is_render_debug: false,
            all_levels,
            level_id: 0,
        };

        new_state.next_level();

        let rect = Rect {
            x: 0.0,
            y: 0.0,
            w: width as f64,
            h: height as f64,
        };
        let qt = QuadTreeNode::new(rect, 0, 0);
        new_state.tree = qt;

        new_state
    }

    pub fn next_level(&mut self) {
        let next_level_id: usize = self.level_id + 1;
        let level: Level = self.all_levels[next_level_id].clone();

        self.captured = 0;
        self.captured_required = level.num_captured;
        self.shots = level.max_shots;
        self.result = GameResult::Playing;

        while self.objects.len() < level.num_of_balls {
            self.insert_object(&mut Ball::random_ball(
                0,
                self.rect.w as usize,
                self.rect.h as usize,
                BallType::random_ball_type(),
            ));
        }

        self.level_id = next_level_id;
    }

    pub fn restart(&mut self) {
        let level: Level = self.all_levels[self.level_id].clone();

        self.captured = 0;
        self.captured_required = level.num_captured;
        self.shots = level.max_shots;
        self.result = GameResult::Playing;

        while self.objects.len() < level.num_of_balls {
            self.insert_object(&mut Ball::random_ball(
                0,
                self.rect.w as usize,
                self.rect.h as usize,
                BallType::random_ball_type(),
            ));
        }
    }

    pub fn quit(&self) {}

    pub fn get_stats(&self) -> String {
        format!(
            "CAPTURED: {} / {}, SHOTS LEFT: {}",
            self.captured, self.captured_required, self.shots
        )
    }

    pub fn get_goal(&self) -> String {
        format!(
            "LEVEL {} - CAPTURE {} SPACEBALLS",
            self.level_id, self.captured_required
        )
    }

    pub fn check_win_lose(&self) -> GameResult {
        let active_balls = self
            .objects
            .values()
            .filter(|item| item.ball_state == Expanding || item.ball_state == Shrinking)
            .count();

        if self.shots == 0 && active_balls == 0 {
            if self.captured >= self.captured_required {
                return GameResult::Won;
            }
            return GameResult::Lost;
        }
        GameResult::Playing
    }

    pub fn get_rectangles(&self) -> Vec<RenderingRect> {
        if self.tree.is_none() {
            return vec![];
        }

        self.tree.as_ref().unwrap().get_rectangles()
    }

    pub fn pause_play(&mut self) {
        self.is_paused = !self.is_paused;

        if self.is_paused {
            self.is_render_debug = true;
        }
    }

    fn handle_collisions(&mut self) {
        if self.tree.is_none() {
            return;
        }

        // find collisions candidates

        let mut candidates: HashSet<BallPairIds> = HashSet::new();
        for ball in self.objects.values() {
            let found = self.tree.as_ref().expect("").search(ball.id);
            for f in found {
                candidates.insert(f);
            }
        }

        // check collisions
        for cand in candidates {
            let mut first_ball_state = self
                .objects
                .get_mut(&cand.first)
                .expect("this ball should exist")
                .ball_state;
            let mut second_ball_state = self
                .objects
                .get(&cand.second)
                .expect("this ball should exist")
                .ball_state;
            let ball1 = self
                .objects
                .get(&cand.first)
                .expect("this ball should exist");
            let ball2 = self
                .objects
                .get(&cand.second)
                .expect("this ball should exist");

            let ball_pair = BallPair {
                first: *ball1,
                second: *ball2,
            };
            let calc_dist = balls_distance_squared(*ball1, *ball2);
            let my_dist = ball1.radius * ball1.radius + ball2.radius * ball2.radius;

            if ball_pair.is_collision_bb() {
                if (first_ball_state == Shrinking || first_ball_state == Expanding)
                    && second_ball_state == Normal
                {
                    let mut second_ball = self
                        .objects
                        .get_mut(&cand.second)
                        .expect("this ball should exist");

                    second_ball.next_ball_state = Expanding;
                    second_ball.set_captured();
                }

                if (second_ball_state == Shrinking || second_ball_state == Expanding)
                    && first_ball_state == Normal
                {
                    let mut first_ball = self
                        .objects
                        .get_mut(&cand.first)
                        .expect("this ball should exist");
                    first_ball.next_ball_state = Expanding;
                    first_ball.set_captured();
                }
            }
        }
    }

    fn get_number_of_vanished_balls(&self) -> usize {
        self.all_levels[self.level_id].num_of_balls - self.objects.len()
            + self.all_levels[self.level_id].max_shots
            - self.shots
    }

    pub fn tick(&mut self) {
        let num_objects = &self.objects.len();
        self.objects.retain(|_key, obj| obj.ball_state != Vanish);
        self.captured = self.get_number_of_vanished_balls()
            + self.objects.values().filter(|obj| obj.is_captured).count();

        self.result = self.check_win_lose();
        for (_, obj) in &mut self.objects {
            obj.tick();
        }

        self.update_quadtree();
        self.handle_collisions();

        for (_, obj) in &mut self.objects {
            obj.apply_tick_changes();
        }
    }

    fn update_quadtree(&mut self) {
        let rect = self.rect;
        let mut qt = QuadTreeNode::new(rect, 0, 0);

        for ball in self.objects.clone().values() {
            qt.as_deref_mut().expect("").insert_ball(ball);
        }
        self.tree = qt;
    }

    pub fn show_state(&self) {
        for (_, obj) in &self.objects {
            console::log_1(&format!("{} {}", obj.pos.x, obj.pos.y).into());
        }
    }

    pub fn create_capture_ball(&mut self, x: f64, y: f64) {
        if self.shots == 0 {
            return;
        }
        if !(Point { x, y }).is_in_rect(&self.rect) {
            return;
        }

        console::log_1(&format!("{} {}", x, y).into());
        console::log_1(
            &format!(
                "shots:{} level:{} captured: {}",
                self.shots, self.level_id, self.captured
            )
            .into(),
        );

        let mut active_ball = Ball::new(
            self.gen_next_id(),
            Point { x, y },
            Point { x: 0.0, y: 0.0 },
            ACTIVE_BALL,
            Expanding,
        );
        active_ball.is_captured = true;
        self.insert_object(active_ball.borrow_mut());
        self.shots -= 1;
    }

    pub fn handle_keydown_event(&mut self, key: &str) {
        let change: ChangeState = match key {
            "G" | "g" => ChangeState::PlayPause,
            "N" | "n" => ChangeState::NextLevel,
            "R" | "r" => ChangeState::RestartLevel,
            "Q" | "q" => ChangeState::Quit,
            _ => ChangeState::NoChange,
        };
        match change {
            ChangeState::PlayPause => self.pause_play(),
            ChangeState::NextLevel => {
                if self.check_win_lose() == GameResult::Won {
                    self.next_level()
                }
            }

            ChangeState::RestartLevel => self.restart(),
            ChangeState::Quit => self.quit(),
            ChangeState::NoChange => {}
        };
    }

    /// Handles a mouse click event and performs actions based on the game's state.
    pub fn handle_mouse_event(&mut self, click: i16, client_x: f64, client_y: f64) {
        // If the click is not a left-click (0), we do nothing.
        console::log_1(&format!("{} {}", client_x, client_y).into());

        if click != 0 {
            return;
        }

        let game_result: GameResult = self.check_win_lose();

        // Act based on the game's state.
        match game_result {
            GameResult::Playing => {
                self.create_capture_ball(client_x, client_y);
            }
            GameResult::Lost => {
                self.restart();
            }
            GameResult::Won => {
                self.next_level();
            } // Potentially handle other states or do nothing for them.
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeState {
    PlayPause,
    NextLevel,
    RestartLevel,
    Quit,
    NoChange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameResult {
    Playing,
    Lost,
    Won,
}
