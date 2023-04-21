use bevy::prelude::*;

#[derive(Component)]
struct MyCamera;

fn camera_input(
    mut q: Query<(&mut MyCamera, &mut Transform, &Projection)>,
    keys: Res<Input<KeyCode>>
) {
    for (mut cam, mut transform, projection) in q.iter_mut() {
        /*if keys.just_pressed(KeyCode::Space) {
            transform.rotate_axis(Vec3::X, 25.0);
        }*/

        let v = {
            let mut _v = Vec3::ZERO;
            if keys.just_pressed(KeyCode::Up) {
                _v += transform.forward();
            }
            if keys.just_pressed(KeyCode::Down) {
                _v += transform.back()
            }
            if keys.just_pressed(KeyCode::Left) {
                _v += transform.left();
            }
            if keys.just_pressed(KeyCode::Right) {
                _v += transform.right();
            }

            if keys.just_pressed(KeyCode::Space) {
                _v += transform.up();
            }

            if keys.just_pressed(KeyCode::LControl) {
                _v += transform.down();
            }

            _v
        };

        transform.translation += v;
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(50.0, 15.0, 50.0))
                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
            ..Default::default()
        },
        MyCamera,
    ));
}
pub struct MyCameraPlugin;

impl Plugin for MyCameraPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(setup)
            .add_system(camera_input);
    }
}