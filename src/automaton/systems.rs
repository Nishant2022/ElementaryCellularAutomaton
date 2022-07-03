use bevy::{prelude::*, input::mouse::{MouseWheel, MouseScrollUnit}, window::WindowResized, ecs::event::Events};
use rand::{distributions::Uniform, prelude::Distribution};
use crate::{WinSize, 
    automaton::{
        enums::CellState, 
        resources::{
            CellSettings, 
            CellGrid, 
            RuleChanged
        },
        components::Cell,
    }
};

// region:      Systems

pub fn cell_spawn_system(mut commands: Commands, win_size: Res<WinSize>) {

    // num_cells defines the length and width of cell grid
    // should optimally be a value 2^n + 1
    const NUM_CELLS: usize = 65;

    // cell_size is the size of each cell
    let cell_size: f32 = win_size.w / NUM_CELLS as f32 * 2.0;
    
    // init_rule is the initial rule that will be calculated
    let init_rule: u8 = 30;

    // Insert CellSettings resources
    commands.insert_resource(CellSettings {
        cell_size: cell_size,
        num_cells: NUM_CELLS as u32,
        dead_color: Color::BLACK,
        alive_color: Color::WHITE,
        rule_num: init_rule,
        rule: get_rule(init_rule),
        random: false,
    });

    // Create cell grid
    let mut cell_grid = CellGrid {grid: Vec::new()};

    // Initialize Cell Grid with grid of dead cells
    for i in 0..NUM_CELLS {
        cell_grid.grid.push(Vec::new());
        for j in 0..NUM_CELLS {
            let x_pos =  -win_size.w / 1.0 + cell_size * i as f32;
            let y_pos =  win_size.h / 2.0 - cell_size * j as f32;

            let new_cell = Cell {
                state: CellState::Dead,
                position_x: i as u32,
                position_y: j as u32,
            };

            cell_grid.grid[i].push(new_cell.clone());

            commands.spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(cell_size, cell_size)),
                    ..Default::default()
                },
                transform: Transform {
                    translation: Vec3::new(x_pos, y_pos, 10.0),
                    ..Default::default()
                },
                ..Default::default()
            })
                .insert(new_cell);

            
        }
    }
    commands.insert_resource(cell_grid);

    // Insert RuleChanged resource
    commands.insert_resource(RuleChanged {
        updated_cell_grid: false,
        updated_sprites: false,
    });
}

// System used to update CellGrid resouce 
pub fn update_cell_grid_system(
    mut cell_grid: ResMut<CellGrid>, 
    cell_settings: Res<CellSettings>, 
    mut rule_changed: ResMut<RuleChanged>
) {
    let num_cells = cell_settings.num_cells;

    if !rule_changed.updated_cell_grid {

        if cell_settings.random {
            // Sets random values for first row
            let mut rng = rand::thread_rng();
            let coin = Uniform::from(0..2);
            for i in 0..num_cells {
                let value = coin.sample(&mut rng);
                if value == 0 {
                    cell_grid.grid[i as usize][0].state = CellState::Dead;
                }
                else {
                    cell_grid.grid[i as usize][0].state = CellState::Alive;
                }
            }
        }
        else {
            // Go through first row of cells and set them to Dead
            for i in 0..num_cells {
                cell_grid.grid[i as usize][0].state = CellState::Dead;
            }
            // Set middle cell of first row to Alive
            cell_grid.grid[(num_cells / 2) as usize][0].state = CellState::Alive;
        
        }
    
        // Go through each cell, row by row, skipping the first row,
        // and determine whether a cell should be alive or dead given a rule
        for j in 1..num_cells as usize {
            for i in 0..num_cells as usize {
    
                // Counter is used to index into the rule vec
                let mut counter = 0;
    
                if i != 0 && cell_grid.grid[i - 1][j - 1].state == CellState::Alive {
                    counter += 4;    
                }
                
                if i != (num_cells - 1) as usize && cell_grid.grid[i + 1][j - 1].state == CellState::Alive {
                    counter += 2;
                }
                
                if cell_grid.grid[i][j - 1].state == CellState::Alive {
                    counter += 1;   
                }
    
                // If the set rule states a cell should be alive, make it alive
                // Otherwise make it dead
                if cell_settings.rule[counter as usize] { cell_grid.grid[i][j].state = CellState::Alive}
                else { cell_grid.grid[i][j].state = CellState::Dead }
    
            }
        }

        rule_changed.updated_cell_grid = true;
    }
}

// System used to update sprites based on CellGrid Resource
pub fn color_grid_system(
    mut query: Query<(&mut Sprite, &mut Cell)>, 
    cell_grid: Res<CellGrid>, 
    cell_settings: Res<CellSettings>,
    mut rule_changed: ResMut<RuleChanged>,
) {
    if !rule_changed.updated_sprites {

        // Iterate through all cells
        for (mut sprite, mut cell) in query.iter_mut() {
            cell.state = cell_grid.grid[cell.position_x as usize][cell.position_y as usize].state;
            match cell.state {
                CellState::Alive => sprite.color = cell_settings.alive_color,
                CellState::Dead => sprite.color = cell_settings.dead_color,
            }
        }

        rule_changed.updated_sprites = true;
    }
}

// Updates rule based on mouse input
pub fn mouse_button_input_system (
    buttons: Res<Input<MouseButton>>, 
    mut cell_settings: ResMut<CellSettings>,
    mut rule_changed: ResMut<RuleChanged>,
) {
    // Right mouse button decreases rule number
    if buttons.just_pressed(MouseButton::Right) {
        cell_settings.rule_num -= 1;
        cell_settings.rule = get_rule(cell_settings.rule_num);

        rule_changed.updated_cell_grid = false;
        rule_changed.updated_sprites = false;
    }
    
    // Left mouse button increases rule number
    if buttons.just_pressed(MouseButton::Left) {
        cell_settings.rule_num += 1;
        cell_settings.rule = get_rule(cell_settings.rule_num);

        rule_changed.updated_cell_grid = false;
        rule_changed.updated_sprites = false;
    }

    // Middle mouse button pressed, toggle random first row
    if buttons.just_pressed(MouseButton::Middle) {
        cell_settings.random = !cell_settings.random;
        rule_changed.updated_cell_grid = false;
        rule_changed.updated_sprites = false;
    }
}

// Update cell positions based on keyboard input
pub fn key_press_system (
    keys: Res<Input<KeyCode>>,
    mut query: Query<(&mut Transform, &mut Sprite), With<Cell>>,
    mut settings: ResMut<CellSettings>,
) {
    // W key moves cells up
    if keys.pressed(KeyCode::W) {
        for (mut transform, _sprite) in query.iter_mut() {
            transform.translation.y -= 0.5 * settings.cell_size;
        }
    }
    
    // S key moves cells down
    if keys.pressed(KeyCode::S) {
        for (mut transform, _sprite) in query.iter_mut() {
            transform.translation.y += 0.5 * settings.cell_size;
        }
    }

    // D key moves cells right
    if keys.pressed(KeyCode::D) {
        for (mut transform, _sprite) in query.iter_mut() {
            transform.translation.x -= 0.5 * settings.cell_size;
        }
    }
    
    // A key moves cells left
    if keys.pressed(KeyCode::A) {
        for (mut transform, _sprite) in query.iter_mut() {
            transform.translation.x += 0.5 * settings.cell_size;
        }
    }
    
    // Constants for scaling
    const SCALE_MULTIPLIER: f32 = 1.02;
    const MAX_CELL_SIZE: f32 = 100.;
    const MIN_CELL_SIZE: f32 = 1.;
    
    // If Q is pressed, zooom in
    if keys.pressed(KeyCode::Q) {
        
        // Change cell_size by scale multiplier
        // If cells reach max size, do not scale anymore
        let mut changed: bool = true;
        settings.cell_size *= SCALE_MULTIPLIER;
        if settings.cell_size > MAX_CELL_SIZE {
            settings.cell_size = MAX_CELL_SIZE;
            changed = false;
        }

        // If cells are max size, do not scale
        if changed {
            for (mut transform, mut sprite) in query.iter_mut() {
                
                // First update scale then reposition cells
                sprite.custom_size = Some(Vec2::new(settings.cell_size, settings.cell_size));
                transform.translation.x *= SCALE_MULTIPLIER;
                transform.translation.y *= SCALE_MULTIPLIER;
            }
        }
    }
    
    // If E is pressed, zoom out
    if keys.pressed(KeyCode::E) {

        // Change cell_size by scale multiplier
        // If cells reach min size, do not scale anymore
        let mut changed: bool = true;
        settings.cell_size /= SCALE_MULTIPLIER;
        if settings.cell_size < MIN_CELL_SIZE {
            settings.cell_size = MIN_CELL_SIZE;
            changed = false;
        }
        
        // If cells are min size, do not scale
        if changed {
            for (mut transform, mut sprite) in query.iter_mut() {
                
                // First update scale then reposition cells
                sprite.custom_size = Some(Vec2::new(settings.cell_size, settings.cell_size));
                transform.translation.x /= SCALE_MULTIPLIER;
                transform.translation.y /= SCALE_MULTIPLIER;
            }
        }
    }
}

// Handles mouse scroll events
pub fn mouse_scroll_system (
    mut scoll_evr: EventReader<MouseWheel>,
    mut cell_settings: ResMut<CellSettings>,
    mut rule_changed: ResMut<RuleChanged>,
) {
    for ev in scoll_evr.iter() {
        match ev.unit {

            // If a desktop mouse is scrolled up or down, 
            // increase or decrease the rule num
            MouseScrollUnit::Line => {
                if ev.y > 0.0 {
                    cell_settings.rule_num += ev.y as u8;
                }
                else {
                    cell_settings.rule_num -= (-1.0 * ev.y) as u8;
                }
                cell_settings.rule = get_rule(cell_settings.rule_num);

                rule_changed.updated_cell_grid = false;
                rule_changed.updated_sprites = false;        
            }
            MouseScrollUnit::Pixel => {

            }
        }
    }
}

// Window resize system
// System moves cells so that the top of the automaton
// stays at the top when the window is resized
pub fn window_resize_system(
    resize_event: Res<Events<WindowResized>>, 
    mut win_size: ResMut<WinSize>,
    mut query: Query<&mut Transform, With<Cell>>,
) {

    // Initial height of window
    let init_height: f32 = win_size.h;

    // Checks for resize event and updates win_size resource
    let mut reader = resize_event.get_reader();
    for e in reader.iter(&resize_event) {
        win_size.w = e.width;
        win_size.h = e.height;
    }

    // Moves all cells by half of change in height
    for mut transform in query.iter_mut() {
        transform.translation.y += (win_size.h - init_height) / 2.0;
    }

}

// endregion:       Systems

// Used to calculate a rule array based on a given u8 value
fn get_rule(mut rule_num: u8) -> [bool; 8] {
    let mut rule: [bool; 8] = [false; 8];

    for i in 1..=8 {
        if rule_num / u8::pow(2, 8 - i) > 0 {
            rule_num -= u8::pow(2, 8 - i); 
            rule[8 - i as usize] = true;
        }
    }

    return rule;
}