use super::{renderer, Vector2, WINDOW_HEIGHT, WINDOW_WIDTH};

const BORDER_L: f32 = -(WINDOW_WIDTH as f32) / 2.0;
const BORDER_R: f32 = WINDOW_WIDTH as f32 / 2.0;
const BORDER_T: f32 = WINDOW_HEIGHT as f32 / 2.0;
const BORDER_B: f32 = -(WINDOW_HEIGHT as f32) / 2.0;

type Color = (f32, f32, f32, f32);

pub struct PlayerUi {
    player_1: PlayerSpecificUI<true>,
    player_2: PlayerSpecificUI<false>,
}
impl PlayerUi {
    pub fn new() -> Self {
        Self {
            player_1: PlayerSpecificUI::new(),
            player_2: PlayerSpecificUI::new(),
        }
    }
    pub fn update(
        &mut self,
        player_1_state: &fighting_game::PlayerState,
        player_2_state: &fighting_game::PlayerState,
    ) {
        self.player_1.update(player_1_state);
        self.player_2.update(player_2_state);
    }
    pub fn get_render_info(
        &self,
    ) -> (
        impl Iterator<Item = renderer::Sprite>,
        impl Iterator<Item = renderer::Primative>,
    ) {
        (
            vec![].into_iter(),
            self.player_1
                .get_render_info()
                .chain(self.player_2.get_render_info()),
        )
    }
}

struct PlayerSpecificUI<const IS_PLAYER_1: bool> {
    health_bar: HealthBar,
    burst_meter: ProgressBar,
    meter: ProgressBar,
}

impl<const IS_PLAYER_1: bool> PlayerSpecificUI<IS_PLAYER_1> {
    fn new() -> Self {
        PlayerSpecificUI {
            health_bar: HealthBar {
                position: Vector2::new(
                    if IS_PLAYER_1 {
                        BORDER_L + 500.0
                    } else {
                        BORDER_R - 500.0
                    },
                    BORDER_T - 60.0,
                ),
                value: 100.0,
                display_value: 100.0,
            },
            burst_meter: ProgressBar {
                position: Vector2::new(
                    if IS_PLAYER_1 {
                        BORDER_L + 60.0
                    } else {
                        BORDER_R - 60.0
                    },
                    BORDER_T - 100.0,
                ),
                color: (0.251, 0.8627, 1.0, 1.0),
                height: 30.0,
                max_width: 60.0,
                value: 100.0,
                display_value: 100.0,
            },
            meter: ProgressBar {
                position: Vector2::new(
                    if IS_PLAYER_1 {
                        BORDER_L + 80.0
                    } else {
                        BORDER_R - 80.0
                    },
                    BORDER_B + 50.0,
                ),
                color: (0.251, 1.0, 0.8, 1.0),
                height: 10.0,
                max_width: 300.0,
                value: 100.0,
                display_value: 100.0,
            },
        }
    }

    fn update(&mut self, state: &fighting_game::PlayerState) {
        self.health_bar.set_progress(state.health_percent);
        self.burst_meter.set_progress(state.burst_percent);
        self.meter.set_progress(state.meter_percent);

        self.health_bar.update();
        self.burst_meter.update();
        self.meter.update();
    }

    fn get_render_info(&self) -> impl Iterator<Item = renderer::Primative> {
        self.get_player_ui_background_ui(!IS_PLAYER_1).chain(
            vec![
                self.health_bar.get_render_info(!IS_PLAYER_1),
                self.burst_meter.get_render_info(!IS_PLAYER_1),
                self.meter.get_render_info(!IS_PLAYER_1),
            ]
            .into_iter(),
        )
    }

    fn get_player_ui_background_ui(
        &self,
        flipped: bool,
    ) -> impl Iterator<Item = renderer::Primative> {
        const OVERALL_OFFSET: Vector2 = Vector2::new(0.0, -5.0);
        const OFFSET: Vector2 = Vector2::new(5.0, -10.0);
        const TOP: Vector2 = Vector2::new(-400.0, 8.0).mul(1.01);
        const BOTTOM: Vector2 = Vector2::new(-395.0, -12.0).mul(1.01);

        vec![if !flipped {
            renderer::Primative {
                top_r: self.health_bar.position + OVERALL_OFFSET,
                top_l: self.health_bar.position + TOP + OVERALL_OFFSET,
                bottom_r: self.health_bar.position + OFFSET + OVERALL_OFFSET,
                bottom_l: self.health_bar.position + OFFSET + BOTTOM + OVERALL_OFFSET,
                colors: Box::new([(0.0, 0.0, 0.0, 1.0); 4]),
            }
        } else {
            renderer::Primative {
                top_l: self.health_bar.position + OVERALL_OFFSET.flip_x(),
                top_r: self.health_bar.position + (TOP + OVERALL_OFFSET).flip_x(),
                bottom_l: self.health_bar.position + (OFFSET + OVERALL_OFFSET).flip_x(),
                bottom_r: self.health_bar.position + (OFFSET + BOTTOM + OVERALL_OFFSET).flip_x(),
                colors: Box::new([(0.0, 0.0, 0.0, 1.0); 4]),
            }
        }]
        .into_iter()
    }
}

fn basic_progress_bar(
    percent: f32,
    max_len: f32,
    flipped: bool,
    height: f32,
    position: Vector2,
    color: Color,
) -> renderer::Primative {
    let width = percent * max_len / 100.0;
    renderer::Primative::rect(
        if flipped {
            position - Vector2::new(width, 0.0)
        } else {
            position
        },
        Vector2::new(width, height),
        color,
    )
}

struct ProgressBar {
    position: Vector2,
    color: Color,
    height: f32,
    max_width: f32,

    value: f32,
    display_value: f32,
}

impl ProgressBar {
    fn set_progress(&mut self, new_value: f32) {
        self.value = new_value;
    }
    fn update(&mut self) {
        self.display_value = lerp(self.value, self.display_value, 0.9);

        if (self.display_value - self.value).abs() < 1.0 {
            self.display_value = self.value;
        }
    }
    fn get_render_info(&self, flipped: bool) -> renderer::Primative {
        basic_progress_bar(
            self.display_value,
            self.max_width,
            flipped,
            self.height,
            self.position,
            self.color,
        )
    }
}

struct HealthBar {
    position: Vector2,
    value: f32,
    display_value: f32,
}

impl HealthBar {
    fn set_progress(&mut self, new_value: f32) {
        self.value = new_value;
    }
    fn update(&mut self) {
        self.display_value = lerp(self.value, self.display_value, 0.9);

        if (self.display_value - self.value).abs() < 1.0 {
            self.display_value = self.value;
        }
    }
    fn get_render_info(&self, flipped: bool) -> renderer::Primative {
        const OFFSET: Vector2 = Vector2::new(5.0, -10.0);
        const TOP: Vector2 = Vector2::new(-400.0, 8.0);
        const BOTTOM: Vector2 = Vector2::new(-395.0, -12.0);

        let progress = self.display_value;

        if !flipped {
            renderer::Primative {
                top_r: self.position,
                top_l: self.position + TOP * progress / 100.0,
                bottom_r: self.position + OFFSET,
                bottom_l: self.position + OFFSET + BOTTOM * progress / 100.0,
                colors: Box::new([
                    (1.0, 0.7529, 0.1294, 1.0),
                    (1.0, 0.7529, 0.1294, 1.0),
                    (1.0, 0.4059, 0.1294, 1.0),
                    (1.0, 0.4059, 0.1294, 1.0),
                ]),
            }
        } else {
            renderer::Primative {
                top_l: self.position,
                top_r: self.position + TOP.flip_x() * progress / 100.0,
                bottom_l: self.position + OFFSET.flip_x(),
                bottom_r: self.position + OFFSET.flip_x() + BOTTOM.flip_x() * progress / 100.0,
                colors: Box::new([
                    (1.0, 0.4059, 0.1294, 1.0),
                    (1.0, 0.4059, 0.1294, 1.0),
                    (1.0, 0.7529, 0.1294, 1.0),
                    (1.0, 0.7529, 0.1294, 1.0),
                ]),
            }
        }
    }
}

fn lerp<A, B, C, D, R>(a: A, b: B, t: f32) -> R
where
    A: Sized + Copy,
    B: std::ops::Sub<A, Output = C>,
    C: std::ops::Mul<f32, Output = D>,
    A: std::ops::Add<D, Output = R>,
{
    let d = b - a;
    a + d * t
}
