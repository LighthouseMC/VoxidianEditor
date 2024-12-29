use crate::prelude::*;
use std::mem;


pub struct LayoutPlugin;
impl Plugin for LayoutPlugin {
    fn build(&self, app : &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(Update, update_grabbers_grabbing);
        app.add_systems(Update, update_grabbers_drag);
    }
}


#[derive(Component)]
struct ResizeGrabber(Entity);
#[derive(Component)]
struct ResizeGrabberActive(Option<Vec2>);


fn setup(
    mut commands : Commands,
        assets   : Res<AssetServer>
) {

    commands.spawn((
        Camera2d,
        IsDefaultUiCamera,
        //UiBoxShadowSamples(6)
    ));

    commands.spawn(Node {
        width  : Val::Percent(100.0),
        height : Val::Percent(100.0),
        ..default()
    }).insert(PickingBehaviour::IGNORE)
        .with_children(|parent| {

            // File tree.
            let file_tree = parent.spawn((
                Node {
                    width          : Val::Px(300.0),
                    min_width      : Val::Px(200.0),
                    max_width      : Val::Percent(87.5),
                    flex_direction : FlexDirection::Column,
                    overflow       : Overflow::scroll_y(),
                    ..default()
                },
                BackgroundColour(Colour::srgb(0.05, 0.05, 0.05)),
                PickingBehaviour::IGNORE
            )).id();

            // Resize grabber.
            parent.spawn((
                Node {
                    width          : Val::Px(5.0),
                    height         : Val::Percent(100.0),
                    flex_direction : FlexDirection::Column,
                    border         : UiRect::right(Val::Px(1.0)),
                    ..default()
                },
                Button,
                ResizeGrabber(file_tree),
                BackgroundColour(Colour::srgb(0.0, 0.0, 0.0)),
                BorderColour(Colour::srgb(0.25, 0.25, 0.25))
            ));

            parent.spawn((
                Node {
                    flex_direction : FlexDirection::Column,
                    flex_grow      : 1.0,
                    width          : Val::Auto,
                    ..default()
                },
                PickingBehaviour::IGNORE
            ))
                .with_children(|parent| {

                    // Open file tabs.
                    parent.spawn((
                        Node {
                            height         : Val::Px(37.5),
                            flex_direction : FlexDirection::Row,
                            border         : UiRect::bottom(Val::Px(1.0)),
                            overflow       : Overflow::scroll_x(),
                            ..default()
                        },
                        BackgroundColour(Colour::srgb(0.05, 0.05, 0.05)),
                        BorderColour(Colour::srgb(0.25, 0.25, 0.25)),
                        PickingBehaviour::IGNORE
                    ));

                    // Editor.
                    parent.spawn((
                        Node {
                            flex_direction : FlexDirection::Row,
                            flex_grow      : 1.0,
                            overflow       : Overflow::scroll(),
                            ..default()
                        },
                        PickingBehaviour::IGNORE
                    ));

                });

        });

}


fn update_grabbers_grabbing(
    mut commands : Commands,
    mut grabbers : Query<(Entity, &Interaction, &mut BackgroundColour), (Changed<Interaction>, With<Button>, With<ResizeGrabber>)>
) {
    for (entity, interact, mut colour) in &mut grabbers {
        match (interact) {
            Interaction::Pressed => {
                *colour = BackgroundColour(Colour::srgb(0.0, 0.0, 0.0));
                commands.entity(entity).insert(ResizeGrabberActive(None));
            },
            Interaction::Hovered => {
                *colour = BackgroundColour(Colour::srgb(0.375, 0.375, 0.375));
                commands.entity(entity).remove::<ResizeGrabberActive>();
            },
            Interaction::None => {
                *colour = BackgroundColour(Colour::srgb(0.0, 0.0, 0.0));
                commands.entity(entity).remove::<ResizeGrabberActive>();
            }
        };
    }
}


fn update_grabbers_drag(
        window   : Query<(&Window), With<PrimaryWindow>>,
        camera   : Query<(&Camera, &GlobalTransform)>,
    mut grabbers : Query<(&ResizeGrabber, &mut ResizeGrabberActive)>,
    mut nodes    : Query<(&mut Node)>
) {
    let (window) = window.single();
    let (camera, camera_transform) = camera.single();

    let Some(cursor_screen) = window.cursor_position() else { return };
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_screen) else { return };
    let cursor_world = ray.origin.truncate();

    for (ResizeGrabber(target), mut old_cursor_world) in &mut grabbers {
        let old_cursor_world = mem::replace(&mut old_cursor_world.0, Some(cursor_world)).unwrap_or(cursor_world);
        if let Ok((mut node)) = nodes.get_mut(*target) {
            node.width = Val::Px((window.width() / 2.0) + cursor_world.x);
        }
    }
}
