mod ctrlc;
mod status;
mod ui;

use ncurses::*;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, ErrorKind, Write};
use std::process;
use status::*;
use ui::*;

fn list_delete(list: &mut Vec<String>, list_curr: &mut usize) {
    if *list_curr < list.len() {
        list.remove(*list_curr);
        if *list_curr >= list.len() && !list.is_empty() {
            *list_curr = list.len() - 1;
        }
    }
}

fn load_state(todos: &mut Vec<String>, dones: &mut Vec<String>, file_path: &str) -> io::Result<()> {
    let file = File::open(file_path)?;
    for (index, line) in io::BufReader::new(file).lines().enumerate() {
        match parse_item(&line?) {
            Some((Status::Todo, title)) => todos.push(title.to_string()),
            Some((Status::Done, title)) => dones.push(title.to_string()),
            None => {
                eprintln!("{}:{}: ERROR: ill-formed item line", file_path, index + 1);
                process::exit(1);
            }
        }
    }
    Ok(())
}

fn save_state(todos: &[String], dones: &[String], file_path: &str) {
    let mut file = File::create(file_path).unwrap();
    for todo in todos.iter() {
        writeln!(file, "TODO: {}", todo).unwrap();
    }
    for done in dones.iter() {
        writeln!(file, "DONE: {}", done).unwrap();
    }
}

fn main() {
    ctrlc::init();

    let mut args = env::args();
    args.next().unwrap();

    let file_path = match args.next() {
        Some(file_path) => file_path,
        None => {
            eprintln!("Usage: todo-rs <file-path>");
            eprintln!("ERROR: file path is not provided");
            process::exit(1);
        }
    };

    let mut todos = Vec::<String>::new();
    let mut todo_curr: usize = 0;
    let mut dones = Vec::<String>::new();
    let mut done_curr: usize = 0;

    let mut notification: String;

    match load_state(&mut todos, &mut dones, &file_path) {
        Ok(()) => notification = format!("Loaded file {}", file_path),
        Err(error) => {
            if error.kind() == ErrorKind::NotFound {
                notification = format!("New file {}", file_path)
            } else {
                panic!(
                    "Could not load state from file `{}`: {:?}",
                    file_path, error
                );
            }
        }
    };

    initscr();
    noecho();
    keypad(stdscr(), true);
    timeout(16); // running in 60 FPS
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

    start_color();
    init_color(0, 0, 43 * 4, 54 * 4);
    init_pair(REGULAR_PAIR, COLOR_WHITE, COLOR_GREEN);
    init_pair(HIGHLIGHT_PAIR, COLOR_GREEN, COLOR_WHITE);

    let mut quit = false;
    let mut panel = Status::Todo;
    let mut editing = false;
    let mut editing_cursor = 0;

    let mut ui = Ui::default();
    while !quit && !ctrlc::poll() {
        erase();

        let mut x = 0;
        let mut y = 0;
        getmaxyx(stdscr(), &mut y, &mut x);

        ui.begin(Vec2::new(0, 0), LayoutKind::Vert);
        {
            ui.label_fixed_width(&notification, x, REGULAR_PAIR);
            ui.label_fixed_width("", x, REGULAR_PAIR);

            ui.begin_layout(LayoutKind::Horz);
            {
                ui.begin_layout(LayoutKind::Vert);
                {
                    if panel == Status::Todo {
                        ui.label_fixed_width("TODO", x / 2, HIGHLIGHT_PAIR);
                        // TODO(#27): the item lists don't have a scroll area
                        for (index, todo) in todos.iter_mut().enumerate() {
                            if index == todo_curr {
                                if editing {
                                    ui.edit_field(todo, &mut editing_cursor, x / 2);

                                    if let Some('\n') = ui.key.take().map(|x| x as u8 as char) {
                                        editing = false;
                                    }
                                } else {
                                    ui.label_fixed_width(
                                        &format!("- [ ] {}", todo),
                                        x / 2,
                                        HIGHLIGHT_PAIR,
                                    );
                                    if let Some('r') = ui.key.map(|x| x as u8 as char) {
                                        editing = true;
                                        editing_cursor = todo.len();
                                        ui.key = None;
                                    }
                                }
                            } else {
                                ui.label_fixed_width(
                                    &format!("- [ ] {}", todo),
                                    x / 2,
                                    REGULAR_PAIR,
                                );
                            }
                        }

                        if let Some(key) = ui.key.take() {
                            match key as u8 as char {
                                'K' => list_drag_up(&mut todos, &mut todo_curr),
                                'J' => list_drag_down(&mut todos, &mut todo_curr),
                                'i' => {
                                    todos.insert(todo_curr, String::new());
                                    editing_cursor = 0;
                                    editing = true;
                                    notification.push_str("What needs to be done?");
                                }
                                'd' => {
                                    notification.push_str(
                                        "Can't remove items from TODO. Mark it as DONE first.",
                                    );
                                }
                                'k' => list_up(&mut todo_curr),
                                'j' => list_down(&todos, &mut todo_curr),
                                'g' => list_first(&mut todo_curr),
                                'G' => list_last(&todos, &mut todo_curr),
                                '\n' => {
                                    list_transfer(&mut dones, &mut todos, &mut todo_curr);
                                    notification.push_str("DONE!")
                                }
                                '\t' => {
                                    panel = panel.toggle();
                                }
                                _ => {
                                    ui.key = Some(key);
                                }
                            }
                        }
                    } else {
                        ui.label_fixed_width("TODO", x / 2, REGULAR_PAIR);
                        for todo in todos.iter() {
                            ui.label_fixed_width(&format!("- [ ] {}", todo), x / 2, REGULAR_PAIR);
                        }
                    }
                }
                ui.end_layout();

                ui.begin_layout(LayoutKind::Vert);
                {
                    if panel == Status::Done {
                        ui.label_fixed_width("DONE", x / 2, HIGHLIGHT_PAIR);
                        for (index, done) in dones.iter_mut().enumerate() {
                            if index == done_curr {
                                if editing {
                                    ui.edit_field(done, &mut editing_cursor, x / 2);

                                    if let Some('\n') = ui.key.take().map(|x| x as u8 as char) {
                                        editing = false;
                                    }
                                } else {
                                    ui.label_fixed_width(
                                        &format!("- [x] {}", done),
                                        x / 2,
                                        HIGHLIGHT_PAIR,
                                    );
                                    if let Some('r') = ui.key.map(|x| x as u8 as char) {
                                        editing = true;
                                        editing_cursor = done.len();
                                        ui.key = None;
                                    }
                                }
                            } else {
                                ui.label_fixed_width(
                                    &format!("- [x] {}", done),
                                    x / 2,
                                    REGULAR_PAIR,
                                );
                            }
                        }

                        if let Some(key) = ui.key.take() {
                            match key as u8 as char {
                                'K' => list_drag_up(&mut dones, &mut done_curr),
                                'J' => list_drag_down(&mut dones, &mut done_curr),
                                'k' => list_up(&mut done_curr),
                                'j' => list_down(&dones, &mut done_curr),
                                'g' => list_first(&mut done_curr),
                                'G' => list_last(&dones, &mut done_curr),
                                'i' => {
                                    notification.push_str(
                                        "Can't insert new DONE items. Only TODO is allowed.",
                                    );
                                }
                                'd' => {
                                    list_delete(&mut dones, &mut done_curr);
                                    notification.push_str("Into The Abyss!");
                                }
                                '\n' => {
                                    list_transfer(&mut todos, &mut dones, &mut done_curr);
                                    notification.push_str("No, not done yet...")
                                }
                                '\t' => {
                                    panel = panel.toggle();
                                }
                                _ => ui.key = Some(key),
                            }
                        }
                    } else {
                        ui.label_fixed_width("DONE", x / 2, REGULAR_PAIR);
                        for done in dones.iter() {
                            ui.label_fixed_width(&format!("- [x] {}", done), x / 2, REGULAR_PAIR);
                        }
                    }
                }
                ui.end_layout();
            }
            ui.end_layout();
        }
        ui.end();

        if let Some('q') = ui.key.take().map(|x| x as u8 as char) {
            quit = true;
        }

        refresh();

        let key = getch();
        if key != ERR {
            notification.clear();
            ui.key = Some(key);
        }
    }

    endwin();

    save_state(&todos, &dones, &file_path);
    println!("Saved state to {}", file_path);
}
