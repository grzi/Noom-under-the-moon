use crate::entities::canons::{
    canon_kind_to_bullet_life_duration, canon_kind_to_bullet_speed, Bullet, Canon, CanonKind,
};
use crate::entities::collision::{are_colliding, compute_is_eligible_for_collision, Colliders};
use crate::entities::ship::ShipParent;
use crate::resources::main_resource::MainResource;
use crate::utils::sound::{play_air, play_hit, Sounds};
use crate::utils::sprites::sprite_to_entities::init_bullet_collider;
use crate::utils::Direction;
use amethyst::assets::AssetStorage;
use amethyst::audio::output::Output;
use amethyst::audio::Source;
use amethyst::core::ecs::{
    Entities, Join, Read, ReadExpect, ReadStorage, System, Write, WriteStorage,
};
use amethyst::core::math::Vector3;
use amethyst::core::{Time, Transform};
use geo::Polygon;

const DEFAULT_AIR_TIMER: f32 = 0.2;

pub struct BulletSystem {
    pub play_air_timer: f32,
}

impl Default for BulletSystem {
    fn default() -> Self {
        BulletSystem {
            play_air_timer: DEFAULT_AIR_TIMER,
        }
    }
}

impl<'s> System<'s> for BulletSystem {
    type SystemData = (
        WriteStorage<'s, Bullet>,
        ReadStorage<'s, Canon>,
        WriteStorage<'s, Transform>,
        ReadStorage<'s, Colliders>,
        ReadStorage<'s, ShipParent>,
        Write<'s, MainResource>,
        Read<'s, Time>,
        Entities<'s>,
        Read<'s, AssetStorage<Source>>,
        ReadExpect<'s, Sounds>,
        Option<Read<'s, Output>>,
    );

    fn run(
        &mut self,
        (
            mut bullets,
            canons,
            mut transforms,
            colliders,
            ships,
            mut main_resource,
            time,
            entities,
            storage,
            sounds,
            audio_output,
        ): Self::SystemData,
    ) {
        self.play_air_timer -= time.delta_seconds();
        let mut ship_polygon = Vec::new();
        for (_ship, transform) in (&ships, &transforms).join() {
            ship_polygon = main_resource
                .get_colliders_for_collision(transform.translation().x, transform.translation().y);
        }
        let mut bullet_vec: Vec<(u32, Colliders)> = Vec::new();
        for (bullet, transform, entity) in (&mut bullets, &mut transforms, &entities).join() {
            let colliders = init_bullet_collider(
                &bullet.kind,
                transform.translation().x,
                transform.translation().y,
            );
            match bullet.kind {
                CanonKind::Air => {
                    transform.set_scale(Vector3::new(
                        1.,
                        1. + (canon_kind_to_bullet_life_duration(&bullet.kind)
                            - bullet.life_duration)
                            / 1.5,
                        1.0,
                    ));
                }
                _ => {}
            }
            if are_colliding(colliders.polygons(), &ship_polygon) {
                match bullet.kind {
                    CanonKind::Air => {
                        if self.play_air_timer <= 0. {
                            play_air(&*sounds, &storage, audio_output.as_deref());
                            self.play_air_timer = DEFAULT_AIR_TIMER;
                        }
                        match bullet.direction {
                            Direction::Left => {
                                main_resource.x_force -= 3. * time.delta_seconds();
                            }
                            Direction::Right => {
                                main_resource.x_force += 3. * time.delta_seconds();
                            }
                            _ => {}
                        }
                    }
                    _ => {
                        play_hit(&*sounds, &storage, audio_output.as_deref());
                        main_resource.bullet_hit();

                        let _res = entities.delete(entity);
                    }
                }
            } else {
                bullet_vec.push((entity.id(), colliders));
            }
            let bullet_speed = canon_kind_to_bullet_speed(&bullet.kind);
            match bullet.direction {
                Direction::Left => transform.append_translation_xyz(
                    -1. * bullet_speed * time.delta_seconds(),
                    0.,
                    0.,
                ),
                Direction::Right => {
                    transform.append_translation_xyz(bullet_speed * time.delta_seconds(), 0., 0.)
                }
                Direction::Top => {
                    transform.append_translation_xyz(0., bullet_speed * time.delta_seconds(), 0.)
                }
                Direction::Bottom => transform.append_translation_xyz(
                    0.,
                    -1. * bullet_speed * time.delta_seconds(),
                    0.,
                ),
                _ => transform.append_translation_xyz(0., 0., 0.),
            };
            bullet.life_duration -= time.delta_seconds();
            if bullet.life_duration <= 0. {
                let _res = entities.delete(entity);
            }
        }

        let joined: Vec<_> = (&colliders, !&bullets, !&canons)
            .join()
            .filter(|(a, b, c)| {
                return bullet_vec
                    .iter()
                    .any(|(id, collider)| compute_is_eligible_for_collision(*a, &collider));
            })
            .flat_map(|(a, b, c)| a.to_owned_polygons())
            .collect();
        for (id, col) in bullet_vec.iter() {
            if are_colliding(&col.to_owned_polygons(), &joined) {
                let e = entities.entity(*id);
                let _res = entities.delete(e);
            }
        }
    }
}
