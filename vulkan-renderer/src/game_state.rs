pub struct GameState {}

enum GameStateInner {
    Game(Game),
    CharacterSelect(CharacterSelect),
}

struct CharacterSelect;

struct Game {
    game: fighting_game::Game,
    ui: crate::ui::PlayerUi,
}
