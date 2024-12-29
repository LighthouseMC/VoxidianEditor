mod layout;


mod prelude {
    pub use bevy::prelude::*;
    pub use bevy::prelude::PickingBehavior as PickingBehaviour;
    pub use bevy::prelude::ClearColor as ClearColour;
    pub use bevy::prelude::Color as Colour;
    pub use bevy::prelude::BackgroundColor as BackgroundColour;
    pub use bevy::prelude::BorderColor as BorderColour;
    pub use bevy::window::PrimaryWindow;
}
use prelude::*;
pub use bevy::asset::AssetMetaCheck;


fn main() {

    App::new()
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window : Some(Window {
                    fit_canvas_to_parent : true,
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                meta_check : AssetMetaCheck::Never,
                ..default()
            })
        )
        .insert_resource(ClearColour(Colour::BLACK))
        .add_plugins(layout::LayoutPlugin)
        .run();

}
