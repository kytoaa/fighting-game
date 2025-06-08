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
    meter: PlayerMeter,
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
                value: 1.0,
                display_value: 1.0,
                behind_value: 1.0,
                behind_display_value: 1.0,
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
                value: 1.0,
                display_value: 1.0,
            },
            meter: PlayerMeter {
                position: Vector2::new(
                    if IS_PLAYER_1 {
                        BORDER_L + 370.0
                    } else {
                        BORDER_R - 370.0
                    },
                    BORDER_B + 70.0,
                ),
                value: 1.0,
                display_value: 1.0,
            },
        }
    }

    fn update(&mut self, state: &fighting_game::PlayerState) {
        self.health_bar.set_progress(
            state.health_percent,
            state.health_percent + state.combo_damage_health_percent.unwrap_or(0.0),
        );
        self.burst_meter.set_progress(state.burst_percent);
        self.meter.set_progress(state.meter_percent);

        self.health_bar.update();
        self.burst_meter.update();
        self.meter.update();
    }

    fn get_render_info(&self) -> impl Iterator<Item = renderer::Primative> {
        self.get_player_ui_background_ui(!IS_PLAYER_1)
            .chain(self.health_bar.get_render_info(!IS_PLAYER_1))
            .chain(self.meter.get_render_info(!IS_PLAYER_1))
            .chain(vec![self.burst_meter.get_render_info(!IS_PLAYER_1)].into_iter())
    }

    fn get_player_ui_background_ui(
        &self,
        flipped: bool,
    ) -> impl Iterator<Item = renderer::Primative> {
        vec![
            {
                const OVERALL_OFFSET: Vector2 = Vector2::new(0.0, -5.0);
                const OFFSET: Vector2 = Vector2::new(8.0, -10.0);
                const TOP: Vector2 = Vector2::new(-400.0, 8.0).mul(1.01);
                const BOTTOM: Vector2 = Vector2::new(-395.0, -12.0).mul(1.01);
                if !flipped {
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
                        top_r: self.health_bar.position + TOP.flip_x() + OVERALL_OFFSET.flip_x(),
                        bottom_l: self.health_bar.position
                            + OFFSET.flip_x()
                            + OVERALL_OFFSET.flip_x(),
                        bottom_r: self.health_bar.position
                            + OFFSET.flip_x()
                            + BOTTOM.flip_x()
                            + OVERALL_OFFSET.flip_x(),
                        colors: Box::new([(0.0, 0.0, 0.0, 1.0); 4]),
                    }
                }
            },
            {
                const OVERALL_OFFSET: Vector2 = Vector2::new(2.0, -4.0);
                const OFFSET: Vector2 = Vector2::new(-4.0, -12.0);
                const TOP: Vector2 = Vector2::new(-300.0, 8.0).mul(1.05);
                const BOTTOM: Vector2 = Vector2::new(-320.0, -12.0).mul(1.05);
                if !flipped {
                    renderer::Primative {
                        top_r: self.meter.position + OVERALL_OFFSET,
                        top_l: self.meter.position + TOP + OVERALL_OFFSET,
                        bottom_r: self.meter.position + OFFSET + OVERALL_OFFSET,
                        bottom_l: self.meter.position + OFFSET + BOTTOM + OVERALL_OFFSET,
                        colors: Box::new([(0.0, 0.0, 0.0, 1.0); 4]),
                    }
                } else {
                    renderer::Primative {
                        top_l: self.meter.position + OVERALL_OFFSET.flip_x(),
                        top_r: self.meter.position + TOP.flip_x() + OVERALL_OFFSET.flip_x(),
                        bottom_l: self.meter.position + OFFSET.flip_x() + OVERALL_OFFSET.flip_x(),
                        bottom_r: self.meter.position
                            + OFFSET.flip_x()
                            + BOTTOM.flip_x()
                            + OVERALL_OFFSET.flip_x(),
                        colors: Box::new([(0.0, 0.0, 0.0, 1.0); 4]),
                    }
                }
            },
        ]
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
    let width = percent * max_len;
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
    behind_value: f32,
    behind_display_value: f32,
}

impl HealthBar {
    fn set_progress(&mut self, new_value: f32, behind_value: f32) {
        self.value = new_value;
        self.behind_value = behind_value;
    }
    fn update(&mut self) {
        self.display_value = lerp(self.value, self.display_value, 0.9);
        if self.behind_display_value != self.behind_value {
            self.behind_display_value = lerp(self.behind_value, self.behind_display_value, 0.9);
        }

        if (self.display_value - self.value).abs() < 0.01 {
            self.display_value = self.value;
        }
        if (self.behind_display_value - self.behind_value).abs() < 0.01 {
            self.behind_display_value = self.behind_value;
        }
    }
    fn get_render_info(&self, flipped: bool) -> impl Iterator<Item = renderer::Primative> {
        const OFFSET: Vector2 = Vector2::new(8.0, -10.0);
        const TOP: Vector2 = Vector2::new(-400.0, 8.0);
        const BOTTOM: Vector2 = Vector2::new(-395.0, -12.0);

        let progress = self.display_value;

        if !flipped {
            vec![
                renderer::Primative {
                    top_r: self.position,
                    top_l: self.position + TOP * self.behind_display_value,
                    bottom_r: self.position + OFFSET,
                    bottom_l: self.position + OFFSET + BOTTOM * self.behind_display_value,
                    colors: Box::new([
                        (1.0, 0.0784, 0.1725, 1.0),
                        (1.0, 0.1804, 0.2627, 1.0),
                        (1.0, 0.0784, 0.1725, 1.0),
                        (1.0, 0.0784, 0.1725, 1.0),
                    ]),
                },
                renderer::Primative {
                    top_r: self.position,
                    top_l: self.position + TOP * progress,
                    bottom_r: self.position + OFFSET,
                    bottom_l: self.position + OFFSET + BOTTOM * progress,
                    colors: Box::new([
                        (1.0, 0.7529, 0.1294, 1.0),
                        (1.0, 0.7529, 0.1294, 1.0),
                        (1.0, 0.4059, 0.1294, 1.0),
                        (1.0, 0.4059, 0.1294, 1.0),
                    ]),
                },
            ]
            .into_iter()
        } else {
            vec![
                renderer::Primative {
                    top_l: self.position,
                    top_r: self.position + TOP.flip_x() * self.behind_display_value,
                    bottom_l: self.position + OFFSET.flip_x(),
                    bottom_r: self.position
                        + OFFSET.flip_x()
                        + BOTTOM.flip_x() * self.behind_display_value,
                    colors: Box::new([
                        (1.0, 0.0784, 0.1725, 1.0),
                        (1.0, 0.0784, 0.1725, 1.0),
                        (1.0, 0.1804, 0.2627, 1.0),
                        (1.0, 0.0784, 0.1725, 1.0),
                    ]),
                },
                renderer::Primative {
                    top_l: self.position,
                    top_r: self.position + TOP.flip_x() * progress,
                    bottom_l: self.position + OFFSET.flip_x(),
                    bottom_r: self.position + OFFSET.flip_x() + BOTTOM.flip_x() * progress,
                    colors: Box::new([
                        (1.0, 0.4059, 0.1294, 1.0),
                        (1.0, 0.4059, 0.1294, 1.0),
                        (1.0, 0.7529, 0.1294, 1.0),
                        (1.0, 0.7529, 0.1294, 1.0),
                    ]),
                },
            ]
            .into_iter()
        }
    }
}

struct PlayerMeter {
    position: Vector2,
    value: f32,
    display_value: f32,
}

impl PlayerMeter {
    fn set_progress(&mut self, new_value: f32) {
        self.value = new_value;
    }
    fn update(&mut self) {
        self.display_value = lerp(self.value, self.display_value, 0.9);

        if (self.display_value - self.value).abs() < 0.01 {
            self.display_value = self.value;
        }
    }
    fn get_render_info(&self, flipped: bool) -> impl Iterator<Item = renderer::Primative> {
        const OFFSET: Vector2 = Vector2::new(-4.0, -12.0);
        const TOP: Vector2 = Vector2::new(-300.0, 8.0);
        const BOTTOM: Vector2 = Vector2::new(-320.0, -12.0);

        let progress = self.display_value;
        let full_count = (progress * 3.0).ceil() as u32;

        if !flipped {
            (0..full_count)
                .map(|i| {
                    let min = i as f32 / 3.0 + (0.02 * i as f32);
                    let max = min + f32::min(progress - (min - (0.02 * i as f32)), 1.0 / 3.0);
                    renderer::Primative {
                        top_r: self.position + TOP * min,
                        top_l: self.position + TOP * max,
                        bottom_r: self.position + OFFSET + BOTTOM * min,
                        bottom_l: self.position + OFFSET + BOTTOM * max,
                        colors: Box::new(
                            [if i == full_count - 1 && progress != 1.0 {
                                (0.196, 0.6235, 1.0, 1.0)
                            } else {
                                (0.251, 1.0, 0.7, 1.0)
                            }; 4],
                        ),
                    }
                })
                .collect::<Vec<_>>()
                .into_iter()
        } else {
            (0..full_count)
                .map(|i| {
                    let min = i as f32 / 3.0 + (0.02 * i as f32);
                    let max = min + f32::min(progress - (min - (0.02 * i as f32)), 1.0 / 3.0);
                    renderer::Primative {
                        top_l: self.position + TOP.flip_x() * min,
                        top_r: self.position + TOP.flip_x() * max,
                        bottom_l: self.position + OFFSET.flip_x() + BOTTOM.flip_x() * min,
                        bottom_r: self.position + OFFSET.flip_x() + BOTTOM.flip_x() * max,
                        colors: Box::new(
                            [if i == full_count - 1 && progress != 1.0 {
                                (0.196, 0.6235, 1.0, 1.0)
                            } else {
                                (0.251, 1.0, 0.7, 1.0)
                            }; 4],
                        ),
                    }
                })
                .collect::<Vec<_>>()
                .into_iter()
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
