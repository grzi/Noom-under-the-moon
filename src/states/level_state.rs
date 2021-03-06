use amethyst::assets::Handle;
use amethyst::core::ecs::{Builder, Entity, World, WorldExt};
use amethyst::core::{Parent, Transform};
use amethyst::renderer::{Camera, SpriteRender, SpriteSheet};
use amethyst::{GameData, SimpleState, SimpleTrans, StateData, Trans};

use crate::entities::blade_saw::BladeSawSprite;
use crate::entities::bonus::Bonus;
use crate::entities::collision::{Arrival, LandingPlatform};
use crate::entities::doors::{DoorState, PlasmaDoor};
use crate::entities::ship::{
    Coin, Ship, ShipFuel, ShipLife, ShipParent, ShipPowerLeftNumber, ShipPowerRightNumber,
    Thrusters,
};
use crate::resources::main_resource::{MainResource, MainSprites};
use crate::states::end_state::EndLevelState;
use crate::states::next_level::NextLevelState;
use crate::states::CurrentState;
use crate::utils::level_reader::{read_level, LevelConfig};
use crate::utils::sprites::plasma_doors::is_plasma_door_part;
use crate::utils::sprites::sprite_to_entities::{
    is_arrival, is_landing_platform_start, sprite_to_bonus_kind, sprite_to_canon,
    sprite_to_colliders, BLADE_SAW_SPRITE,
};
use crate::utils::sprites::*;
use amethyst::core::math::Point3;
use amethyst::ui::{Anchor, ScaleMode, UiCreator, UiImage, UiTransform};
use amethyst::utils::application_root_dir;
use serde_json::from_reader;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fs::File;

pub struct LevelState {
    pub level_nb: usize,
}

const MAX_LVL: usize = 10;

impl SimpleState for LevelState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;
        *world.write_resource::<CurrentState>() = CurrentState::Level;
        load_level(self.level_nb, world);
    }

    fn on_stop(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        data.world.delete_all();
        *data.world.write_resource::<CurrentState>() = CurrentState::Level;
    }

    fn fixed_update(&mut self, data: StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        let world = data.world;
        let (victory, current_level, should_go_to_next_level) = {
            let resource = world.read_resource::<MainResource>();
            (
                resource.victory,
                resource.current_level,
                resource.should_go_to_next_level,
            )
        };
        if victory && should_go_to_next_level {
            let new_level = current_level + 1;
            if new_level <= MAX_LVL {
                return Trans::Switch(Box::new(NextLevelState::new(new_level)));
            } else {
                return Trans::Switch(Box::new(EndLevelState));
            }
        }
        return Trans::None;
    }
}

fn load_level(lvl_number: usize, world: &mut World) {
    let level = read_level(lvl_number);
    let misc_spritesheet_handle = load_misc_spritesheet(world);
    let level_spritesheet_handle = load_level_spritesheet(world, lvl_number);
    let ship_spritesheet_handle = load_ship_spritesheet(world);
    let bullet_spritesheet_handle = load_bullets_spritesheet(world);
    let ship_explosion_handle = load_explosion_spritesheet(world);
    let numbers_spritesheet_handle = load_numbers_spritesheet(world);

    initialize_level_tileset(world, level_spritesheet_handle, &level);
    initialize_colliders_with_entitites(world, &level, misc_spritesheet_handle.clone());
    let ship = initialize_ship(world, &level, ship_spritesheet_handle);
    initialize_camera(world, ship);
    world.exec(|mut creator: UiCreator<'_>| {
        creator.create("ui/ui.ron", ());
    });
    initialize_level_ui(world, numbers_spritesheet_handle.clone(), lvl_number);
    initialize_power_ui(world, numbers_spritesheet_handle);
    initialize_life_and_fuel_ui(world);
    initialize_coins_ui(world, &level, misc_spritesheet_handle);
    let mut ship_resource = MainResource::new_from_level(Some(level), lvl_number);
    ship_resource.sprites = Some(MainSprites {
        explosion_sprite_render: ship_explosion_handle,
        bullet_sprite_render: bullet_spritesheet_handle,
    });
    world.insert(ship_resource);
}

fn initialize_colliders_with_entitites(
    world: &mut World,
    level: &LevelConfig,
    sprite_sheet_handle: Handle<SpriteSheet>,
) {
    for (point, sprite) in level.tiles.borrow() {
        if let Some(bonus) = sprite_to_bonus_kind(*sprite) {
            let mut transform = Transform::default();
            transform.set_translation_xyz(
                point.x as f32 * TILE_SIZE,
                point.y as f32 * TILE_SIZE,
                0.6,
            );

            world
                .create_entity()
                .with(Bonus {
                    initial_sprite: *sprite,
                    kind: bonus,
                    taken: false,
                })
                .with(SpriteRender {
                    sprite_sheet: sprite_sheet_handle.clone(),
                    sprite_number: *sprite,
                })
                .with(transform)
                .build();
        }

        let collider = sprite_to_colliders(
            *sprite,
            point.x as f32 * TILE_SIZE,
            point.y as f32 * TILE_SIZE,
        );

        if collider.is_some() {
            let mut builder = world.create_entity().with(collider.unwrap());
            if is_plasma_door_part(*sprite) {
                let mut transform = Transform::default();
                transform.set_translation_xyz(
                    point.x as f32 * TILE_SIZE,
                    point.y as f32 * TILE_SIZE,
                    0.6,
                );
                builder = builder
                    .with(PlasmaDoor {
                        initial_sprite: *sprite,
                        state: DoorState::Closed,
                    })
                    .with(SpriteRender {
                        sprite_sheet: sprite_sheet_handle.clone(),
                        sprite_number: *sprite,
                    })
                    .with(transform);
            }
            if is_landing_platform_start(*sprite) {
                builder = builder.with(LandingPlatform);
            }
            if is_arrival(*sprite) {
                builder = builder.with(Arrival);
            }
            if let Some(canon) = sprite_to_canon(*sprite, point.x as usize, point.y as usize) {
                builder = builder.with(canon);
            }
            builder.build();
        }
    }

    for blade_saw in level.blade_saws.iter() {
        let mut parent_transform = Transform::default();
        parent_transform.set_translation_xyz(
            blade_saw.start_x as f32 * TILE_SIZE,
            (level.height as f32 - blade_saw.start_y - 1.) as f32 * TILE_SIZE,
            0.6,
        );
        let parent = world
            .create_entity()
            .with(blade_saw.clone())
            .with(parent_transform)
            .build();

        let mut transform = Transform::default();

        world
            .create_entity()
            .with(BladeSawSprite)
            .with(SpriteRender {
                sprite_sheet: sprite_sheet_handle.clone(),
                sprite_number: BLADE_SAW_SPRITE,
            })
            .with(transform)
            .with(Parent { entity: parent })
            .build();
    }
}

fn initialize_level_tileset(
    world: &mut World,
    sprite_sheet_handle: Handle<SpriteSheet>,
    level: &LevelConfig,
) {
    let sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle,
        sprite_number: 0,
    };
    let mut t = Transform::default();
    t.set_translation_xyz(
        (TILE_SIZE * level.width as f32) / 2. - TILE_SIZE / 2.,
        (TILE_SIZE * level.height as f32) / 2. - TILE_SIZE / 2.,
        0.,
    );
    world.create_entity().with(sprite_render).with(t).build();
}

fn initialize_ship(
    world: &mut World,
    level: &LevelConfig,
    sprite_sheet_handle: Handle<SpriteSheet>,
) -> Entity {
    let ship_sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle.clone(),
        sprite_number: 0,
    };

    let thrusters_sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle.clone(),
        sprite_number: 5,
    };

    let mut transform = Transform::default();
    transform.set_translation_xyz(
        level.start_x as f32 * TILE_SIZE - 16.,
        (level.height - level.start_y) as f32 * TILE_SIZE,
        0.8,
    );

    let parent = world
        .create_entity()
        .with(ShipParent)
        .with(transform)
        .build();
    let transform_ship = Transform::default();
    let ship = world
        .create_entity()
        .with(ship_sprite_render)
        .with(Ship)
        .with(transform_ship)
        .with(Parent { entity: parent })
        .build();

    let mut transform_thruster = Transform::default();
    transform_thruster.set_translation_xyz(0., -32., 0.);
    world
        .create_entity()
        .with(thrusters_sprite_render)
        .with(Thrusters)
        .with(transform_thruster)
        .with(Parent { entity: ship })
        .build();

    parent
}

pub fn initialize_camera(world: &mut World, ship: Entity) {
    let mut transform = Transform::default();
    transform.set_translation_xyz(0., 0., 1.1);
    world
        .create_entity()
        .with(Camera::standard_2d(SCREEN_WIDTH, SCREEN_HEIGHT))
        .with(transform)
        .with(Parent { entity: ship })
        .build();
}

fn initialize_power_ui(world: &mut World, spritesheet: Handle<SpriteSheet>) {
    let mut power_nb_left_transform = UiTransform::new(
        "power_left".to_string(),
        Anchor::BottomLeft,
        Anchor::BottomLeft,
        0.82954545454,
        0.02056944444,
        10.,
        0.01420454545,
        0.02430555555,
    );
    power_nb_left_transform.scale_mode = ScaleMode::Percent;

    world
        .create_entity()
        .with(power_nb_left_transform)
        .with(UiImage::Sprite(SpriteRender {
            sprite_sheet: spritesheet.clone(),
            sprite_number: 0,
        }))
        .with(ShipPowerLeftNumber)
        .build();

    let mut power_nb_right_transform = UiTransform::new(
        "power_right".to_string(),
        Anchor::BottomLeft,
        Anchor::BottomLeft,
        0.84754545454,
        0.02056944444,
        10.,
        0.01420454545,
        0.02430555555,
    );
    power_nb_right_transform.scale_mode = ScaleMode::Percent;

    world
        .create_entity()
        .with(power_nb_right_transform)
        .with(UiImage::Sprite(SpriteRender {
            sprite_sheet: spritesheet.clone(),
            sprite_number: 0,
        }))
        .with(ShipPowerRightNumber)
        .build();
}

fn initialize_level_ui(world: &mut World, spritesheet: Handle<SpriteSheet>, lvl_number: usize) {
    let level_sprites = format_lvl_number(lvl_number);
    let mut level_nb_left_transform = UiTransform::new(
        "level_left".to_string(),
        Anchor::BottomLeft,
        Anchor::BottomLeft,
        0.64488636363,
        0.02056944444,
        10.,
        0.01420454545,
        0.02430555555,
    );
    level_nb_left_transform.scale_mode = ScaleMode::Percent;

    world
        .create_entity()
        .with(level_nb_left_transform)
        .with(UiImage::Sprite(SpriteRender {
            sprite_sheet: spritesheet.clone(),
            sprite_number: level_sprites.0,
        }))
        .build();

    let mut level_nb_right_transform = UiTransform::new(
        "level_right".to_string(),
        Anchor::BottomLeft,
        Anchor::BottomLeft,
        0.66088636363,
        0.02056944444,
        10.,
        0.01420454545,
        0.02430555555,
    );
    level_nb_right_transform.scale_mode = ScaleMode::Percent;

    world
        .create_entity()
        .with(level_nb_right_transform)
        .with(UiImage::Sprite(SpriteRender {
            sprite_sheet: spritesheet.clone(),
            sprite_number: level_sprites.1,
        }))
        .build();
}

fn format_lvl_number(lvl_number: usize) -> (usize, usize) {
    let lvl_nb_str = lvl_number.to_string();
    if lvl_nb_str.len() > 1 {
        return (
            lvl_nb_str[..1].parse().unwrap(),
            lvl_nb_str[1..2].parse().unwrap(),
        );
    }
    (0, lvl_nb_str.parse().unwrap())
}

fn initialize_life_and_fuel_ui(world: &mut World) {
    let power_spritesheet_handle = load_power_spritesheet(world);
    for life_point in 0..3 {
        let mut life_point_transform = UiTransform::new(
            format!("life_{}", life_point.to_string()),
            Anchor::BottomLeft,
            Anchor::BottomLeft,
            0.17755681818 + (life_point as f32 * (0.00426136363 + 0.01420454545)),
            0.01215277777,
            10.,
            0.01420454545,
            0.04166666666,
        );
        life_point_transform.scale_mode = ScaleMode::Percent;
        world
            .create_entity()
            .with(life_point_transform)
            .with(ShipLife {
                life_point: life_point as u8 + 1,
            })
            .with(UiImage::Sprite(SpriteRender {
                sprite_sheet: power_spritesheet_handle.clone(),
                sprite_number: 0,
            }))
            .build();
    }

    for fuel_point in 0..10 {
        let mut fuel_point_transform = UiTransform::new(
            format!("fuel_{}", fuel_point.to_string()),
            Anchor::BottomLeft,
            Anchor::BottomLeft,
            0.32244318181 + (fuel_point as f32 * (0.00426136363 + 0.01420454545)),
            0.01215277777,
            10.,
            0.01420454545,
            0.04166666666,
        );
        fuel_point_transform.scale_mode = ScaleMode::Percent;
        world
            .create_entity()
            .with(fuel_point_transform)
            .with(ShipFuel {
                fuel_point: fuel_point as u8,
            })
            .with(UiImage::Sprite(SpriteRender {
                sprite_sheet: power_spritesheet_handle.clone(),
                sprite_number: 0,
            }))
            .build();
    }
}

fn initialize_coins_ui(world: &mut World, config: &LevelConfig, spritesheet: Handle<SpriteSheet>) {
    for coin in 0..config.coin_nb {
        let mut coins_transform = UiTransform::new(
            format!("coin_{}", coin.to_string()),
            Anchor::BottomLeft,
            Anchor::BottomLeft,
            0.10279545454 + (coin as f32 * (0.04545454545 + 0.00008454545)),
            0.0716590909,
            10.,
            0.04545454545,
            0.04845454545,
        );
        coins_transform.scale_mode = ScaleMode::Percent;
        world
            .create_entity()
            .with(coins_transform)
            .with(Coin { coin_id: coin + 1 })
            .with(UiImage::Sprite(SpriteRender {
                sprite_sheet: spritesheet.clone(),
                sprite_number: 87,
            }))
            .build();
    }
}
