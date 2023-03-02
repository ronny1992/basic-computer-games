use crate::{model::{Galaxy, Pos, COURSES, EndPosition, self}, view, input};

pub fn perform_short_range_scan(galaxy: &Galaxy) {
    if galaxy.enterprise.damaged.contains_key(model::systems::SHORT_RANGE_SCAN) {
        view::scanners_out();
        return;
    }

    view::short_range_scan(&galaxy)
}

pub fn get_amount_and_set_shields(galaxy: &mut Galaxy, provided: Vec<String>) {

    if galaxy.enterprise.damaged.contains_key(model::systems::SHIELD_CONTROL) {
        view::inoperable("Shield Control");
        return;
    }

    view::energy_available(galaxy.enterprise.total_energy);
    let value = input::param_or_prompt_value(&provided, 0, "Number of units to shields", 0, i32::MAX);
    if value.is_none() {
        view::shields_unchanged();
        return;
    }

    let value = value.unwrap() as u16;
    if value > galaxy.enterprise.total_energy {
        view::ridiculous();
        view::shields_unchanged();
        return;
    }

    galaxy.enterprise.shields = value;
    view::shields_set(value);
}

pub fn gather_dir_and_speed_then_move(galaxy: &mut Galaxy, provided: Vec<String>) {

    let course = input::param_or_prompt_value(&provided, 0, "Course (1-9)?", 1, 9);
    if course.is_none() {
        view::bad_nav();
        return;
    }

    let course = course.unwrap();

    let mut max_warp = 8.0;
    if galaxy.enterprise.damaged.contains_key(model::systems::WARP_ENGINES) {
        max_warp = 0.2;
    }

    let speed = input::param_or_prompt_value(&provided, 1, format!("Warp Factor (0-{})?", max_warp).as_str(), 0.0, 8.0);
    if speed.is_none() {
        view::bad_nav();
        return;
    }
    
    let speed = speed.unwrap();

    if speed > max_warp {
        view::damaged_engines(max_warp, speed);
        return;
    }

    move_klingons_and_fire(galaxy);
    if galaxy.enterprise.destroyed {
        return;
    }
    move_enterprise(course, speed, galaxy);
}

fn move_enterprise(course: u8, warp_speed: f32, galaxy: &mut Galaxy) {

    let ship = &mut galaxy.enterprise;

    // todo account for being blocked

    let end = find_end_quadrant_sector(ship.quadrant, ship.sector, course, warp_speed);

    if end.energy_cost > ship.total_energy {
        view::insuffient_warp_energy(warp_speed);
        return
    }

    if end.hit_edge {
        view::hit_edge(&end);
    }
    

    if ship.quadrant != end.quadrant {
        view::enter_quadrant(&end.quadrant);
        
        if galaxy.quadrants[end.quadrant.as_index()].klingons.len() > 0 {
            view::condition_red();
            if ship.shields <= 200 {
                view::danger_shields();
            }
        }
    }

    ship.quadrant = end.quadrant;
    ship.sector = end.sector;

    ship.total_energy = (ship.total_energy - end.energy_cost).max(0);
    if ship.shields > ship.total_energy {
        view::divert_energy_from_shields();
        ship.shields = ship.total_energy;
    }

    view::short_range_scan(&galaxy)
}

fn find_end_quadrant_sector(start_quadrant: Pos, start_sector: Pos, course: u8, warp_speed: f32) -> EndPosition {
    let (dx, dy): (i8, i8) = COURSES[(course - 1) as usize];

    let mut distance = (warp_speed * 8.0) as i8;
    if distance == 0 {
        distance = 1;
    }

    let galaxy_pos = start_quadrant * 8u8 + start_sector;

    let mut nx = (galaxy_pos.0 as i8) + dx * distance;
    let mut ny = (galaxy_pos.1 as i8) + dy * distance;

    let hit_edge = nx < 0 || ny < 0 || nx >= 64 || ny >= 64;
    nx = nx.min(63).max(0);
    ny = ny.min(63).max(0);
    
    let quadrant = Pos((nx / 8) as u8, (ny / 8) as u8);
    let sector = Pos((nx % 8) as u8, (ny % 8) as u8);
    let energy_cost = distance as u16 + 10;

    EndPosition { quadrant, sector, hit_edge, energy_cost }
}

fn move_klingons_and_fire(galaxy: &mut Galaxy) {
    let quadrant = &mut galaxy.quadrants[galaxy.enterprise.quadrant.as_index()];
    for k in 0..quadrant.klingons.len() {
        let new_sector = quadrant.find_empty_sector();
        quadrant.klingons[k].sector = new_sector;
    }

    // todo: check if enterprise is protected by a starbase

    for k in 0..quadrant.klingons.len() {
        quadrant.klingons[k].fire_on(&mut galaxy.enterprise);
    }
}
